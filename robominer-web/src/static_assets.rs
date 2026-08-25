//! Helpers for linking static JS/CSS from `webroot` with content-hash cache busting.

use sha2::{Digest, Sha256};

pub(crate) const URL_QUERY_JS: &str = include_str!("../static/js/common/url_query.js");
pub(crate) const SESSION_STORE_JS: &str = include_str!("../static/js/common/session_store.js");
pub(crate) const PANEL_STATE_JS: &str = include_str!("../static/js/common/panel_state.js");
pub(crate) const AREA_FILTER_JS: &str = include_str!("../static/js/common/area_filter.js");
pub(crate) const FILTER_RESTORE_JS: &str = include_str!("../static/js/common/filter_restore.js");

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

/// Page scripts with URL/session helpers plus shared filter restore.
pub(crate) fn page_scripts_with_filter_restore(page_src: &str, page_js: &str) -> String {
    let mut out =
        page_scripts_with_url_query_and_session("js/common/filter_restore.js", FILTER_RESTORE_JS);
    // page_scripts_with_url_query_and_session already ends with a script tag + newline for
    // its "page" slot; append the real page script after the filter helper.
    out.push_str(&script_src_tag(page_src, page_js));
    out.push('\n');
    out
}

/// Page scripts that need panel state + URL query helpers, then page logic.
pub(crate) fn page_scripts_with_panel_and_url_query(page_src: &str, page_js: &str) -> String {
    script_src_tags(&[
        ("js/common/panel_state.js", PANEL_STATE_JS),
        ("js/common/url_query.js", URL_QUERY_JS),
        (page_src, page_js),
    ])
}

const LAYOUT_SHELL_CSS: &str = include_str!("../static/css/pages/layout_shell.css");
const LAYOUT_DIALOGS_CSS: &str = include_str!("../static/css/pages/layout_dialogs.css");
const LAYOUT_TABLES_CSS: &str = include_str!("../static/css/pages/layout_tables.css");
const AUTH_CSS: &str = include_str!("../static/css/pages/auth.css");
const ACCOUNT_CSS: &str = include_str!("../static/css/pages/account.css");
const PAGE_WALLET_CSS: &str = include_str!("../static/css/pages/page_wallet.css");
const MINING_QUEUE_CSS: &str = include_str!("../static/css/pages/mining_queue.css");
const MINING_QUEUE_ROBOTS_CSS: &str = include_str!("../static/css/pages/mining_queue_robots.css");
const MINING_AREA_ATLAS_CSS: &str = include_str!("../static/css/pages/mining_area_atlas.css");
const MINING_RESULTS_CSS: &str = include_str!("../static/css/pages/mining_results.css");
const ACTIVITY_CSS: &str = include_str!("../static/css/pages/activity.css");
const RALLY_CSS: &str = include_str!("../static/css/pages/rally.css");
const RALLY_SIDEBAR_CSS: &str = include_str!("../static/css/pages/rally_sidebar.css");
const EDIT_CODE_CSS: &str = include_str!("../static/css/pages/edit_code.css");
const ROBOT_CSS: &str = include_str!("../static/css/pages/robot.css");
const ACHIEVEMENTS_CSS: &str = include_str!("../static/css/pages/achievements.css");
const SHOP_CSS: &str = include_str!("../static/css/pages/shop.css");
const HELP_CSS: &str = include_str!("../static/css/pages/help.css");
const LEADERBOARD_CSS: &str = include_str!("../static/css/pages/leaderboard.css");
const ROBOT_STATS_CSS: &str = include_str!("../static/css/pages/robot_stats.css");

/// Page-specific stylesheet beyond the shared layout shell.
///
/// Always paired with the shared layout partials via [`robominer_stylesheet_tags`].
/// Shared strips used by more than one page (e.g. [`Self::PageWallet`]) are requested
/// alongside the page file.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum PageStylesheet {
    Auth,
    Account,
    PageWallet,
    MiningQueue,
    MiningAreaAtlas,
    MiningResults,
    Activity,
    Rally,
    EditCode,
    Robot,
    Achievements,
    Shop,
    Help,
    Leaderboard,
    RobotStats,
}

impl PageStylesheet {
    fn entries(self) -> &'static [(&'static str, &'static str)] {
        match self {
            Self::Auth => &[("css/pages/auth.css", AUTH_CSS)],
            Self::Account => &[("css/pages/account.css", ACCOUNT_CSS)],
            Self::PageWallet => &[("css/pages/page_wallet.css", PAGE_WALLET_CSS)],
            Self::MiningQueue => &[
                ("css/pages/mining_queue.css", MINING_QUEUE_CSS),
                ("css/pages/mining_queue_robots.css", MINING_QUEUE_ROBOTS_CSS),
            ],
            Self::MiningAreaAtlas => &[("css/pages/mining_area_atlas.css", MINING_AREA_ATLAS_CSS)],
            Self::MiningResults => &[("css/pages/mining_results.css", MINING_RESULTS_CSS)],
            Self::Activity => &[("css/pages/activity.css", ACTIVITY_CSS)],
            Self::Rally => &[
                ("css/pages/rally.css", RALLY_CSS),
                ("css/pages/rally_sidebar.css", RALLY_SIDEBAR_CSS),
            ],
            Self::EditCode => &[("css/pages/edit_code.css", EDIT_CODE_CSS)],
            Self::Robot => &[("css/pages/robot.css", ROBOT_CSS)],
            Self::Achievements => &[("css/pages/achievements.css", ACHIEVEMENTS_CSS)],
            Self::Shop => &[("css/pages/shop.css", SHOP_CSS)],
            Self::Help => &[("css/pages/help.css", HELP_CSS)],
            Self::Leaderboard => &[("css/pages/leaderboard.css", LEADERBOARD_CSS)],
            Self::RobotStats => &[("css/pages/robot_stats.css", ROBOT_STATS_CSS)],
        }
    }
}

