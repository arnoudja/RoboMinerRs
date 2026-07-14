mod support;

use std::collections::HashMap;

use robominer_web::test_support::route;
use serial_test::serial;
use support::{
    cookie_header, create_user_via_engine, ensure_session_configured, login_with_credentials,
    post_request, response_body, server_config, unique_prefix,
};

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
#[serial]
async fn account_update_post_persists_profile_changes() {
    if std::env::var("ROBOMINER_DATABASE_URL").is_err() {
        eprintln!("skipping account web test: ROBOMINER_DATABASE_URL is not set");
        return;
    }

    ensure_session_configured();

    let pool = robominer_db::connect(&std::env::var("ROBOMINER_DATABASE_URL").expect("database url"))
        .await
        .expect("failed to connect to test database");
    let prefix = unique_prefix("rust-web-account");
    let username = format!("{prefix}-user");
    let password = "test-password-1".to_string();
    let user_id = create_user_via_engine(
        &username,
        &format!("{prefix}@example.invalid"),
        &password,
    );
    let config = server_config(pool.clone());
    let cookie = cookie_header(&login_with_credentials(&config, &username, &password));

    let updated_username = format!("{prefix}-renamed");
    let updated_email = format!("{prefix}-renamed@example.invalid");
    let mut form = HashMap::new();
    form.insert("username".to_string(), updated_username.clone());
    form.insert("email".to_string(), updated_email.clone());
    form.insert("currentpassword".to_string(), password.clone());
    form.insert("newpassword".to_string(), String::new());
    form.insert("confirmpassword".to_string(), String::new());

    let response = route(&post_request("/account", form, Some(&cookie)), &config);
    let body = response_body(&response);

    assert_eq!(response.status, 200, "account page should render");
    assert!(
        body.contains("Account information updated"),
        "expected account update success message:\n{body}"
    );
    assert!(
        body.contains(&updated_username),
        "expected updated username in account page body:\n{body}"
    );

    let stored: (String, String) =
        sqlx::query_as("SELECT username, email FROM User WHERE id = ?")
            .bind(user_id)
            .fetch_one(&pool)
            .await
            .expect("failed to load updated user");
    assert_eq!(stored.0, updated_username);
    assert_eq!(stored.1, updated_email);

    let _ = sqlx::query("DELETE FROM Robot WHERE userId = ?")
        .bind(user_id)
        .execute(&pool)
        .await;
    let _ = sqlx::query("DELETE FROM User WHERE id = ?")
        .bind(user_id)
        .execute(&pool)
        .await;
}
