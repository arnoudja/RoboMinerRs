mod support;
use serial_test::serial;

use support::*;

#[tokio::test]
#[serial]
async fn verify_login_accepts_username_and_email_and_updates_last_login() {
    let Ok(database_url) = std::env::var("ROBOMINER_DATABASE_URL") else {
        eprintln!("skipping DB integration test: ROBOMINER_DATABASE_URL is not set");
        return;
    };

    let pool = robominer_db::connect(&database_url)
        .await
        .expect("failed to connect to test database");
    ensure_default_robot_parts(&pool).await;
    let prefix = unique_test_prefix("rust-verify-login-cli");
    let username = format!("{prefix}-user");
    let email = format!("{prefix}@example.invalid");
    let password = "test-password";

    let create = run_engine(&[
        "--database-url".to_string(),
        database_url.clone(),
        "create-user".to_string(),
        "--username".to_string(),
        username.clone(),
        "--email".to_string(),
        email.clone(),
        "--password".to_string(),
        password.to_string(),
    ]);
    let (stdout, stderr) = output_text(&create);
    assert!(
        create.status.success(),
        "expected create-user to succeed\nstdout:\n{stdout}\nstderr:\n{stderr}"
    );
    let user_id: i64 = stdout.trim().parse().expect("new user id should parse");

    sqlx::query(
        "UPDATE User SET password = ( \
            SELECT CONCAT('sha256:', salt, ':', SHA2(CONCAT(salt, ?), 256)) \
            FROM (SELECT REPLACE(UUID(), '-', '') AS salt) generated_salt \
         ) \
         WHERE id = ?",
    )
    .bind(password)
    .bind(user_id)
    .execute(&pool)
    .await
    .expect("failed to seed legacy password hash");

    sqlx::query("UPDATE User SET lastLoginTime = TIMESTAMPADD(DAY, -1, NOW()) WHERE id = ?")
        .bind(user_id)
        .execute(&pool)
        .await
        .expect("failed to backdate login time");

    let username_login = run_engine(&[
        "--database-url".to_string(),
        database_url.clone(),
        "verify-login".to_string(),
        "--login-name".to_string(),
        username,
        "--password".to_string(),
        password.to_string(),
    ]);
    let (stdout, stderr) = output_text(&username_login);
    assert!(
        username_login.status.success(),
        "expected username login to succeed\nstdout:\n{stdout}\nstderr:\n{stderr}"
    );
    assert_eq!(stdout.trim(), user_id.to_string());

    let updated_recently: i64 = sqlx::query_scalar(
        "SELECT COUNT(*) FROM User \
         WHERE id = ? AND lastLoginTime > TIMESTAMPADD(MINUTE, -1, NOW())",
    )
    .bind(user_id)
    .fetch_one(&pool)
    .await
    .expect("failed to check login time");
    assert_eq!(updated_recently, 1);

    let upgraded_hash: String = sqlx::query_scalar("SELECT password FROM User WHERE id = ?")
        .bind(user_id)
        .fetch_one(&pool)
        .await
        .expect("failed to load upgraded password hash");
    assert!(
        upgraded_hash.starts_with("$argon2"),
        "expected legacy password hash to upgrade on login, got: {upgraded_hash}"
    );

    let email_login = run_engine(&[
        "--database-url".to_string(),
        database_url,
        "verify-login".to_string(),
        "--login-name".to_string(),
        email,
        "--password".to_string(),
        password.to_string(),
    ]);
    let (stdout, stderr) = output_text(&email_login);
    assert!(
        email_login.status.success(),
        "expected email login to succeed\nstdout:\n{stdout}\nstderr:\n{stderr}"
    );
    assert_eq!(stdout.trim(), user_id.to_string());

    cleanup_created_user(&pool, user_id).await;
}

