//! Helpers for linking static JS/CSS from `webroot` with content-hash cache busting.

use sha2::{Digest, Sha256};

/// Emit `<script src="...">` with a short content hash query for cache busting.
pub(crate) fn script_src_tag(src_path: &str, file_contents: &str) -> String {
    let hash = short_content_hash(file_contents);
    format!(r#"<script src="{src_path}?v={hash}"></script>"#)
}

/// Concatenate several script tags (order preserved).
pub(crate) fn script_src_tags(entries: &[(&str, &str)]) -> String {
    let mut out = String::new();
    for (src_path, contents) in entries {
        out.push_str(&script_src_tag(src_path, contents));
        out.push('\n');
    }
    out
}

fn short_content_hash(contents: &str) -> String {
    let digest = Sha256::digest(contents.as_bytes());
    let mut hex = String::with_capacity(16);
    for byte in digest.iter().take(8) {
        hex.push_str(&format!("{byte:02x}"));
    }
    hex
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn script_src_tag_includes_stable_hash_query() {
        let tag = script_src_tag("js/shop/page.js", "console.log(1);");
        assert!(tag.starts_with(r#"<script src="js/shop/page.js?v="#));
        assert!(tag.ends_with(r#""></script>"#));
        let again = script_src_tag("js/shop/page.js", "console.log(1);");
        assert_eq!(tag, again);
        let changed = script_src_tag("js/shop/page.js", "console.log(2);");
        assert_ne!(tag, changed);
    }
}