/// Stylesheet links for a page: always the shared layout partials (shell, dialogs,
/// tables), then each requested page file (deduped, stable enum order). Each link is
/// content-hashed independently.
pub(crate) fn robominer_stylesheet_tags(pages: &[PageStylesheet]) -> String {
    let mut out = String::new();
    out.push_str(&stylesheet_href_tag(
        "css/pages/layout_shell.css",
        LAYOUT_SHELL_CSS,
    ));
    out.push('\n');
    out.push_str(&stylesheet_href_tag(
        "css/pages/layout_dialogs.css",
        LAYOUT_DIALOGS_CSS,
    ));
    out.push('\n');
    out.push_str(&stylesheet_href_tag(
        "css/pages/layout_tables.css",
        LAYOUT_TABLES_CSS,
    ));
    let mut emitted = [false; 15];
    for page in pages {
        let index = *page as usize;
        if emitted[index] {
            continue;
        }
        emitted[index] = true;
        for (href_path, contents) in page.entries() {
            out.push('\n');
            out.push_str(&stylesheet_href_tag(href_path, contents));
        }
    }
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
        assert!(
            stylesheet_href_tag("css/pages/layout_shell.css", css).contains(&format!("?v={hash}"))
        );
        assert!(script_src_tag("js/x.js", css).contains(&format!("?v={hash}")));
    }

    #[test]
    fn robominer_stylesheet_tags_always_include_layout_then_requested_pages() {
        let tags = robominer_stylesheet_tags(&[PageStylesheet::Help]);
        let links: Vec<&str> = tags.lines().collect();
        assert_eq!(links.len(), 4);
        assert!(links[0].contains(r#"href="css/pages/layout_shell.css?v="#));
        assert!(links[1].contains(r#"href="css/pages/layout_dialogs.css?v="#));
        assert!(links[2].contains(r#"href="css/pages/layout_tables.css?v="#));
        assert!(links[3].contains(r#"href="css/pages/help.css?v="#));
        assert!(!tags.contains("css/pages/shop.css"));
    }

    #[test]
    fn help_stylesheet_owns_help_page_rules_not_auth() {
        assert!(
            HELP_CSS.contains(".help-page"),
            "help.css should define modern help-page styles"
        );
        assert!(
            !AUTH_CSS.contains(".help-page"),
            "auth.css must not own help-page styles (page-scoped CSS)"
        );
    }

    #[test]
    fn rally_stylesheet_owns_back_link_not_activity() {
        assert!(
            RALLY_CSS.contains(".rally-view-back-link"),
            "rally.css should style the rally view back link"
        );
        assert!(
            !ACTIVITY_CSS.contains(".rally-view-back-link"),
            "activity.css must not own rally-view-back-link (page-scoped CSS)"
        );
    }

    #[test]
    fn layout_shell_owns_shared_claim_banner_innards() {
        assert!(
            LAYOUT_SHELL_CSS.contains(".claim-banner-label"),
            "layout_shell.css should style shared claim-banner innards"
        );
        assert!(
            !MINING_QUEUE_CSS.contains(".claim-banner-label"),
            "mining_queue.css must not own shared claim-banner innards"
        );
    }

    #[test]
    fn robominer_stylesheet_tags_emit_split_page_sheets() {
        let tags = robominer_stylesheet_tags(&[PageStylesheet::MiningQueue]);
        assert!(tags.contains(r#"href="css/pages/mining_queue.css?v="#));
        assert!(tags.contains(r#"href="css/pages/mining_queue_robots.css?v="#));

        let rally = robominer_stylesheet_tags(&[PageStylesheet::Rally]);
        assert!(rally.contains(r#"href="css/pages/rally.css?v="#));
        assert!(rally.contains(r#"href="css/pages/rally_sidebar.css?v="#));
    }

    #[test]
    fn robominer_stylesheet_tags_dedupe_repeated_pages() {
        let tags = robominer_stylesheet_tags(&[PageStylesheet::Shop, PageStylesheet::Shop]);
        assert_eq!(tags.matches(r#"href="css/pages/shop.css?v="#).count(), 1);
        assert_eq!(tags.lines().count(), 4);
    }

    #[test]
    fn robominer_stylesheet_tags_can_pair_shared_wallet_with_page() {
        let tags = robominer_stylesheet_tags(&[PageStylesheet::PageWallet, PageStylesheet::Shop]);
        let links: Vec<&str> = tags.lines().collect();
        assert_eq!(links.len(), 5);
        assert!(links[3].contains(r#"href="css/pages/page_wallet.css?v="#));
        assert!(links[4].contains(r#"href="css/pages/shop.css?v="#));
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
