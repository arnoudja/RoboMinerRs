use std::collections::HashMap;
use std::path::PathBuf;

use crate::html::{assert_contains_all, assert_html_contains, assert_html_not_contains};
use crate::{Request, ServerConfig};

use super::process::{
    auth_redirect_response, login_failure_message, remember_set_cookie_headers,
    signup_password_mismatch_message,
};
use super::render::render_login_page;
use super::{LoginPageState, login_page, logoff_page};

fn request(path: &str) -> Request {
    Request {
        method: "GET".to_string(),
        path: path.to_string(),
        query: HashMap::new(),
        form: HashMap::new(),
        form_values: HashMap::new(),
        headers: HashMap::new(),
    }
}

#[tokio::test(flavor = "current_thread")]
async fn get_logoff_page_does_not_expire_session_cookies() {
    let config = ServerConfig {
        static_root: PathBuf::from("robominer-web/static"),
        database_pool: None,
        allow_signup: false,
        trust_proxy: false,
    };
    let response = logoff_page(&request("/logoff"), &config).await;
    let cookie_headers: Vec<_> = response
        .headers
        .iter()
        .filter(|(name, _)| *name == "Set-Cookie")
        .map(|(_, value)| value.as_str())
        .collect();

    assert_eq!(response.status, 200);
    assert!(
        cookie_headers.is_empty(),
        "GET /logoff must not emit Set-Cookie clears"
    );

    let body = response.body_utf8();
    assert_contains_all(
        &body,
        &[
            r#"class="auth-page auth-logoff-page""#,
            r#"href="login">Log in again</a>"#,
        ],
    );
}

#[tokio::test(flavor = "current_thread")]
async fn post_logoff_without_session_or_csrf_does_not_clear_cookies() {
    let config = ServerConfig {
        static_root: PathBuf::from("robominer-web/static"),
        database_pool: None,
        allow_signup: false,
        trust_proxy: false,
    };
    let mut request = request("/logoff");
    request.method = "POST".to_string();
    let response = logoff_page(&request, &config).await;
    let cookie_headers: Vec<_> = response
        .headers
        .iter()
        .filter(|(name, _)| *name == "Set-Cookie")
        .map(|(_, value)| value.as_str())
        .collect();

    assert_eq!(response.status, 403);
    assert!(
        cookie_headers.is_empty(),
        "anonymous POST /logoff without CSRF must not emit Set-Cookie clears"
    );
}

#[tokio::test(flavor = "current_thread")]
async fn post_logoff_with_anonymous_csrf_clears_cookies() {
    let config = ServerConfig {
        static_root: PathBuf::from("robominer-web/static"),
        database_pool: None,
        allow_signup: false,
        trust_proxy: false,
    };
    let token = crate::csrf::new_anonymous_csrf_token();
    let mut request = request("/logoff");
    request.method = "POST".to_string();
    request.form.insert("csrfToken".to_string(), token.clone());
    request
        .headers
        .insert("cookie".to_string(), format!("robominer_csrf={token}"));
    let response = logoff_page(&request, &config).await;
    let cookie_headers: Vec<_> = response
        .headers
        .iter()
        .filter(|(name, _)| *name == "Set-Cookie")
        .map(|(_, value)| value.as_str())
        .collect();

    assert_eq!(response.status, 200);
    assert!(
        cookie_headers
            .iter()
            .any(|header| header.starts_with("robominer_session=; Max-Age=0;"))
    );
    assert!(
        cookie_headers
            .iter()
            .any(|header| header.starts_with("robominer_user_id=;"))
    );
    assert!(
        cookie_headers
            .iter()
            .any(|header| header.starts_with("robominer_username=;"))
    );
}

#[tokio::test(flavor = "current_thread")]
async fn login_requires_database_configuration() {
    let config = ServerConfig {
        static_root: PathBuf::from("robominer-web/static"),
        database_pool: None,
        allow_signup: true,
        trust_proxy: false,
    };

    let response = login_page(&request("/login"), &config).await;
    let body = response.body_utf8();

    assert_eq!(response.status, 503);
    assert_html_contains(&body, "ROBOMINER_DATABASE_URL");
}

