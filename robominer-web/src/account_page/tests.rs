use std::collections::HashMap;
use std::path::PathBuf;

use crate::html::{assert_contains_all, assert_html_contains, assert_html_not_contains};
use crate::session::format_authenticated_cookie;
use crate::test_support::route;
use crate::{Request, ServerConfig};

use super::render::render_account_page;
use super::{AccountPageState, account_password_mismatch_message};

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

#[tokio::test(flavor = "current_thread")]
async fn account_requires_database_configuration() {
    let config = ServerConfig {
        static_root: PathBuf::from("robominer-web/static"),
        database_pool: None,
        allow_signup: true,
        trust_proxy: false,
    };

    let response = route(&authenticated_request("/account"), &config).await;
    let body = response.body_utf8();

    assert_eq!(response.status, 503);
    assert_html_contains(&body, "ROBOMINER_DATABASE_URL");
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

    assert_contains_all(
        &html,
        &[
            r#"class="account-page""#,
            r#"action="account" method="post""#,
            "Signed in as User &lt;Current&gt;",
            r#"name="username" pattern="[A-Za-z0-9]{3,30}" value="User &lt;Edit&gt;""#,
            r#"name="email" value="user&amp;edit@example.com""#,
            r#"name="currentpassword""#,
            r#"name="newpassword""#,
            r#"pattern="^$|.{12,}""#,
            r#"name="confirmpassword""#,
            r#"<p class="auth-banner-success">Updated &lt;ok&gt;</p>"#,
            r#"<p class="auth-banner-error">Error &lt;bad&gt;</p>"#,
            r#"<button type="submit" class="auth-submit">Save changes</button>"#,
            r#"id="logout-currentpassword""#,
            r#"name="logoutAllDevices" value="1""#,
            r#"Log out all devices"#,
            r#"class="auth-password-toggle""#,
            r#"src="js/common/password_toggle.js?v="#,
            // Account reuses auth form chrome; both sheets must load.
            r#"href="css/pages/auth.css?v="#,
            r#"href="css/pages/account.css?v="#,
        ],
    );
    assert_html_not_contains(&html, r#"<table>"#);
}

#[test]
fn account_password_mismatch_message_is_distinct_from_invalid_password() {
    assert_eq!(
        account_password_mismatch_message(),
        "The passwords do not match."
    );
    assert_ne!(
        account_password_mismatch_message(),
        robominer_domain::rejection_messages::update_user_account_rejection_player_message(
            robominer_db::UpdateUserAccountRejection::InvalidPassword
        )
    );
}

#[test]
fn account_update_rejection_messages_match_legacy_copy() {
    assert_eq!(
        robominer_domain::rejection_messages::update_user_account_rejection_player_message(
            robominer_db::UpdateUserAccountRejection::DuplicateUsername
        ),
        "Could not update that account. Try a different username or e-mail."
    );
    assert_eq!(
        robominer_domain::rejection_messages::update_user_account_rejection_player_message(
            robominer_db::UpdateUserAccountRejection::DuplicateEmail
        ),
        "Only one account per e-mail address is allowed"
    );
    assert_eq!(
        robominer_domain::rejection_messages::update_user_account_rejection_player_message(
            robominer_db::UpdateUserAccountRejection::InvalidPassword
        ),
        "Invalid password"
    );
}
