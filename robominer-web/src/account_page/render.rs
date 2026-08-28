use crate::account_page::AccountPageState;
use crate::html::{
    EscapedHtml, html_attr, layout, render_password_field, render_password_toggle_script,
};
use crate::static_assets::PageStylesheet;

pub(super) fn render_account_page(hud: Option<&str>, state: &AccountPageState) -> String {
    let mut body = String::from(r#"<div class="account-page">"#);
    body.push_str(r#"<div class="account-shell">"#);
    body.push_str(r#"<header class="account-header">"#);
    body.push_str(r#"<h1 class="account-page-title">Account</h1>"#);
    body.push_str(&format!(
        r#"<p class="account-page-subtitle">Signed in as {}</p>"#,
        EscapedHtml::from(state.current_username.as_str())
    ));
    body.push_str("</header>");
    body.push_str(r#"<div class="account-card auth-card">"#);
    body.push_str(r#"<form class="account-form" action="account" method="post">"#);
    if let Some(message) = &state.message {
        body.push_str(&format!(
            r#"<p class="auth-banner-success">{}</p>"#,
            EscapedHtml::from(message.as_str())
        ));
    }
    if let Some(error_message) = &state.error_message {
        body.push_str(&format!(
            r#"<p class="auth-banner-error">{}</p>"#,
            EscapedHtml::from(error_message.as_str())
        ));
    }
    body.push_str(r#"<h2 class="account-section-title">Profile</h2>"#);
    body.push_str(r#"<div class="auth-field">"#);
    body.push_str(r#"<label class="auth-label" for="username">Username</label>"#);
    body.push_str(&format!(
        r#"<input class="auth-input" type="text" id="username" name="username" pattern="[A-Za-z0-9]{{3,30}}" value="{}" required placeholder="Choose your in-game name" />"#,
        html_attr(&state.username),
    ));
    body.push_str(
        r#"<p class="auth-field-hint">3 to 30 characters, letters and numbers only.</p>"#,
    );
    body.push_str("</div>");
    body.push_str(r#"<div class="auth-field">"#);
    body.push_str(r#"<label class="auth-label" for="email">E-mail address</label>"#);
    body.push_str(&format!(
        r#"<input class="auth-input" type="email" id="email" name="email" value="{}" required placeholder="Enter your e-mail address" />"#,
        html_attr(&state.email),
    ));
    body.push_str("</div>");
    body.push_str(r#"<h2 class="account-section-title">Password</h2>"#);
    body.push_str(
        r#"<p class="account-section-hint">Leave new password blank to keep your current password.</p>"#,
    );
    render_password_field(
        &mut body,
        "currentpassword",
        "currentpassword",
        "Current password",
        "Your current password",
        r#" required"#,
        None,
    );
    render_password_field(
        &mut body,
        "newpassword",
        "newpassword",
        "New password",
        "New password, empty to leave unchanged",
        r#" pattern="^$|.{8,}""#,
        Some("At least 8 characters when changing password."),
    );
    render_password_field(
        &mut body,
        "confirmpassword",
        "confirmpassword",
        "Confirm password",
        "Confirm your new password",
        "",
        None,
    );
    body.push_str(r#"<button type="submit" class="auth-submit">Save changes</button>"#);
    body.push_str("</form></div></div></div>");
    render_password_toggle_script(&mut body);

    layout(
        "RoboMiner - Account",
        "account",
        &state.current_username,
        hud,
        &body,
        // Auth form/control classes live in auth.css; account.css adds page chrome.
        &[PageStylesheet::Auth, PageStylesheet::Account],
    )
}