#[test]
fn login_database_failure_does_not_leak_sql_details() {
    let error = crate::page_context::PageLoadError::from(sqlx::Error::Protocol(
        "SELECT * FROM secret_table WHERE leaked".into(),
    ));
    let response = super::login_database_error_response(error);
    let body = response.body_utf8();

    assert_eq!(response.status, 503);
    assert_html_contains(&body, "Unable to load login");
    assert_html_not_contains(&body, "secret_table");
    assert_html_not_contains(&body, "Unable to process login");
}

#[test]
fn login_rendering_preserves_forms_remembered_name_and_signup_errors() {
    let html = render_login_page(&LoginPageState {
        login_name: "user@example.com".to_string(),
        new_username: "New<User>".to_string(),
        email: "new&user@example.com".to_string(),
        error_message: Some("Signup <failed>".to_string()),
        show_signup: true,
        allow_signup: true,
        return_to: None,
    });

    assert_contains_all(
        &html,
        &[
            r#"class="auth-page""#,
            r#"name="viewport" content="width=device-width, initial-scale=1, viewport-fit=cover""#,
            r#"id="loginmenuitem" class="auth-tab""#,
            r#"id="signupmenuitem" class="auth-tab auth-tab-active""#,
            r#"id="loginForm" class="auth-form" action="login" method="post" hidden="hidden""#,
            r#"name="loginName" value="user@example.com""#,
            r#"name="remember" value="remember" checked"#,
            r#"id="signupForm" class="auth-form" action="login" method="post" data-pow-difficulty-bits="16">"#,
            r#"name="newusername" pattern="[A-Za-z0-9]{3,30}" value="New&lt;User&gt;""#,
            r#"name="email" value="new&amp;user@example.com""#,
            r#"<p class="auth-banner-error">Signup &lt;failed&gt;</p>"#,
            r#"class="auth-password-toggle""#,
            r#"src="js/common/password_toggle.js?v="#,
            // Signup PoW must be an external script: CSP is script-src 'self' (no 'unsafe-inline').
            r#"src="js/common/signup_pow.js?v="#,
        ],
    );
    assert_html_not_contains(&html, "Latest news");
    assert_html_not_contains(&html, "window.crypto.subtle");
}

