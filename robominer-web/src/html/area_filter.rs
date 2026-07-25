use super::format::escape_html;

pub(crate) struct AreaFilterOption<'a> {
    pub href: String,
    pub label: &'a str,
    pub selected: bool,
}

pub(crate) fn render_area_filter_select(
    body: &mut String,
    label_class: &str,
    select_id: &str,
    select_class: &str,
    aria_label: &str,
    options: &[AreaFilterOption<'_>],
) {
    if options.is_empty() {
        return;
    }

    body.push_str(&format!(
        r#"<label class="{label_class}" for="{select_id}">Area "#
    ));
    body.push_str(&format!(
        r#"<select id="{select_id}" class="{select_class}" aria-label="{aria_label}" onchange="window.location = this.value;">"#,
    ));
    for option in options {
        body.push_str(&format!(
            r#"<option value="{}"{}>{}</option>"#,
            escape_html(&option.href),
            if option.selected { " selected" } else { "" },
            escape_html(option.label),
        ));
    }
    body.push_str("</select></label>");
}
