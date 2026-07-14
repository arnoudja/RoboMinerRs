use std::collections::HashMap;
use std::path::PathBuf;

use crate::{Request, ServerConfig};

use super::render::render_login_page;
use super::{
    LoginPageState, auth_redirect_response, create_user_rejection_message, login_failure_message,
    login_page, logoff_page, remember_cookie, signup_password_mismatch_message,
};

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

#[test]
fn logoff_route_expires_session_cookies() {
    let response = logoff_page();
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
    assert!(
        cookie_headers
            .iter()
            .any(|header| header.starts_with("JSESSIONID=;"))
    );

    let body = String::from_utf8(response.body).expect("body should be utf-8");
    assert!(body.contains(r#"class="auth-page auth-logoff-page""#));
    assert!(body.contains(r#"href="login">Log in again</a>"#));
}

#[test]
fn login_requires_database_configuration() {
    let config = ServerConfig {
        static_root: PathBuf::from("robominer-web/static"),
        database_pool: None,
        allow_signup: true,
    };

    let response = login_page(&request("/login"), &config);
    let body = String::from_utf8(response.body).expect("message should be utf-8");

    assert_eq!(response.status, 503);
    assert!(body.contains("ROBOMINER_DATABASE_URL"));
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

    assert!(html.contains(r#"class="auth-page""#));
    assert!(html.contains(r#"id="loginmenuitem" class="auth-tab""#));
    assert!(html.contains(r#"id="signupmenuitem" class="auth-tab auth-tab-active""#));
    assert!(html.contains(
        r#"id="loginForm" class="auth-form" action="Login" method="post" hidden="hidden""#
    ));
    assert!(html.contains(r#"name="loginName" value="user@example.com""#));
    assert!(html.contains(r#"name="remember" value="remember" checked"#));
    assert!(html.contains(r#"id="signupForm" class="auth-form" action="Login" method="post">"#));
    assert!(
        html.contains(r#"name="newusername" pattern="[A-Za-z0-9]{3,30}" value="New&lt;User&gt;""#)
    );
    assert!(html.contains(r#"name="email" value="new&amp;user@example.com""#));
    assert!(html.contains(r#"<p class="auth-banner-error">Signup &lt;failed&gt;</p>"#));
    assert!(html.contains(r#"class="auth-password-toggle""#));
    assert!(html.contains("toggleAuthPasswordVisibility"));
    assert!(!html.contains("Latest news"));
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

    assert!(html.contains(r#"<p class="auth-banner-error">Invalid login name or password.</p>"#));
    assert!(html.contains(r#"name="loginName" value="user@example.com""#));
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

    assert!(html.contains(r#"id="loginmenuitem" class="auth-tab auth-tab-active""#));
    assert!(html.contains(
        r#"id="signupForm" class="auth-form" action="Login" method="post" hidden="hidden""#
    ));
    assert!(
        html.contains(r#"class="auth-tagline">Program robots. Mine ore. Compete in rallies.</p>"#)
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

    assert!(html.contains(r#"href="login?returnTo=shop%3FselectedRobotPartTypeId%3D3""#));
    assert!(html.contains(r#"href="login?signup=1&returnTo=shop%3FselectedRobotPartTypeId%3D3""#));
    assert!(html.contains(
        r#"<input type="hidden" name="returnTo" value="shop?selectedRobotPartTypeId=3" />"#
    ));
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

    assert!(html.contains(r#"id="loginmenuitem" class="auth-tab auth-tab-active""#));
    assert!(!html.contains(r#"id="signupmenuitem""#));
    assert!(!html.contains("Sign up</a> for free"));
    assert!(html.contains(
        r#"id="signupForm" class="auth-form" action="Login" method="post" hidden="hidden""#
    ));
}

#[test]
fn auth_redirect_sets_rust_auth_and_remember_cookies() {
    let response = auth_redirect_response(
        "miningQueue",
        42,
        "User Name",
        true,
        remember_cookie("user@example.com", true),
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
    assert!(
        cookie_headers
            .iter()
            .any(|header| header.starts_with("robominer_username=User%20Name;"))
    );
    assert!(
        cookie_headers
            .iter()
            .any(|header| header.starts_with("remember=user@example.com; Max-Age=2678400;"))
    );
}

#[test]
fn signup_password_mismatch_message_is_distinct_from_invalid_password() {
    assert_eq!(
        signup_password_mismatch_message(),
        "The passwords do not match."
    );
    assert_ne!(
        signup_password_mismatch_message(),
        create_user_rejection_message(robominer_db::CreateUserRejection::InvalidPassword)
    );
}

#[test]
fn signup_rejection_messages_match_legacy_copy() {
    assert_eq!(
        create_user_rejection_message(robominer_db::CreateUserRejection::DuplicateUsername),
        "Username already taken, please choose another one"
    );
    assert_eq!(
        create_user_rejection_message(robominer_db::CreateUserRejection::DuplicateEmail),
        "You already have an account, please login using your e-mail address"
    );
    assert_eq!(
        create_user_rejection_message(robominer_db::CreateUserRejection::InvalidPassword),
        "The password doesn't meet the requirements"
    );
}
