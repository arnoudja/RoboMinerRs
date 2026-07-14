mod support;
use serial_test::serial;

use support::*;

#[tokio::test]
#[serial]
async fn create_user_inserts_initial_user_state() {
    let Ok(database_url) = std::env::var("ROBOMINER_DATABASE_URL") else {
        eprintln!("skipping DB integration test: ROBOMINER_DATABASE_URL is not set");
        return;
    };

    let pool = robominer_db::connect(&database_url)
        .await
        .expect("failed to connect to test database");
    ensure_default_robot_parts(&pool).await;
    let prefix = unique_test_prefix("rust-create-user-cli");
    let username = format!("{prefix}-user");
    let email = format!("{prefix}@example.invalid");
    let password = "test-password";

    let output = run_engine(&[
        "--database-url".to_string(),
        database_url,
        "create-user".to_string(),
        "--username".to_string(),
        username.clone(),
        "--email".to_string(),
        email.clone(),
        "--password".to_string(),
        password.to_string(),
    ]);
    let (stdout, stderr) = output_text(&output);

    assert!(
        output.status.success(),
        "expected create-user to succeed\nstdout:\n{stdout}\nstderr:\n{stderr}"
    );
    assert!(stderr.is_empty(), "unexpected stderr:\n{stderr}");
    let user_id: i64 = stdout
        .trim()
        .parse()
        .expect("create-user should print the new user id");

    let user: (String, String, String, i32, i32) = sqlx::query_as(
        "SELECT username, email, password, achievementPoints, miningQueueSize \
         FROM User \
         WHERE id = ?",
    )
    .bind(user_id)
    .fetch_one(&pool)
    .await
    .expect("failed to load created user");
    assert_eq!(user.0, username);
    assert_eq!(user.1, email);
    assert!(
        user.2.starts_with("$argon2"),
        "unexpected password hash: {}",
        user.2
    );
    assert_ne!(user.2, password);
    assert_eq!(user.3, 10);
    assert_eq!(user.4, 1);

    let steps_claimed: i32 = sqlx::query_scalar(
        "SELECT stepsClaimed FROM UserAchievement WHERE userId = ? AND achievementId = 1",
    )
    .bind(user_id)
    .fetch_one(&pool)
    .await
    .expect("failed to load initial achievement");
    assert_eq!(steps_claimed, 1);

    let robot_count: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM Robot WHERE userId = ?")
        .bind(user_id)
        .fetch_one(&pool)
        .await
        .expect("failed to count created robots");
    assert_eq!(robot_count, 1);

    let default_part_assets: i64 = sqlx::query_scalar(
        "SELECT COUNT(*) FROM UserRobotPartAsset \
         WHERE userId = ? AND robotPartId IN (101, 201, 301, 401, 501, 601)",
    )
    .bind(user_id)
    .fetch_one(&pool)
    .await
    .expect("failed to count default part assets");
    assert_eq!(default_part_assets, 6);

    let first_area_unlocked: i64 = sqlx::query_scalar(
        "SELECT COUNT(*) FROM UserMiningArea WHERE userId = ? AND miningAreaId = 1001",
    )
    .bind(user_id)
    .fetch_one(&pool)
    .await
    .expect("failed to count initial mining area");
    assert_eq!(first_area_unlocked, 1);

    cleanup_created_user(&pool, user_id).await;
}

