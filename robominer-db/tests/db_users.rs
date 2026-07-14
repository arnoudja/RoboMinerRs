use robominer_db::{
    CreateUserRejection, CreateUserRequest, UpdateUserAccountRejection, UpdateUserAccountRequest,
    VerifyLoginRejection, VerifyLoginRequest, create_user, get_user_by_id, update_user_account,
    verify_login,
};
use robominer_test_support::{
    ensure_default_robot_parts, insert_user_with_credentials, unique_prefix,
};
use serial_test::serial;

#[tokio::test]
#[serial]
async fn create_user_rejects_invalid_profile_fields() {
    let Ok(database_url) = std::env::var("ROBOMINER_DATABASE_URL") else {
        eprintln!("skipping robominer-db users test: ROBOMINER_DATABASE_URL is not set");
        return;
    };

    let pool = robominer_db::connect(&database_url)
        .await
        .expect("failed to connect to test database");
    ensure_default_robot_parts(&pool).await;

    let rejection = create_user(
        &pool,
        CreateUserRequest {
            username: "ab".to_string(),
            email: "player@example.invalid".to_string(),
            password: "test-password".to_string(),
        },
    )
    .await
    .expect("create should not fail at sql layer")
    .expect_err("short username should reject");

    assert_eq!(rejection, CreateUserRejection::InvalidUsername);
}

#[tokio::test]
#[serial]
async fn create_user_inserts_user_and_claims_initial_achievement() {
    let Ok(database_url) = std::env::var("ROBOMINER_DATABASE_URL") else {
        eprintln!("skipping robominer-db users test: ROBOMINER_DATABASE_URL is not set");
        return;
    };

    let pool = robominer_db::connect(&database_url)
        .await
        .expect("failed to connect to test database");
    ensure_default_robot_parts(&pool).await;

    let prefix = unique_prefix("rust-db-create-user");
    let username = format!("{prefix}-user");
    let email = format!("{prefix}@example.invalid");
    let password = "test-password-1".to_string();

    let created = create_user(
        &pool,
        CreateUserRequest {
            username: username.clone(),
            email: email.clone(),
            password: password.clone(),
        },
    )
    .await
    .expect("create should not fail at sql layer")
    .expect("create should succeed");

    let user = get_user_by_id(&pool, created.user_id)
        .await
        .expect("load should not fail")
        .expect("created user should exist");
    assert_eq!(user.username, username);
    assert_eq!(user.email, email);
    assert!(user.password_hash.starts_with("$argon2"));
    assert_eq!(user.achievement_points, 10);
    assert_eq!(user.mining_queue_size, 1);

    let steps_claimed: i32 = sqlx::query_scalar(
        "SELECT stepsClaimed FROM UserAchievement WHERE userId = ? AND achievementId = 1",
    )
    .bind(created.user_id)
    .fetch_one(&pool)
    .await
    .expect("failed to load initial achievement");
    assert_eq!(steps_claimed, 1);

    cleanup_created_user(&pool, created.user_id).await;
}

#[tokio::test]
#[serial]
async fn create_user_rejects_duplicate_username() {
    let Ok(database_url) = std::env::var("ROBOMINER_DATABASE_URL") else {
        eprintln!("skipping robominer-db users test: ROBOMINER_DATABASE_URL is not set");
        return;
    };

    let pool = robominer_db::connect(&database_url)
        .await
        .expect("failed to connect to test database");
    let prefix = unique_prefix("rust-db-duplicate-user");
    let username = format!("{prefix}-user");
    let user_id = insert_user_with_credentials(
        &pool,
        &username,
        &format!("{prefix}@example.invalid"),
        "test-password",
    )
    .await;

    let rejection = create_user(
        &pool,
        CreateUserRequest {
            username,
            email: format!("{prefix}-other@example.invalid"),
            password: "test-password-1".to_string(),
        },
    )
    .await
    .expect("create should not fail at sql layer")
    .expect_err("duplicate username should reject");

    assert_eq!(rejection, CreateUserRejection::DuplicateUsername);

    let _ = sqlx::query("DELETE FROM User WHERE id = ?")
        .bind(user_id)
        .execute(&pool)
        .await;
}

