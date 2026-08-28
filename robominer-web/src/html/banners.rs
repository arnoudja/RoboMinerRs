use super::format::EscapedHtml;

/// Success/error banner used by shop, robot, edit-code, and achievements pages.
pub(crate) fn render_status_banner(body: &mut String, class_prefix: &str, message: Option<&str>) {
    let Some(message) = message else {
        return;
    };
    let banner_class = if message.starts_with("Unable") {
        format!("{class_prefix}-banner {class_prefix}-banner-error")
    } else {
        format!("{class_prefix}-banner {class_prefix}-banner-success")
    };
    body.push_str(&format!(
        r#"<p class="{banner_class}">{}</p>"#,
        EscapedHtml::from(message)
    ));
}