#[tokio::test]
#[serial]
async fn verify_login_rejects_unknown_user_and_bad_password() {
    let Ok(database_url) = std::env::var("ROBOMINER_DATABASE_URL") else {
        eprintln!("skipping DB integration test: ROBOMINER_DATABASE_URL is not set");
        return;
    };

    let pool = robominer_db::connect(&database_url)
        .await
        .expect("failed to connect to test database");
    ensure_default_robot_parts(&pool).await;
    let prefix = unique_test_prefix("rust-verify-login-reject-cli");
    let username = format!("{prefix}-user");
    let email = format!("{prefix}@example.invalid");

    let create = run_engine(&[
        "--database-url".to_string(),
        database_url.clone(),
        "create-user".to_string(),
        "--username".to_string(),
        username.clone(),
        "--email".to_string(),
        email,
        "--password".to_string(),
        "test-password".to_string(),
    ]);
    let (stdout, stderr) = output_text(&create);
    assert!(
        create.status.success(),
        "expected create-user to succeed\nstdout:\n{stdout}\nstderr:\n{stderr}"
    );
    let user_id: i64 = stdout.trim().parse().expect("new user id should parse");

    let bad_password = run_engine(&[
        "--database-url".to_string(),
        database_url.clone(),
        "verify-login".to_string(),
        "--login-name".to_string(),
        username,
        "--password".to_string(),
        "wrong-password".to_string(),
    ]);
    let (stdout, stderr) = output_text(&bad_password);
    assert!(
        !bad_password.status.success(),
        "expected bad password login to fail\nstdout:\n{stdout}\nstderr:\n{stderr}"
    );
    assert!(
        stderr.contains("invalid password"),
        "unexpected stderr:\n{stderr}"
    );

    let unknown_user = run_engine(&[
        "--database-url".to_string(),
        database_url,
        "verify-login".to_string(),
        "--login-name".to_string(),
        format!("{prefix}-missing"),
        "--password".to_string(),
        "test-password".to_string(),
    ]);
    let (stdout, stderr) = output_text(&unknown_user);
    assert!(
        !unknown_user.status.success(),
        "expected unknown user login to fail\nstdout:\n{stdout}\nstderr:\n{stderr}"
    );
    assert!(
        stderr.contains("unknown user"),
        "unexpected stderr:\n{stderr}"
    );

    cleanup_created_user(&pool, user_id).await;
}

#[tokio::test]
#[serial]
async fn verify_user_password_checks_current_password_without_touching_login_time() {
    let Ok(database_url) = std::env::var("ROBOMINER_DATABASE_URL") else {
        eprintln!("skipping DB integration test: ROBOMINER_DATABASE_URL is not set");
        return;
    };

    let pool = robominer_db::connect(&database_url)
        .await
        .expect("failed to connect to test database");
    ensure_default_robot_parts(&pool).await;
    let prefix = unique_test_prefix("rust-verify-user-password-cli");
    let username = format!("{prefix}-user");
    let email = format!("{prefix}@example.invalid");

    let create = run_engine(&[
        "--database-url".to_string(),
        database_url.clone(),
        "create-user".to_string(),
        "--username".to_string(),
        username,
        "--email".to_string(),
        email,
        "--password".to_string(),
        "test-password".to_string(),
    ]);
    let (stdout, stderr) = output_text(&create);
    assert!(
        create.status.success(),
        "expected create-user to succeed\nstdout:\n{stdout}\nstderr:\n{stderr}"
    );
    let user_id: i64 = stdout.trim().parse().expect("new user id should parse");

    sqlx::query("UPDATE User SET lastLoginTime = TIMESTAMPADD(DAY, -1, NOW()) WHERE id = ?")
        .bind(user_id)
        .execute(&pool)
        .await
        .expect("failed to backdate login time");

    let verify = run_engine(&[
        "--database-url".to_string(),
        database_url,
        "verify-user-password".to_string(),
        "--user-id".to_string(),
        user_id.to_string(),
        "--password".to_string(),
        "test-password".to_string(),
    ]);
    let (stdout, stderr) = output_text(&verify);
    assert!(
        verify.status.success(),
        "expected password verification to succeed\nstdout:\n{stdout}\nstderr:\n{stderr}"
    );
    assert_eq!(stdout.trim(), user_id.to_string());

    let updated_recently: i64 = sqlx::query_scalar(
        "SELECT COUNT(*) FROM User \
         WHERE id = ? AND lastLoginTime > TIMESTAMPADD(MINUTE, -1, NOW())",
    )
    .bind(user_id)
    .fetch_one(&pool)
    .await
    .expect("failed to check login time");
    assert_eq!(updated_recently, 0);

    cleanup_created_user(&pool, user_id).await;
}
