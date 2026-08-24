/// Escape JSON for safe embedding in `<script type="application/json">` blocks.
pub(crate) fn escape_embedded_json(json: &str) -> String {
    json.replace('<', "\\u003c")
}

/// Render a JSON bootstrap block with `<` escaped to prevent script breakout.
pub(crate) fn embed_json_script(id: &str, json: &str) -> String {
    format!(
        r#"<script type="application/json" id="{id}">{json}</script>"#,
        json = escape_embedded_json(json)
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn escape_embedded_json_breaks_out_script_tags() {
        assert_eq!(
            escape_embedded_json(r#"{"x":"</script>"}"#),
            r#"{"x":"\u003c/script>"}"#
        );
    }
}