#[tokio::test]
#[serial]
async fn verify_login_accepts_username_or_email() {
    let Ok(database_url) = std::env::var("ROBOMINER_DATABASE_URL") else {
        eprintln!("skipping robominer-db users test: ROBOMINER_DATABASE_URL is not set");
        return;
    };

    let pool = robominer_db::connect(&database_url)
        .await
        .expect("failed to connect to test database");
    ensure_default_robot_parts(&pool).await;

    let prefix = unique_prefix("rust-db-verify-login");
    let username = format!("{prefix}-user");
    let email = format!("{prefix}@example.invalid");
    let password = "test-password-1".to_string();
    let created = create_user(
        &pool,
        CreateUserRequest {
            username: username.clone(),
            email: email.clone(),
            password: password.clone(),
        },
    )
    .await
    .expect("create should not fail at sql layer")
    .expect("create should succeed");

    let by_username = verify_login(
        &pool,
        VerifyLoginRequest {
            login_name: username.clone(),
            password: password.clone(),
        },
    )
    .await
    .expect("verify should not fail at sql layer")
    .expect("username login should succeed");
    assert_eq!(by_username.user_id, created.user_id);

    let by_email = verify_login(
        &pool,
        VerifyLoginRequest {
            login_name: email,
            password,
        },
    )
    .await
    .expect("verify should not fail at sql layer")
    .expect("email login should succeed");
    assert_eq!(by_email.user_id, created.user_id);

    cleanup_created_user(&pool, created.user_id).await;
}

#[tokio::test]
#[serial]
async fn verify_login_rejects_invalid_password() {
    let Ok(database_url) = std::env::var("ROBOMINER_DATABASE_URL") else {
        eprintln!("skipping robominer-db users test: ROBOMINER_DATABASE_URL is not set");
        return;
    };

    let pool = robominer_db::connect(&database_url)
        .await
        .expect("failed to connect to test database");
    ensure_default_robot_parts(&pool).await;

    let prefix = unique_prefix("rust-db-verify-login-fail");
    let username = format!("{prefix}-user");
    let created = create_user(
        &pool,
        CreateUserRequest {
            username: username.clone(),
            email: format!("{prefix}@example.invalid"),
            password: "test-password-1".to_string(),
        },
    )
    .await
    .expect("create should not fail at sql layer")
    .expect("create should succeed");

    let rejection = verify_login(
        &pool,
        VerifyLoginRequest {
            login_name: username,
            password: "wrong-password".to_string(),
        },
    )
    .await
    .expect("verify should not fail at sql layer")
    .expect_err("wrong password should reject");

    assert_eq!(rejection, VerifyLoginRejection::InvalidPassword);

    cleanup_created_user(&pool, created.user_id).await;
}

#[tokio::test]
#[serial]
async fn update_user_account_rejects_duplicate_email() {
    let Ok(database_url) = std::env::var("ROBOMINER_DATABASE_URL") else {
        eprintln!("skipping robominer-db users test: ROBOMINER_DATABASE_URL is not set");
        return;
    };

    let pool = robominer_db::connect(&database_url)
        .await
        .expect("failed to connect to test database");
    let prefix = unique_prefix("rust-db-update-user");
    let first_user_id = insert_user_with_credentials(
        &pool,
        &format!("{prefix}-first"),
        &format!("{prefix}-first@example.invalid"),
        "test-password",
    )
    .await;
    let second_user_id = insert_user_with_credentials(
        &pool,
        &format!("{prefix}-second"),
        &format!("{prefix}-second@example.invalid"),
        "test-password",
    )
    .await;

    let rejection = update_user_account(
        &pool,
        UpdateUserAccountRequest {
            user_id: second_user_id,
            username: format!("{prefix}-second-renamed"),
            email: format!("{prefix}-first@example.invalid"),
            password: None,
        },
    )
    .await
    .expect("update should not fail at sql layer")
    .expect_err("duplicate email should reject");

    assert_eq!(rejection, UpdateUserAccountRejection::DuplicateEmail);

    let _ = sqlx::query("DELETE FROM User WHERE id IN (?, ?)")
        .bind(first_user_id)
        .bind(second_user_id)
        .execute(&pool)
        .await;
}

async fn cleanup_created_user(pool: &robominer_db::MySqlPool, user_id: i64) {
    let _ = sqlx::query("DELETE FROM Robot WHERE userId = ?")
        .bind(user_id)
        .execute(pool)
        .await;
    let _ = sqlx::query("DELETE FROM ProgramSource WHERE userId = ?")
        .bind(user_id)
        .execute(pool)
        .await;
    let _ = sqlx::query("DELETE FROM UserRobotPartAsset WHERE userId = ?")
        .bind(user_id)
        .execute(pool)
        .await;
    let _ = sqlx::query("DELETE FROM UserOreAsset WHERE userId = ?")
        .bind(user_id)
        .execute(pool)
        .await;
    let _ = sqlx::query("DELETE FROM UserAchievement WHERE userId = ?")
        .bind(user_id)
        .execute(pool)
        .await;
    let _ = sqlx::query("DELETE FROM User WHERE id = ?")
        .bind(user_id)
        .execute(pool)
        .await;
}
