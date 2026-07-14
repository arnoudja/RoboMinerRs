mod support;

use robominer_web::test_support::route;
use serial_test::serial;
use support::{
    cookie_header, create_user_via_engine, ensure_session_configured, login_with_credentials,
    post_request, server_config, unique_prefix,
};

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
#[serial]
async fn login_post_redirects_to_mining_queue_with_session_cookie() {
    if std::env::var("ROBOMINER_DATABASE_URL").is_err() {
        eprintln!("skipping login web test: ROBOMINER_DATABASE_URL is not set");
        return;
    }

    ensure_session_configured();

    let pool =
        robominer_db::connect(&std::env::var("ROBOMINER_DATABASE_URL").expect("database url"))
            .await
            .expect("failed to connect to test database");
    let prefix = unique_prefix("rust-web-login");
    let username = format!("{prefix}-user");
    let password = "test-password-1".to_string();
    let user_id =
        create_user_via_engine(&username, &format!("{prefix}@example.invalid"), &password);
    let config = server_config(pool.clone());

    let login_response = login_with_credentials(&config, &username, &password);

    assert_eq!(
        login_response.status, 302,
        "login should redirect after success"
    );
    assert!(
        login_response
            .headers
            .iter()
            .any(|(name, value)| *name == "Location" && value == "miningQueue"),
        "login should redirect to mining queue"
    );
    let cookie = cookie_header(&login_response);
    assert!(
        cookie.contains("robominer_session="),
        "login should mint a session cookie"
    );

    let queue_response = route(
        &support::get_request("/miningQueue", Some(&cookie)),
        &config,
    );
    assert_eq!(
        queue_response.status, 200,
        "authenticated queue page should render"
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

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
#[serial]
async fn signup_post_redirects_to_welcome_with_session_cookie() {
    if std::env::var("ROBOMINER_DATABASE_URL").is_err() {
        eprintln!("skipping signup web test: ROBOMINER_DATABASE_URL is not set");
        return;
    }

    ensure_session_configured();

    let pool =
        robominer_db::connect(&std::env::var("ROBOMINER_DATABASE_URL").expect("database url"))
            .await
            .expect("failed to connect to test database");
    let prefix = unique_prefix("rust-web-signup");
    let username = format!("{prefix}user");
    let email = format!("{prefix}@example.invalid");
    let password = "test-password-1".to_string();
    let config = server_config(pool.clone());

    let mut form = std::collections::HashMap::new();
    form.insert("newusername".to_string(), username.clone());
    form.insert("email".to_string(), email.clone());
    form.insert("newpassword".to_string(), password.clone());
    form.insert("confirmpassword".to_string(), password.clone());

    let response = route(&post_request("/login", form, None), &config);

    assert_eq!(response.status, 302, "signup should redirect after success");
    assert!(
        response
            .headers
            .iter()
            .any(|(name, value)| *name == "Location" && value.starts_with("help?welcome=1")),
        "signup should redirect to welcome help page"
    );
    let cookie = cookie_header(&response);
    assert!(
        cookie.contains("robominer_session="),
        "signup should mint a session cookie"
    );

    let user_id: i64 = sqlx::query_scalar("SELECT id FROM User WHERE username = ?")
        .bind(&username)
        .fetch_one(&pool)
        .await
        .expect("failed to load created user id");

    let _ = sqlx::query("DELETE FROM Robot WHERE userId = ?")
        .bind(user_id)
        .execute(&pool)
        .await;
    let _ = sqlx::query("DELETE FROM UserAchievement WHERE userId = ?")
        .bind(user_id)
        .execute(&pool)
        .await;
    let _ = sqlx::query("DELETE FROM User WHERE id = ?")
        .bind(user_id)
        .execute(&pool)
        .await;
}
