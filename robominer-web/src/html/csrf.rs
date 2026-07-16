use super::format::escape_html;

/// Insert a CSRF meta tag and hidden inputs into every POST form.
pub(crate) fn inject_csrf_tokens(html: &str, token: &str) -> String {
    let escaped = escape_html(token);
    let field = format!(
        r#"<input type="hidden" name="{}" value="{escaped}"/>"#,
        crate::csrf::CSRF_FIELD_NAME
    );
    let meta = format!(r#"<meta name="csrf-token" content="{escaped}">"#);

    let with_meta = if html.contains(r#"name="csrf-token""#) {
        html.to_string()
    } else if let Some(idx) = html.find("</head>") {
        format!("{}{}{}", &html[..idx], meta, &html[idx..])
    } else {
        html.to_string()
    };

    inject_csrf_into_post_forms(&with_meta, &field)
}

fn inject_csrf_into_post_forms(html: &str, field: &str) -> String {
    let mut out = String::with_capacity(html.len().saturating_add(field.len().saturating_mul(4)));
    let mut rest = html;

    while let Some(form_start) = rest.find("<form") {
        out.push_str(&rest[..form_start]);
        let form_region = &rest[form_start..];
        let Some(tag_end) = form_region.find('>') else {
            out.push_str(form_region);
            return out;
        };
        let open_tag = &form_region[..=tag_end];
        out.push_str(open_tag);

        let is_post = open_tag.to_ascii_lowercase().contains("method=\"post\"");
        let after_tag = &form_region[tag_end + 1..];
        let already_present = after_tag
            .find("</form>")
            .is_some_and(|close| after_tag[..close].contains(crate::csrf::CSRF_FIELD_NAME));

        if is_post && !already_present {
            out.push_str(field);
        }

        rest = after_tag;
    }

    out.push_str(rest);
    out
}