#[test]
fn login_rendering_omits_signup_pow_script_when_signup_disabled() {
    let html = render_login_page(&LoginPageState {
        login_name: String::new(),
        new_username: String::new(),
        email: String::new(),
        error_message: None,
        show_signup: false,
        allow_signup: false,
        return_to: None,
    });

    assert_html_not_contains(&html, r#"src="js/common/signup_pow.js?v="#);
}

#[test]
fn login_rendering_shows_login_failure_banner() {
    let html = render_login_page(&LoginPageState {
        login_name: "user@example.com".to_string(),
        new_username: String::new(),
        email: String::new(),
        error_message: Some(login_failure_message().to_string()),
        show_signup: false,
        allow_signup: true,
        return_to: None,
    });

    assert_contains_all(
        &html,
        &[
            r#"<p class="auth-banner-error">Invalid login name or password.</p>"#,
            r#"name="loginName" value="user@example.com""#,
        ],
    );
}

#[test]
fn login_rendering_shows_login_form_by_default() {
    let html = render_login_page(&LoginPageState {
        login_name: String::new(),
        new_username: String::new(),
        email: String::new(),
        error_message: None,
        show_signup: false,
        allow_signup: true,
        return_to: None,
    });

    assert_contains_all(
        &html,
        &[
            r#"id="loginmenuitem" class="auth-tab auth-tab-active""#,
            r#"id="signupForm" class="auth-form" action="login" method="post" data-pow-difficulty-bits="16" hidden="hidden""#,
            r#"class="auth-tagline">Program robots. Mine ore. Compete in rallies.</p>"#,
        ],
    );
}

#[test]
fn login_rendering_preserves_return_to_in_form_and_links() {
    let html = render_login_page(&LoginPageState {
        login_name: String::new(),
        new_username: String::new(),
        email: String::new(),
        error_message: None,
        show_signup: false,
        allow_signup: true,
        return_to: Some("shop?selectedRobotPartTypeId=3".to_string()),
    });

    assert_contains_all(
        &html,
        &[
            r#"href="login?returnTo=shop%3FselectedRobotPartTypeId%3D3""#,
            r#"href="login?signup=1&returnTo=shop%3FselectedRobotPartTypeId%3D3""#,
            r#"<input type="hidden" name="returnTo" value="shop?selectedRobotPartTypeId=3" />"#,
        ],
    );
}

#[test]
fn login_rendering_hides_signup_when_disabled() {
    let html = render_login_page(&LoginPageState {
        login_name: String::new(),
        new_username: String::new(),
        email: String::new(),
        error_message: None,
        show_signup: false,
        allow_signup: false,
        return_to: None,
    });

    assert_contains_all(
        &html,
        &[
            r#"id="loginmenuitem" class="auth-tab auth-tab-active""#,
            r#"id="signupForm" class="auth-form" action="login" method="post" data-pow-difficulty-bits="16" hidden="hidden""#,
        ],
    );
    for absent in [r#"id="signupmenuitem""#, "Sign up</a> for free"] {
        assert_html_not_contains(&html, absent);
    }
}

#[test]
fn auth_redirect_sets_rust_auth_and_remember_cookies() {
    let response = auth_redirect_response(
        "miningQueue",
        42,
        0,
        "User Name",
        true,
        remember_set_cookie_headers("user@example.com", true),
    );
    let cookie_headers: Vec<_> = response
        .headers
        .iter()
        .filter(|(name, _)| *name == "Set-Cookie")
        .map(|(_, value)| value.as_str())
        .collect();

    assert_eq!(response.status, 302);
    assert!(
        response
            .headers
            .iter()
            .any(|(name, value)| *name == "Location" && value == "miningQueue")
    );
    assert!(cookie_headers.iter().any(|header| {
        header.starts_with("robominer_session=")
            && header.contains('.')
            && header.contains("Max-Age=2592000")
    }));
    assert!(cookie_headers.iter().any(
        |header| header.starts_with("robominer_username=User%20Name;")
            && header.contains("HttpOnly")
    ));
    assert!(cookie_headers.iter().any(|header| header.starts_with(
        "remember=user@example.com; Max-Age=2678400; Path=/; HttpOnly; SameSite=Lax"
    )));
}

#[test]
fn signup_password_mismatch_message_is_distinct_from_invalid_password() {
    assert_eq!(
        signup_password_mismatch_message(),
        "The passwords do not match."
    );
    assert_ne!(
        signup_password_mismatch_message(),
        robominer_domain::rejection_messages::create_user_rejection_player_message(
            robominer_db::CreateUserRejection::InvalidPassword
        )
    );
}

#[test]
fn signup_rejection_messages_match_legacy_copy() {
    assert_eq!(
        robominer_domain::rejection_messages::create_user_rejection_player_message(
            robominer_db::CreateUserRejection::DuplicateUsername
        ),
        "Could not create that account. Try a different username or e-mail, or log in if you already have one."
    );
    assert_eq!(
        robominer_domain::rejection_messages::create_user_rejection_player_message(
            robominer_db::CreateUserRejection::DuplicateEmail
        ),
        "Could not create that account. Try a different username or e-mail, or log in if you already have one."
    );
    assert_eq!(
        robominer_domain::rejection_messages::create_user_rejection_player_message(
            robominer_db::CreateUserRejection::InvalidPassword
        ),
        "The password doesn't meet the requirements"
    );
}
