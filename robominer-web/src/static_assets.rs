//! Helpers for linking static JS/CSS from `webroot` with content-hash cache busting.

use sha2::{Digest, Sha256};

/// First 16 bytes of SHA-256 as lowercase hex (32 chars).
/// Shared by `?v=` query busting and HTTP `ETag` so validators stay coherent.
pub(crate) fn content_hash_hex(bytes: &[u8]) -> String {
    let digest = Sha256::digest(bytes);
    let mut hex = String::with_capacity(32);
    for byte in digest.iter().take(16) {
        hex.push_str(&format!("{byte:02x}"));
    }
    hex
}

/// Emit `<script src="...">` with a content hash query for cache busting.
pub(crate) fn script_src_tag(src_path: &str, file_contents: &str) -> String {
    let hash = content_hash_hex(file_contents.as_bytes());
    format!(r#"<script src="{src_path}?v={hash}"></script>"#)
}

/// Emit `<link rel="stylesheet" …>` with a content hash query for cache busting.
pub(crate) fn stylesheet_href_tag(href_path: &str, file_contents: &str) -> String {
    let hash = content_hash_hex(file_contents.as_bytes());
    format!(r#"<link rel="stylesheet" type="text/css" href="{href_path}?v={hash}">"#)
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

const ROBOMINER_CSS: &str = include_str!("../static/css/robominer.css");

/// Canonical RoboMiner stylesheet link with cache-busting query.
pub(crate) fn robominer_stylesheet_tag() -> String {
    stylesheet_href_tag("css/robominer.css", ROBOMINER_CSS)
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

    #[test]
    fn stylesheet_and_script_hashes_match_content_hash_helper() {
        let css = "body{color:red}";
        let hash = content_hash_hex(css.as_bytes());
        assert_eq!(hash.len(), 32);
        assert!(stylesheet_href_tag("css/robominer.css", css).contains(&format!("?v={hash}")));
        assert!(script_src_tag("js/x.js", css).contains(&format!("?v={hash}")));
    }
}
