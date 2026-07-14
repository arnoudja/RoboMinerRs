mod support;
use serial_test::serial;

use support::*;

#[tokio::test]
#[serial]
async fn update_user_account_updates_profile_and_password() {
    let Ok(database_url) = std::env::var("ROBOMINER_DATABASE_URL") else {
        eprintln!("skipping DB integration test: ROBOMINER_DATABASE_URL is not set");
        return;
    };

    let pool = robominer_db::connect(&database_url)
        .await
        .expect("failed to connect to test database");
    let prefix = unique_test_prefix("rust-update-user-cli");
    let user_id = insert_test_user(
        &pool,
        &format!("{prefix}-old-user"),
        &format!("{prefix}-old@example.invalid"),
        "old-password-hash",
    )
    .await;
    let new_username = format!("{prefix}-new-user");
    let new_email = format!("{prefix}-new@example.invalid");

    let output = run_engine(&[
        "--database-url".to_string(),
        database_url,
        "update-user-account".to_string(),
        "--user-id".to_string(),
        user_id.to_string(),
        "--username".to_string(),
        new_username.clone(),
        "--email".to_string(),
        new_email.clone(),
        "--password".to_string(),
        "new-password".to_string(),
    ]);
    let (stdout, stderr) = output_text(&output);

    assert!(
        output.status.success(),
        "expected update-user-account to succeed\nstdout:\n{stdout}\nstderr:\n{stderr}"
    );
    assert!(
        stdout.contains("Updated user account"),
        "unexpected stdout:\n{stdout}"
    );
    assert!(stderr.is_empty(), "unexpected stderr:\n{stderr}");

    let user: (String, String, String) =
        sqlx::query_as("SELECT username, email, password FROM User WHERE id = ?")
            .bind(user_id)
            .fetch_one(&pool)
            .await
            .expect("failed to load updated user");
    assert_eq!(user.0, new_username);
    assert_eq!(user.1, new_email);
    assert!(
        user.2.starts_with("$argon2"),
        "unexpected password hash: {}",
        user.2
    );
    assert_ne!(user.2, "new-password");

    cleanup_created_user(&pool, user_id).await;
}

#[tokio::test]
#[serial]
async fn update_user_account_leaves_password_when_hash_is_absent() {
    let Ok(database_url) = std::env::var("ROBOMINER_DATABASE_URL") else {
        eprintln!("skipping DB integration test: ROBOMINER_DATABASE_URL is not set");
        return;
    };

    let pool = robominer_db::connect(&database_url)
        .await
        .expect("failed to connect to test database");
    let prefix = unique_test_prefix("rust-update-user-no-password-cli");
    let user_id = insert_test_user(
        &pool,
        &format!("{prefix}-old-user"),
        &format!("{prefix}-old@example.invalid"),
        "old-password-hash",
    )
    .await;
    let new_username = format!("{prefix}-new-user");
    let new_email = format!("{prefix}-new@example.invalid");

    let output = run_engine(&[
        "--database-url".to_string(),
        database_url,
        "update-user-account".to_string(),
        "--user-id".to_string(),
        user_id.to_string(),
        "--username".to_string(),
        new_username.clone(),
        "--email".to_string(),
        new_email.clone(),
    ]);
    let (stdout, stderr) = output_text(&output);

    assert!(
        output.status.success(),
        "expected update-user-account to succeed\nstdout:\n{stdout}\nstderr:\n{stderr}"
    );
    assert!(stderr.is_empty(), "unexpected stderr:\n{stderr}");

    let user: (String, String, String) =
        sqlx::query_as("SELECT username, email, password FROM User WHERE id = ?")
            .bind(user_id)
            .fetch_one(&pool)
            .await
            .expect("failed to load updated user");
    assert_eq!(
        user,
        (new_username, new_email, "old-password-hash".to_string())
    );

    cleanup_created_user(&pool, user_id).await;
}

#[tokio::test]
#[serial]
async fn account_state_reports_profile_fields() {
    let Ok(database_url) = std::env::var("ROBOMINER_DATABASE_URL") else {
        eprintln!("skipping DB integration test: ROBOMINER_DATABASE_URL is not set");
        return;
    };

    let pool = robominer_db::connect(&database_url)
        .await
        .expect("failed to connect to test database");
    let prefix = unique_test_prefix("rust-account-state-cli");
    let username = format!("{prefix}-user");
    let email = format!("{prefix}@example.invalid");
    let user_id = insert_test_user(&pool, &username, &email, "test-password-hash").await;

    let output = run_engine(&[
        "--database-url".to_string(),
        database_url,
        "account-state".to_string(),
        "--user-id".to_string(),
        user_id.to_string(),
    ]);
    let (stdout, stderr) = output_text(&output);

    assert!(
        output.status.success(),
        "expected account-state to succeed\nstdout:\n{stdout}\nstderr:\n{stderr}"
    );
    assert!(stderr.is_empty(), "unexpected stderr:\n{stderr}");

    let fields = find_prefixed_line(&stdout, "U\t");
    assert_eq!(fields, vec!["U", username.as_str(), email.as_str()]);

    cleanup_created_user(&pool, user_id).await;
}

