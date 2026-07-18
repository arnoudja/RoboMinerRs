mod support;

use std::collections::HashMap;

use robominer_web::test_support::route;
use serial_test::serial;
use support::{
    cookie_header, create_user_via_engine, ensure_session_configured, get_request,
    login_with_credentials, post_request, response_body, server_config, unique_prefix,
};

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
#[serial]
async fn account_update_post_persists_profile_changes() {
    if std::env::var("ROBOMINER_DATABASE_URL").is_err() {
        eprintln!("skipping account web test: ROBOMINER_DATABASE_URL is not set");
        return;
    }

    ensure_session_configured();

    let pool =
        robominer_db::connect(&std::env::var("ROBOMINER_DATABASE_URL").expect("database url"))
            .await
            .expect("failed to connect to test database");
    let prefix = unique_prefix("rust-web-account");
    let username = format!("{prefix}-user");
    let password = "test-password-1".to_string();
    let user_id =
        create_user_via_engine(&username, &format!("{prefix}@example.invalid"), &password);
    let config = server_config(pool.clone());
    let cookie = cookie_header(&login_with_credentials(&config, &username, &password).await);

    let updated_username = format!("{prefix}-renamed");
    let updated_email = format!("{prefix}-renamed@example.invalid");
    let mut form = HashMap::new();
    form.insert("username".to_string(), updated_username.clone());
    form.insert("email".to_string(), updated_email.clone());
    form.insert("currentpassword".to_string(), password.clone());
    form.insert("newpassword".to_string(), String::new());
    form.insert("confirmpassword".to_string(), String::new());

    let response = route(&post_request("/account", form, Some(&cookie)), &config).await;
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

    let stored: (String, String) = sqlx::query_as("SELECT username, email FROM User WHERE id = ?")
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

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
#[serial]
async fn account_password_change_persists_and_invalidates_other_sessions() {
    if std::env::var("ROBOMINER_DATABASE_URL").is_err() {
        eprintln!("skipping account web test: ROBOMINER_DATABASE_URL is not set");
        return;
    }

    ensure_session_configured();

    let pool =
        robominer_db::connect(&std::env::var("ROBOMINER_DATABASE_URL").expect("database url"))
            .await
            .expect("failed to connect to test database");
    let prefix = unique_prefix("rust-web-account-pw");
    let username = format!("{prefix}-user");
    let password = "test-password-1".to_string();
    let new_password = "test-password-2".to_string();
    let user_id =
        create_user_via_engine(&username, &format!("{prefix}@example.invalid"), &password);
    let config = server_config(pool.clone());

    let hash_before: String = sqlx::query_scalar("SELECT password FROM User WHERE id = ?")
        .bind(user_id)
        .fetch_one(&pool)
        .await
        .expect("password hash before change");

    let old_cookie = cookie_header(&login_with_credentials(&config, &username, &password).await);
    let changing_cookie =
        cookie_header(&login_with_credentials(&config, &username, &password).await);

    let mut form = HashMap::new();
    form.insert("username".to_string(), username.clone());
    form.insert("email".to_string(), format!("{prefix}@example.invalid"));
    form.insert("currentpassword".to_string(), password.clone());
    form.insert("newpassword".to_string(), new_password.clone());
    form.insert("confirmpassword".to_string(), new_password.clone());

    let change_response =
        route(&post_request("/account", form, Some(&changing_cookie)), &config).await;
    let change_body = response_body(&change_response);
    assert_eq!(change_response.status, 200);
    assert!(
        change_body.contains("Account information updated"),
        "expected password change success:\n{change_body}"
    );

    let hash_after: String = sqlx::query_scalar("SELECT password FROM User WHERE id = ?")
        .bind(user_id)
        .fetch_one(&pool)
        .await
        .expect("password hash after change");
    assert!(
        hash_after.starts_with("$argon2"),
        "changed password should be stored as Argon2"
    );
    assert_ne!(
        hash_before, hash_after,
        "password change should rewrite the stored hash"
    );

    let fresh_cookie = cookie_header(&change_response);
    assert!(
        fresh_cookie.contains("robominer_session="),
        "password change should re-issue session cookie"
    );

    let session_version: i32 = sqlx::query_scalar("SELECT sessionVersion FROM User WHERE id = ?")
        .bind(user_id)
        .fetch_one(&pool)
        .await
        .expect("session version");
    assert_eq!(session_version, 1);

    let stale_response = route(&get_request("/account", Some(&old_cookie)), &config).await;
    assert_eq!(
        stale_response.status, 302,
        "old session should be rejected after password change"
    );
    assert!(
        stale_response
            .headers
            .iter()
            .any(|(name, value)| *name == "Location" && value.starts_with("login")),
        "stale session should redirect to login"
    );
    assert!(
        stale_response.headers.iter().any(|(name, value)| *name
            == "Set-Cookie"
            && value.starts_with("robominer_session=; Max-Age=0;")),
        "stale session should clear the session cookie"
    );

    let active_response = route(&get_request("/account", Some(&fresh_cookie)), &config).await;
    assert_eq!(
        active_response.status, 200,
        "re-issued session after password change should still work"
    );

    let old_login = login_with_credentials(&config, &username, &password).await;
    let old_login_body = response_body(&old_login);
    assert_eq!(old_login.status, 200, "old password should no longer log in");
    assert!(
        old_login_body.contains("Invalid login name or password"),
        "expected login failure for old password:\n{old_login_body}"
    );

    let new_login = login_with_credentials(&config, &username, &new_password).await;
    assert_eq!(
        new_login.status, 302,
        "new password should log in successfully"
    );
    assert!(
        cookie_header(&new_login).contains("robominer_session="),
        "successful login should set a session cookie"
    );

    let _ = sqlx::query("DELETE FROM Robot WHERE userId = ?")
        .bind(user_id)
        .execute(&pool)
        .await;
    let _ = sqlx::query("DELETE FROM User WHERE id = ?")
        .bind(user_id)
        .execute(&pool)
        .await;
}
