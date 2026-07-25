//! Helpers for linking static JS/CSS from `webroot` with content-hash cache busting.

use sha2::{Digest, Sha256};

pub(crate) const URL_QUERY_JS: &str = include_str!("../static/js/common/url_query.js");
pub(crate) const SESSION_STORE_JS: &str = include_str!("../static/js/common/session_store.js");
pub(crate) const PANEL_STATE_JS: &str = include_str!("../static/js/common/panel_state.js");
pub(crate) const AREA_FILTER_JS: &str = include_str!("../static/js/common/area_filter.js");

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

/// Page scripts that need the shared URL query helper, then page logic.
pub(crate) fn page_scripts_with_url_query(page_src: &str, page_js: &str) -> String {
    script_src_tags(&[
        ("js/common/url_query.js", URL_QUERY_JS),
        (page_src, page_js),
    ])
}

/// Page scripts that need URL query + session store helpers, then page logic.
pub(crate) fn page_scripts_with_url_query_and_session(page_src: &str, page_js: &str) -> String {
    script_src_tags(&[
        ("js/common/url_query.js", URL_QUERY_JS),
        ("js/common/session_store.js", SESSION_STORE_JS),
        (page_src, page_js),
    ])
}

/// Page scripts that need panel state + URL query helpers, then page logic.
pub(crate) fn page_scripts_with_panel_and_url_query(page_src: &str, page_js: &str) -> String {
    script_src_tags(&[
        ("js/common/panel_state.js", PANEL_STATE_JS),
        ("js/common/url_query.js", URL_QUERY_JS),
        (page_src, page_js),
    ])
}

// `robominer.css` was split into page-scoped files under `static/css/pages/`
// (see git history for the original monolith). Each is served as its own
// `/css/pages/<name>.css` static file and content-hashed independently, so
// changing one page's styles doesn't bust the cache for every other page.
// Order matches the original section order in the monolithic stylesheet.
const LAYOUT_CSS: &str = include_str!("../static/css/pages/layout.css");
const AUTH_CSS: &str = include_str!("../static/css/pages/auth.css");
const ACCOUNT_CSS: &str = include_str!("../static/css/pages/account.css");
const MINING_QUEUE_CSS: &str = include_str!("../static/css/pages/mining_queue.css");
const MINING_AREA_ATLAS_CSS: &str = include_str!("../static/css/pages/mining_area_atlas.css");
const MINING_RESULTS_CSS: &str = include_str!("../static/css/pages/mining_results.css");
const ACTIVITY_CSS: &str = include_str!("../static/css/pages/activity.css");
const RALLY_CSS: &str = include_str!("../static/css/pages/rally.css");
const EDIT_CODE_CSS: &str = include_str!("../static/css/pages/edit_code.css");
const ROBOT_CSS: &str = include_str!("../static/css/pages/robot.css");
const ACHIEVEMENTS_CSS: &str = include_str!("../static/css/pages/achievements.css");
const SHOP_CSS: &str = include_str!("../static/css/pages/shop.css");
const HELP_CSS: &str = include_str!("../static/css/pages/help.css");
const LEADERBOARD_CSS: &str = include_str!("../static/css/pages/leaderboard.css");
const ROBOT_STATS_CSS: &str = include_str!("../static/css/pages/robot_stats.css");

/// Page CSS files in original monolith section order, paired with their
/// `static/` relative path so the served file and the hashed `<link>` stay coherent.
const PAGE_STYLESHEETS: &[(&str, &str)] = &[
    ("css/pages/layout.css", LAYOUT_CSS),
    ("css/pages/auth.css", AUTH_CSS),
    ("css/pages/account.css", ACCOUNT_CSS),
    ("css/pages/mining_queue.css", MINING_QUEUE_CSS),
    ("css/pages/mining_area_atlas.css", MINING_AREA_ATLAS_CSS),
    ("css/pages/mining_results.css", MINING_RESULTS_CSS),
    ("css/pages/activity.css", ACTIVITY_CSS),
    ("css/pages/rally.css", RALLY_CSS),
    ("css/pages/edit_code.css", EDIT_CODE_CSS),
    ("css/pages/robot.css", ROBOT_CSS),
    ("css/pages/achievements.css", ACHIEVEMENTS_CSS),
    ("css/pages/shop.css", SHOP_CSS),
    ("css/pages/help.css", HELP_CSS),
    ("css/pages/leaderboard.css", LEADERBOARD_CSS),
    ("css/pages/robot_stats.css", ROBOT_STATS_CSS),
];

/// Canonical RoboMiner stylesheet links (one per page-scoped CSS file, in
/// original section order), each with its own cache-busting content hash.
pub(crate) fn robominer_stylesheet_tag() -> String {
    let mut out = String::new();
    for (href_path, contents) in PAGE_STYLESHEETS {
        out.push_str(&stylesheet_href_tag(href_path, contents));
        out.push('\n');
    }
    // Drop the trailing newline so callers embedding this in a single
    // `{}` slot don't accumulate stray blank lines.
    out.pop();
    out
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
        assert!(stylesheet_href_tag("css/pages/layout.css", css).contains(&format!("?v={hash}")));
        assert!(script_src_tag("js/x.js", css).contains(&format!("?v={hash}")));
    }

    #[test]
    fn robominer_stylesheet_tag_emits_one_link_per_page_css_file_in_order() {
        let tags = robominer_stylesheet_tag();
        let links: Vec<&str> = tags.lines().collect();
        assert_eq!(links.len(), PAGE_STYLESHEETS.len());
        for ((href_path, _), link) in PAGE_STYLESHEETS.iter().zip(links.iter()) {
            assert!(
                link.contains(&format!(r#"href="{href_path}?v="#)),
                "expected link for {href_path}, got: {link}"
            );
        }
        // Spot check original section order is preserved: layout first, auth
        // before account, leaderboard before the trailing robot-stats page.
        assert!(
            tags.find("css/pages/layout.css").unwrap() < tags.find("css/pages/auth.css").unwrap()
        );
        assert!(
            tags.find("css/pages/auth.css").unwrap() < tags.find("css/pages/account.css").unwrap()
        );
        assert!(
            tags.find("css/pages/leaderboard.css").unwrap()
                < tags.find("css/pages/robot_stats.css").unwrap()
        );
    }

    #[test]
    fn page_script_helpers_include_shared_modules_first() {
        let with_url = page_scripts_with_url_query("js/x.js", "x");
        assert!(with_url.contains("js/common/url_query.js?v="));
        assert!(with_url.contains("js/x.js?v="));

        let with_session = page_scripts_with_url_query_and_session("js/x.js", "x");
        assert!(with_session.contains("js/common/session_store.js?v="));

        let with_panel = page_scripts_with_panel_and_url_query("js/x.js", "x");
        assert!(with_panel.contains("js/common/panel_state.js?v="));
    }
}