#[tokio::test]
#[serial]
async fn update_user_account_rejects_duplicate_username_and_email() {
    let Ok(database_url) = std::env::var("ROBOMINER_DATABASE_URL") else {
        eprintln!("skipping DB integration test: ROBOMINER_DATABASE_URL is not set");
        return;
    };

    let pool = robominer_db::connect(&database_url)
        .await
        .expect("failed to connect to test database");
    let prefix = unique_test_prefix("rust-update-user-duplicate-cli");
    let user_id = insert_test_user(
        &pool,
        &format!("{prefix}-user"),
        &format!("{prefix}-user@example.invalid"),
        "user-password-hash",
    )
    .await;
    let other_user_id = insert_test_user(
        &pool,
        &format!("{prefix}-other-user"),
        &format!("{prefix}-other@example.invalid"),
        "other-password-hash",
    )
    .await;

    let duplicate_username = run_engine(&[
        "--database-url".to_string(),
        database_url.clone(),
        "update-user-account".to_string(),
        "--user-id".to_string(),
        user_id.to_string(),
        "--username".to_string(),
        format!("{prefix}-other-user"),
        "--email".to_string(),
        format!("{prefix}-updated@example.invalid"),
    ]);
    let (stdout, stderr) = output_text(&duplicate_username);
    assert!(
        !duplicate_username.status.success(),
        "expected duplicate username update to fail\nstdout:\n{stdout}\nstderr:\n{stderr}"
    );
    assert!(
        stderr.contains("duplicate username"),
        "unexpected stderr:\n{stderr}"
    );

    let duplicate_email = run_engine(&[
        "--database-url".to_string(),
        database_url,
        "update-user-account".to_string(),
        "--user-id".to_string(),
        user_id.to_string(),
        "--username".to_string(),
        format!("{prefix}-updated-user"),
        "--email".to_string(),
        format!("{prefix}-other@example.invalid"),
    ]);
    let (stdout, stderr) = output_text(&duplicate_email);
    assert!(
        !duplicate_email.status.success(),
        "expected duplicate email update to fail\nstdout:\n{stdout}\nstderr:\n{stderr}"
    );
    assert!(
        stderr.contains("duplicate email"),
        "unexpected stderr:\n{stderr}"
    );

    let original: (String, String, String) =
        sqlx::query_as("SELECT username, email, password FROM User WHERE id = ?")
            .bind(user_id)
            .fetch_one(&pool)
            .await
            .expect("failed to load original user");
    assert_eq!(
        original,
        (
            format!("{prefix}-user"),
            format!("{prefix}-user@example.invalid"),
            "user-password-hash".to_string()
        )
    );

    cleanup_created_user(&pool, user_id).await;
    cleanup_created_user(&pool, other_user_id).await;
}

#[tokio::test]
#[serial]
async fn update_user_account_rejects_invalid_profile_fields() {
    let Ok(database_url) = std::env::var("ROBOMINER_DATABASE_URL") else {
        eprintln!("skipping DB integration test: ROBOMINER_DATABASE_URL is not set");
        return;
    };

    let pool = robominer_db::connect(&database_url)
        .await
        .expect("failed to connect to test database");
    let prefix = unique_test_prefix("rust-update-user-invalid-cli");
    let user_id = insert_test_user(
        &pool,
        &format!("{prefix}-user"),
        &format!("{prefix}-user@example.invalid"),
        "user-password-hash",
    )
    .await;

    let invalid_username = run_engine(&[
        "--database-url".to_string(),
        database_url.clone(),
        "update-user-account".to_string(),
        "--user-id".to_string(),
        user_id.to_string(),
        "--username".to_string(),
        "bad user".to_string(),
        "--email".to_string(),
        format!("{prefix}-updated@example.invalid"),
    ]);
    let (_, stderr) = output_text(&invalid_username);
    assert!(
        !invalid_username.status.success(),
        "expected invalid username update to fail"
    );
    assert!(
        stderr.contains("invalid username"),
        "unexpected stderr:\n{stderr}"
    );

    let invalid_email = run_engine(&[
        "--database-url".to_string(),
        database_url.clone(),
        "update-user-account".to_string(),
        "--user-id".to_string(),
        user_id.to_string(),
        "--username".to_string(),
        format!("{prefix}-updated-user"),
        "--email".to_string(),
        "invalid-email".to_string(),
    ]);
    let (_, stderr) = output_text(&invalid_email);
    assert!(
        !invalid_email.status.success(),
        "expected invalid email update to fail"
    );
    assert!(
        stderr.contains("invalid email"),
        "unexpected stderr:\n{stderr}"
    );

    let invalid_password = run_engine(&[
        "--database-url".to_string(),
        database_url,
        "update-user-account".to_string(),
        "--user-id".to_string(),
        user_id.to_string(),
        "--username".to_string(),
        format!("{prefix}-updated-user"),
        "--email".to_string(),
        format!("{prefix}-updated@example.invalid"),
        "--password".to_string(),
        "short".to_string(),
    ]);
    let (_, stderr) = output_text(&invalid_password);
    assert!(
        !invalid_password.status.success(),
        "expected invalid password update to fail"
    );
    assert!(
        stderr.contains("invalid password"),
        "unexpected stderr:\n{stderr}"
    );

    cleanup_created_user(&pool, user_id).await;
}