#[tokio::test]
#[serial]
async fn create_user_rejects_duplicate_username_and_email() {
    let Ok(database_url) = std::env::var("ROBOMINER_DATABASE_URL") else {
        eprintln!("skipping DB integration test: ROBOMINER_DATABASE_URL is not set");
        return;
    };

    let pool = robominer_db::connect(&database_url)
        .await
        .expect("failed to connect to test database");
    ensure_default_robot_parts(&pool).await;
    let prefix = unique_test_prefix("rust-create-user-duplicate-cli");
    let username = format!("{prefix}-user");
    let email = format!("{prefix}@example.invalid");

    let first = run_engine(&[
        "--database-url".to_string(),
        database_url.clone(),
        "create-user".to_string(),
        "--username".to_string(),
        username.clone(),
        "--email".to_string(),
        email.clone(),
        "--password".to_string(),
        "test-password".to_string(),
    ]);
    let (first_stdout, first_stderr) = output_text(&first);
    assert!(
        first.status.success(),
        "expected first create-user to succeed\nstdout:\n{first_stdout}\nstderr:\n{first_stderr}"
    );
    let user_id: i64 = first_stdout
        .trim()
        .parse()
        .expect("create-user should print the new user id");

    let duplicate_username = run_engine(&[
        "--database-url".to_string(),
        database_url.clone(),
        "create-user".to_string(),
        "--username".to_string(),
        username,
        "--email".to_string(),
        format!("{prefix}-other@example.invalid"),
        "--password".to_string(),
        "test-password".to_string(),
    ]);
    let (stdout, stderr) = output_text(&duplicate_username);
    assert!(
        !duplicate_username.status.success(),
        "expected duplicate username create-user to fail\nstdout:\n{stdout}\nstderr:\n{stderr}"
    );
    assert!(
        stderr.contains("duplicate username"),
        "unexpected stderr:\n{stderr}"
    );

    let duplicate_email = run_engine(&[
        "--database-url".to_string(),
        database_url,
        "create-user".to_string(),
        "--username".to_string(),
        format!("{prefix}-other-user"),
        "--email".to_string(),
        email,
        "--password".to_string(),
        "test-password".to_string(),
    ]);
    let (stdout, stderr) = output_text(&duplicate_email);
    assert!(
        !duplicate_email.status.success(),
        "expected duplicate email create-user to fail\nstdout:\n{stdout}\nstderr:\n{stderr}"
    );
    assert!(
        stderr.contains("duplicate email"),
        "unexpected stderr:\n{stderr}"
    );

    cleanup_created_user(&pool, user_id).await;
}

#[tokio::test]
#[serial]
async fn create_user_rejects_invalid_profile_fields() {
    let Ok(database_url) = std::env::var("ROBOMINER_DATABASE_URL") else {
        eprintln!("skipping DB integration test: ROBOMINER_DATABASE_URL is not set");
        return;
    };

    let invalid_username = run_engine(&[
        "--database-url".to_string(),
        database_url.clone(),
        "create-user".to_string(),
        "--username".to_string(),
        "bad user".to_string(),
        "--email".to_string(),
        "valid@example.invalid".to_string(),
        "--password".to_string(),
        "test-password".to_string(),
    ]);
    let (_, stderr) = output_text(&invalid_username);
    assert!(
        !invalid_username.status.success(),
        "expected invalid username create-user to fail"
    );
    assert!(
        stderr.contains("invalid username"),
        "unexpected stderr:\n{stderr}"
    );

    let invalid_email = run_engine(&[
        "--database-url".to_string(),
        database_url.clone(),
        "create-user".to_string(),
        "--username".to_string(),
        "validuser".to_string(),
        "--email".to_string(),
        "invalid-email".to_string(),
        "--password".to_string(),
        "test-password".to_string(),
    ]);
    let (_, stderr) = output_text(&invalid_email);
    assert!(
        !invalid_email.status.success(),
        "expected invalid email create-user to fail"
    );
    assert!(
        stderr.contains("invalid email"),
        "unexpected stderr:\n{stderr}"
    );

    let invalid_password = run_engine(&[
        "--database-url".to_string(),
        database_url,
        "create-user".to_string(),
        "--username".to_string(),
        "validuser".to_string(),
        "--email".to_string(),
        "valid@example.invalid".to_string(),
        "--password".to_string(),
        "short".to_string(),
    ]);
    let (_, stderr) = output_text(&invalid_password);
    assert!(
        !invalid_password.status.success(),
        "expected invalid password create-user to fail"
    );
    assert!(
        stderr.contains("invalid password"),
        "unexpected stderr:\n{stderr}"
    );
}
