use crate::auth_pages::LoginPageState;
use crate::html::{escape_html, page_footer};
use crate::request_helpers::auth_page_href;

pub(super) fn render_login_page(state: &LoginPageState) -> String {
    let mut body = String::from(r#"<div class="auth-page">"#);
    render_auth_header(
        &mut body,
        state.show_signup,
        state.allow_signup,
        state.return_to.as_deref(),
    );
    body.push_str(r#"<div class="auth-shell">"#);
    body.push_str(r#"<div class="auth-card">"#);
    render_login_form(&mut body, state);
    render_signup_form(&mut body, state);
    body.push_str("</div>");
    body.push_str(r#"<p class="auth-tagline">Program robots. Mine ore. Compete in rallies.</p>"#);
    body.push_str("</div></div>");
    render_auth_scripts(&mut body);

    format!(
        r##"<!DOCTYPE html>
<html>
    <head>
        <meta http-equiv="Content-Type" content="text/html; charset=UTF-8">
        <link rel="stylesheet" type="text/css" href="css/robominer.css">
        <title>RoboMiner - Login</title>
    </head>
    <body>
        <div class="main">
            <div class="interface">
                {body}
            </div>
            {footer}
        </div>
    </body>
</html>"##,
        footer = page_footer()
    )
}

fn render_auth_header(
    body: &mut String,
    show_signup: bool,
    allow_signup: bool,
    return_to: Option<&str>,
) {
    body.push_str(r#"<header class="auth-header">"#);
    body.push_str(r#"<p class="auth-brand">RoboMiner</p>"#);
    body.push_str(r#"<nav class="auth-tabs" aria-label="Authentication">"#);
    body.push_str(&format!(
        r#"<a id="loginmenuitem" class="auth-tab{}" href="{}">Login</a>"#,
        if show_signup { "" } else { " auth-tab-active" },
        auth_page_href(false, return_to),
    ));
    if allow_signup {
        body.push_str(&format!(
            r#"<a id="signupmenuitem" class="auth-tab{}" href="{}">Sign up</a>"#,
            if show_signup { " auth-tab-active" } else { "" },
            auth_page_href(true, return_to),
        ));
    }
    body.push_str("</nav></header>");
}

pub(super) fn render_logoff_body() -> String {
    r#"<div class="auth-page auth-logoff-page">
<div class="auth-shell">
<div class="auth-card auth-logoff-card">
<h1 class="auth-form-title">Logged off</h1>
<p class="auth-form-subtitle">Your session has ended. Log in again when you are ready to return to the command deck.</p>
<a class="auth-submit auth-logoff-link" href="login">Log in again</a>
</div>
</div>
</div>"#
        .to_string()
}

fn render_auth_password_field(
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
        r#"<input class="auth-input auth-password-input" type="password" id="{field_id}" name="{name}" required placeholder="{placeholder}"{extra_attrs} />"#,
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

fn render_auth_scripts(body: &mut String) {
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

fn render_login_form(body: &mut String, state: &LoginPageState) {
    let hidden = if state.show_signup {
        r#" hidden="hidden""#
    } else {
        ""
    };
    let login_autofocus = if state.login_name.is_empty() {
        r#" autofocus="autofocus""#
    } else {
        ""
    };
    let password_autofocus = if state.login_name.is_empty() {
        ""
    } else {
        r#" autofocus="autofocus""#
    };
    let remember_checked = if state.login_name.is_empty() {
        ""
    } else {
        " checked"
    };

    body.push_str(&format!(
        r#"<form id="loginForm" class="auth-form" action="Login" method="post"{hidden}>"#,
    ));
    body.push_str(r#"<h1 class="auth-form-title">Welcome back</h1>"#);
    body.push_str(
        r#"<p class="auth-form-subtitle">Log in with your username or e-mail address.</p>"#,
    );
    if let Some(message) = &state.error_message
        && !state.show_signup
    {
        body.push_str(&format!(
            r#"<p class="auth-banner-error">{}</p>"#,
            escape_html(message)
        ));
    }
    body.push_str(r#"<div class="auth-field">"#);
    body.push_str(r#"<label class="auth-label" for="loginName">Login name</label>"#);
    body.push_str(&format!(
        r#"<input class="auth-input" type="text" id="loginName" name="loginName" value="{}" required placeholder="Username or e-mail address"{login_autofocus} />"#,
        escape_html(&state.login_name),
    ));
    body.push_str("</div>");
    render_auth_password_field(
        body,
        "password",
        "password",
        "Password",
        "Your password",
        password_autofocus,
        None,
    );
    body.push_str(
        r#"<label class="auth-remember" title="Keeps you signed in on this device for 30 days and saves your login name.">"#,
    );
    body.push_str(&format!(
        r#"<input type="checkbox" name="remember" value="remember"{remember_checked} />Keep me signed in on this device"#,
    ));
    body.push_str("</label>");
    body.push_str(r#"<button type="submit" class="auth-submit">Log in</button>"#);
    if let Some(return_to) = &state.return_to {
        body.push_str(&format!(
            r#"<input type="hidden" name="returnTo" value="{}" />"#,
            escape_html(return_to)
        ));
    }
    if state.allow_signup {
        body.push_str(&format!(
            r#"<p class="auth-switch">No account yet? <a class="auth-switch-link" href="{}">Sign up</a> for free.</p>"#,
            auth_page_href(true, state.return_to.as_deref()),
        ));
    }
    body.push_str("</form>");
}

fn render_signup_form(body: &mut String, state: &LoginPageState) {
    let hidden = if state.show_signup {
        ""
    } else {
        r#" hidden="hidden""#
    };

    body.push_str(&format!(
        r#"<form id="signupForm" class="auth-form" action="Login" method="post"{hidden}>"#,
    ));
    body.push_str(r#"<h1 class="auth-form-title">Create account</h1>"#);
    body.push_str(
        r#"<p class="auth-form-subtitle">Choose a username and password to join RoboMiner.</p>"#,
    );
    if let Some(message) = &state.error_message {
        body.push_str(&format!(
            r#"<p class="auth-banner-error">{}</p>"#,
            escape_html(message)
        ));
    }
    body.push_str(r#"<div class="auth-field">"#);
    body.push_str(r#"<label class="auth-label" for="newusername">Username</label>"#);
    body.push_str(&format!(
        r#"<input class="auth-input" type="text" id="newusername" name="newusername" pattern="[A-Za-z0-9]{{3,30}}" value="{}" required placeholder="Choose your in-game name" autofocus="autofocus" />"#,
        escape_html(&state.new_username),
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
    render_auth_password_field(
        body,
        "newpassword",
        "newpassword",
        "Password",
        "Choose a password",
        r#" pattern=".{8,}""#,
        Some("At least 8 characters."),
    );
    render_auth_password_field(
        body,
        "confirmpassword",
        "confirmpassword",
        "Confirm password",
        "Confirm your password",
        "",
        None,
    );
    body.push_str(r#"<button type="submit" class="auth-submit">Sign up</button>"#);
    body.push_str(&format!(
        r#"<p class="auth-switch">Already have an account? <a class="auth-switch-link" href="{}">Log in</a>.</p>"#,
        auth_page_href(false, state.return_to.as_deref()),
    ));
    body.push_str("</form>");
}
