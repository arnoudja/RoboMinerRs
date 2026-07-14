use crate::account_page::AccountPageState;
use crate::html::{escape_html, layout};

pub(super) fn render_account_page(hud: Option<&str>, state: &AccountPageState) -> String {
    let mut body = String::from(r#"<div class="account-page">"#);
    body.push_str(r#"<div class="account-shell">"#);
    body.push_str(r#"<header class="account-header">"#);
    body.push_str(r#"<h1 class="account-page-title">Account</h1>"#);
    body.push_str(&format!(
        r#"<p class="account-page-subtitle">Signed in as {}</p>"#,
        escape_html(&state.current_username)
    ));
    body.push_str("</header>");
    body.push_str(r#"<div class="account-card auth-card">"#);
    body.push_str(r#"<form class="account-form" action="account" method="post">"#);
    if let Some(message) = &state.message {
        body.push_str(&format!(
            r#"<p class="auth-banner-success">{}</p>"#,
            escape_html(message)
        ));
    }
    if let Some(error_message) = &state.error_message {
        body.push_str(&format!(
            r#"<p class="auth-banner-error">{}</p>"#,
            escape_html(error_message)
        ));
    }
    body.push_str(r#"<h2 class="account-section-title">Profile</h2>"#);
    body.push_str(r#"<div class="auth-field">"#);
    body.push_str(r#"<label class="auth-label" for="username">Username</label>"#);
    body.push_str(&format!(
        r#"<input class="auth-input" type="text" id="username" name="username" pattern="[A-Za-z0-9]{{3,30}}" value="{}" required placeholder="Choose your in-game name" />"#,
        escape_html(&state.username),
    ));
    body.push_str(
        r#"<p class="auth-field-hint">3 to 30 characters, letters and numbers only.</p>"#,
    );
    body.push_str("</div>");
    body.push_str(r#"<div class="auth-field">"#);
    body.push_str(r#"<label class="auth-label" for="email">E-mail address</label>"#);
    body.push_str(&format!(
        r#"<input class="auth-input" type="email" id="email" name="email" value="{}" required placeholder="Enter your e-mail address" />"#,
        escape_html(&state.email),
    ));
    body.push_str("</div>");
    body.push_str(r#"<h2 class="account-section-title">Password</h2>"#);
    body.push_str(
        r#"<p class="account-section-hint">Leave new password blank to keep your current password.</p>"#,
    );
    render_account_password_field(
        &mut body,
        "currentpassword",
        "currentpassword",
        "Current password",
        "Your current password",
        r#" required"#,
        None,
    );
    render_account_password_field(
        &mut body,
        "newpassword",
        "newpassword",
        "New password",
        "New password, empty to leave unchanged",
        r#" pattern="^$|.{8,}""#,
        Some("At least 8 characters when changing password."),
    );
    render_account_password_field(
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
    render_account_scripts(&mut body);

    layout(
        "RoboMiner - Account",
        "account",
        &state.current_username,
        hud,
        &body,
    )
}

fn render_account_password_field(
    body: &mut String,
    field_id: &str,
    name: &str,
    label: &str,
    placeholder: &str,
    extra_attrs: &str,
    hint: Option<&str>,
) {
    body.push_str(r#"<div class="auth-field">"#);
    body.push_str(&format!(
        r#"<label class="auth-label" for="{field_id}">{label}</label>"#
    ));
    body.push_str(r#"<div class="auth-password-wrap">"#);
    body.push_str(&format!(
        r#"<input class="auth-input auth-password-input" type="password" id="{field_id}" name="{name}" placeholder="{placeholder}"{extra_attrs} />"#,
    ));
    body.push_str(&format!(
        r#"<button type="button" class="auth-password-toggle" data-target="{field_id}" aria-controls="{field_id}" aria-pressed="false">Show</button>"#,
    ));
    body.push_str("</div>");
    if let Some(hint) = hint {
        body.push_str(&format!(r#"<p class="auth-field-hint">{hint}</p>"#));
    }
    body.push_str("</div>");
}

fn render_account_scripts(body: &mut String) {
    body.push_str(
        r#"<script>
    function toggleAuthPasswordVisibility(button) {
        var fieldId = button.getAttribute('data-target');
        var input = document.getElementById(fieldId);
        if (!input) {
            return;
        }
        var showing = input.type === 'text';
        input.type = showing ? 'password' : 'text';
        button.textContent = showing ? 'Show' : 'Hide';
        button.setAttribute('aria-pressed', showing ? 'false' : 'true');
        button.setAttribute('aria-label', showing ? 'Show password' : 'Hide password');
    }

    var authPasswordToggles = document.querySelectorAll('.auth-password-toggle');
    for (var index = 0; index < authPasswordToggles.length; index += 1) {
        authPasswordToggles[index].addEventListener('click', function(event) {
            toggleAuthPasswordVisibility(event.currentTarget);
        });
    }
</script>"#,
    );
}
