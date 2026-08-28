use super::format::{EscapedHtml, html_attr};
use crate::static_assets::{AREA_FILTER_JS, script_src_tag};

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
        r#"<select id="{select_id}" class="{select_class}" aria-label="{aria_label}" data-area-filter-nav="true">"#,
    ));
    for option in options {
        body.push_str(&format!(
            r#"<option value="{}"{}>{}</option>"#,
            html_attr(&option.href),
            if option.selected { " selected" } else { "" },
            EscapedHtml::from(option.label),
        ));
    }
    body.push_str("</select></label>");
    body.push_str(&script_src_tag("js/common/area_filter.js", AREA_FILTER_JS));
}
