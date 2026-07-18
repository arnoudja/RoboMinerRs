use super::{
    format_relative_time_millis, inject_csrf_tokens, layout, render_claimed_ore_rewards_banner,
};
use crate::app_shell::render_app_shell_hud;
use robominer_db::{
    AppShellHudRecord, ClaimedOreRewardRecord, ClaimedUserResults, UserOreAssetStateRecord,
};

#[test]
fn inject_csrf_tokens_adds_meta_and_post_form_fields() {
    let html = r#"<!DOCTYPE html><html><head><title>t</title></head><body>
<form action="shop" method="post"><button>Buy</button></form>
<form action="search" method="get"><button>Go</button></form>
</body></html>"#;
    let injected = inject_csrf_tokens(html, "abc123");

    assert!(injected.contains(r#"<meta name="csrf-token" content="abc123">"#));
    assert!(injected.contains(
        r#"<form action="shop" method="post"><input type="hidden" name="csrfToken" value="abc123"/>"#
    ));
    assert!(
        !injected.contains(
            r#"<form action="search" method="get"><input type="hidden" name="csrfToken""#
        )
    );

    let again = inject_csrf_tokens(&injected, "abc123");
    assert_eq!(again.matches(r#"name="csrfToken""#).count(), 1);
    assert_eq!(again.matches(r#"name="csrf-token""#).count(), 1);
}

#[test]
fn relative_time_formats_recent_intervals() {
    let now = 3_600_000;
    assert_eq!(format_relative_time_millis(now, now), "just now");
    assert_eq!(format_relative_time_millis(now - 30_000, now), "just now");
    assert_eq!(
        format_relative_time_millis(now - 120_000, now),
        "2 minutes ago"
    );
    assert_eq!(
        format_relative_time_millis(now - 3_600_000, now),
        "1 hour ago"
    );
    assert_eq!(
        format_relative_time_millis(now - 86_400_000, now),
        "1 day ago"
    );
}

#[test]
fn relative_time_falls_back_to_absolute_for_old_events() {
    let now = 86_400_000_i64 * 400;
    assert_eq!(
        format_relative_time_millis(0, now),
        "1970-01-01 00:00:00 UTC"
    );
}

#[test]
fn app_shell_header_marks_active_page_and_includes_atlas() {
    let html = layout(
        "RoboMiner - Mining queue",
        "miningQueue",
        "Player",
        None,
        "<p>Body</p>",
    );

    assert!(html.contains(r##"href="#main-content">Skip to content</a>"##));
    assert!(html.contains(r#"id="main-content""#));
    assert!(html.contains(r#"class="app-shell-header""#));
    assert!(html.contains(r#"href="miningAreaOverview">Areas</a>"#));
    assert!(html.contains(r#"app-shell-account-user">Player</span>"#));
    assert!(html.contains(r#"id="app-shell-nav-toggle""#));
    assert!(html.contains(r#"for="app-shell-nav-toggle""#));
    assert!(html.contains(r#"id="app-shell-nav-panel""#));
    assert!(html.contains(r#"href="miningQueue" aria-current="page">Mining queue</a>"#));
    assert!(!html.contains("menuitemselected"));
    assert!(!html.contains(r#"nav class="logoff""#));
    assert!(html.contains(r#"class="robominer-dialog""#));
    assert!(html.contains("window.robominerConfirm"));
    assert!(html.contains("window.robominerAlert"));
}

#[test]
fn app_shell_header_marks_account_page_active() {
    let html = layout(
        "RoboMiner - Account",
        "account",
        "Player",
        None,
        "<p>Body</p>",
    );

    assert!(html.contains(r#"href="account" aria-current="page""#));
    assert!(html.contains(r#"title="Player""#));
    assert!(html.contains(r#"app-shell-account-user">Player</span>"#));
    assert!(!html.contains(r#"href="miningQueue" aria-current="page""#));
}

#[test]
fn app_shell_footer_includes_help_link() {
    let html = layout("RoboMiner - Help", "help", "Player", None, "<p>Body</p>");

    assert!(html.contains(r#"class="app-shell-footer""#));
    assert!(html.contains(r#"class="app-shell-footer-link" href="help">Help</a>"#));
    assert!(html.contains(r#"href="miningQueue">Mining queue</a>"#));
    assert!(html.contains(r#"href="https://opensource.org/license/mit""#));
    assert!(html.contains(r#"href="https://www.apache.org/licenses/LICENSE-2.0""#));
}

#[test]
fn claimed_ore_rewards_banner_lists_net_wallet_increases() {
    let html = render_claimed_ore_rewards_banner(
        "mining-queue-claim-banner",
        &ClaimedUserResults {
            claimed_queues: 2,
            ore_rewards: vec![
                ClaimedOreRewardRecord {
                    ore_id: 3,
                    ore_name: "Lithabine".to_string(),
                    reward: 4,
                },
                ClaimedOreRewardRecord {
                    ore_id: 1,
                    ore_name: "Cerbonium".to_string(),
                    reward: 9,
                },
            ],
        },
        true,
    );

    assert!(html.contains("Added to wallet:"));
    assert!(html.contains("Lithabine"));
    assert!(html.contains(r#"class="claim-banner-reward-amount">+4</span>"#));
    assert!(html.contains(r#"class="claim-banner-reward-amount">+9</span>"#));
    assert!(html.contains(r#"href="miningResults">View results</a>"#));
    assert!(!html.contains("Claimed 2 mining result(s)"));
}

#[test]
fn claimed_ore_rewards_banner_is_empty_when_nothing_was_claimed() {
    let html = render_claimed_ore_rewards_banner(
        "mining-results-claim-banner",
        &ClaimedUserResults {
            claimed_queues: 0,
            ore_rewards: vec![],
        },
        false,
    );

    assert!(html.is_empty());
}

#[test]
fn app_shell_header_renders_hud_markup() {
    let hud = render_app_shell_hud(&AppShellHudRecord {
        ore_assets: vec![UserOreAssetStateRecord {
            ore_id: 1,
            ore_name: "Iron".to_string(),
            amount: 4,
            max_allowed: 20,
        depot_max_allowed: 0,
        }],
        queue_used: 1,
        queue_capacity: 4,
        claimable_achievements_count: 0,
    });
    let html = layout(
        "RoboMiner - Shop",
        "shop",
        "Player",
        Some(&hud),
        "<p>Body</p>",
    );

    assert!(html.contains(r#"class="app-shell-hud""#));
    assert!(html.contains(r#"app-shell-hud-value">1/4</span>"#));
    assert!(html.contains("Iron"));
}
