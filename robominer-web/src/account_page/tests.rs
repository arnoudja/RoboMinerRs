use std::collections::HashMap;
use std::path::PathBuf;

use crate::csrf::csrf_token_from_cookie;
use crate::rate_limit::{
    MAX_ATTEMPTS_PER_LOGIN, lock_auth_rate_limiter_for_tests, record_auth_attempt,
    reset_auth_rate_limiter_for_tests,
};
use crate::session::format_authenticated_cookie;
use crate::{Request, ServerConfig};

use super::render::render_account_page;
use super::{
    AccountPageState, account_page, account_password_mismatch_message,
    update_user_account_rejection_message,
};

fn authenticated_request(path: &str) -> Request {
    Request {
        method: "GET".to_string(),
        path: path.to_string(),
        query: HashMap::new(),
        form: HashMap::new(),
        form_values: HashMap::new(),
        headers: HashMap::from([(
            "cookie".to_string(),
            format_authenticated_cookie(42, "Player"),
        )]),
    }
}

fn authenticated_account_update_request() -> Request {
    let cookie = format_authenticated_cookie(42, "Player");
    let token = csrf_token_from_cookie(&cookie).expect("csrf token");
    let mut form = HashMap::new();
    form.insert("username".to_string(), "Player".to_string());
    form.insert("email".to_string(), "player@example.invalid".to_string());
    form.insert("currentpassword".to_string(), "wrong-password".to_string());
    form.insert("csrfToken".to_string(), token);
    Request {
        method: "POST".to_string(),
        path: "/account".to_string(),
        query: HashMap::new(),
        form: form.clone(),
        form_values: form
            .into_iter()
            .map(|(name, value)| (name, vec![value]))
            .collect(),
        headers: HashMap::from([
            ("cookie".to_string(), cookie),
            ("x-robominer-peer".to_string(), "198.51.100.70".to_string()),
        ]),
    }
}

#[tokio::test(flavor = "current_thread")]
async fn account_requires_database_configuration() {
    let config = ServerConfig {
        static_root: PathBuf::from("robominer-web/static"),
        database_pool: None,
        allow_signup: true,
        trust_proxy: false,
    };

    let response = account_page(&authenticated_request("/account"), &config).await;
    let body = String::from_utf8(response.body).expect("message should be utf-8");

    assert_eq!(response.status, 503);
    assert!(body.contains("ROBOMINER_DATABASE_URL"));
}

#[tokio::test(flavor = "current_thread")]
async fn account_update_is_rate_limited_before_database_work() {
    let _guard = lock_auth_rate_limiter_for_tests();
    reset_auth_rate_limiter_for_tests();
    let config = ServerConfig {
        static_root: PathBuf::from("robominer-web/static"),
        database_pool: None,
        allow_signup: true,
        trust_proxy: false,
    };

    for index in 0..MAX_ATTEMPTS_PER_LOGIN {
        let ip = format!("198.51.100.{index}");
        record_auth_attempt(&ip, "user:42");
    }

    let response = account_page(&authenticated_account_update_request(), &config).await;
    let body = String::from_utf8(response.body).expect("message should be utf-8");
    assert_eq!(response.status, 429);
    assert!(body.contains("Too many account password checks"));
}

#[test]
fn account_rendering_preserves_form_contract_and_escapes_fields() {
    let html = render_account_page(
        None,
        &AccountPageState {
            username: "User <Edit>".to_string(),
            email: "user&edit@example.com".to_string(),
            current_username: "User <Current>".to_string(),
            message: Some("Updated <ok>".to_string()),
            error_message: Some("Error <bad>".to_string()),
            reissue_session_version: None,
        },
    );

    assert!(html.contains(r#"class="account-page""#));
    assert!(html.contains(r#"action="account" method="post""#));
    assert!(html.contains("Signed in as User &lt;Current&gt;"));
    assert!(
        html.contains(r#"name="username" pattern="[A-Za-z0-9]{3,30}" value="User &lt;Edit&gt;""#)
    );
    assert!(html.contains(r#"name="email" value="user&amp;edit@example.com""#));
    assert!(html.contains(r#"name="currentpassword""#));
    assert!(html.contains(r#"name="newpassword""#));
    assert!(html.contains(r#"pattern="^$|.{8,}""#));
    assert!(html.contains(r#"name="confirmpassword""#));
    assert!(html.contains(r#"name="confirmpassword""#));
    assert!(html.contains(r#"<p class="auth-banner-success">Updated &lt;ok&gt;</p>"#));
    assert!(html.contains(r#"<p class="auth-banner-error">Error &lt;bad&gt;</p>"#));
    assert!(html.contains(r#"<button type="submit" class="auth-submit">Save changes</button>"#));
    assert!(html.contains(r#"class="auth-password-toggle""#));
    assert!(html.contains("toggleAuthPasswordVisibility"));
    assert!(!html.contains(r#"<table>"#));
}

#[test]
fn account_password_mismatch_message_is_distinct_from_invalid_password() {
    assert_eq!(
        account_password_mismatch_message(),
        "The passwords do not match."
    );
    assert_ne!(
        account_password_mismatch_message(),
        update_user_account_rejection_message(
            robominer_db::UpdateUserAccountRejection::InvalidPassword
        )
    );
}

#[test]
fn account_update_rejection_messages_match_legacy_copy() {
    assert_eq!(
        update_user_account_rejection_message(
            robominer_db::UpdateUserAccountRejection::DuplicateUsername
        ),
        "That username is already taken"
    );
    assert_eq!(
        update_user_account_rejection_message(
            robominer_db::UpdateUserAccountRejection::DuplicateEmail
        ),
        "Only one account per e-mail address is allowed"
    );
    assert_eq!(
        update_user_account_rejection_message(
            robominer_db::UpdateUserAccountRejection::InvalidPassword
        ),
        "Invalid password"
    );
}
