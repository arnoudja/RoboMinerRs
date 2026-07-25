use crate::static_assets::script_src_tag;

const PASSWORD_TOGGLE_JS: &str = include_str!("../../static/js/common/password_toggle.js");

pub(crate) fn render_password_field(
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

pub(crate) fn render_password_toggle_script(body: &mut String) {
    body.push_str(&script_src_tag(
        "js/common/password_toggle.js",
        PASSWORD_TOGGLE_JS,
    ));
}
