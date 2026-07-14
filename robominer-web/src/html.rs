pub(crate) fn selected_attr(selected: bool) -> &'static str {
    if selected { " selected" } else { "" }
}

pub(crate) fn layout(
    title: &str,
    current_form: &str,
    username: &str,
    hud_markup: Option<&str>,
    body: &str,
) -> String {
    format!(
        r##"<!DOCTYPE html>
<html>
    <head>
        <meta http-equiv="Content-Type" content="text/html; charset=UTF-8">
        <meta name="viewport" content="width=device-width, initial-scale=1">
        <link rel="stylesheet" type="text/css" href="css/robominer.css">
        <title>{}</title>
    </head>
    <body>
        <a class="app-shell-skip" href="#main-content">Skip to content</a>
        <div class="main">
            {}
            <div class="interface" id="main-content">
                {}
            </div>
            {}
        </div>
        {}
        {}
    </body>
</html>"##,
        escape_html(title),
        app_shell_header(current_form, username, hud_markup),
        body,
        page_footer(),
        app_dialog_markup(),
        app_dialog_script()
    )
}

fn app_dialog_markup() -> &'static str {
    r#"<div id="robominerDialog" class="robominer-dialog" hidden>
    <button type="button" class="robominer-dialog-backdrop" id="robominerDialogBackdrop" aria-label="Close dialog"></button>
    <div class="robominer-dialog-panel" role="dialog" aria-modal="true" aria-labelledby="robominerDialogTitle">
        <h2 id="robominerDialogTitle" class="robominer-dialog-title">Confirm</h2>
        <p id="robominerDialogMessage" class="robominer-dialog-message"></p>
        <div class="robominer-dialog-actions">
            <button type="button" id="robominerDialogCancel" class="robominer-dialog-btn robominer-dialog-btn-secondary">Cancel</button>
            <button type="button" id="robominerDialogConfirm" class="robominer-dialog-btn robominer-dialog-btn-primary">Confirm</button>
        </div>
    </div>
</div>"#
}

fn app_dialog_script() -> &'static str {
    r#"<script>
(function() {
    var dialog = document.getElementById('robominerDialog');
    var title = document.getElementById('robominerDialogTitle');
    var message = document.getElementById('robominerDialogMessage');
    var cancelButton = document.getElementById('robominerDialogCancel');
    var confirmButton = document.getElementById('robominerDialogConfirm');
    var backdrop = document.getElementById('robominerDialogBackdrop');
    if (!dialog || !title || !message || !cancelButton || !confirmButton || !backdrop) {
        return;
    }

    var pendingCallback = null;
    var alertMode = false;
    var lastFocusedElement = null;

    function finish(result) {
        dialog.hidden = true;
        document.body.classList.remove('robominer-dialog-open');
        var callback = pendingCallback;
        pendingCallback = null;
        alertMode = false;
        if (lastFocusedElement && typeof lastFocusedElement.focus === 'function') {
            lastFocusedElement.focus();
        }
        lastFocusedElement = null;
        if (callback) {
            callback(result);
        }
    }

    function openDialog(options) {
        alertMode = !!options.alert;
        title.textContent = options.title;
        message.textContent = options.message;
        cancelButton.hidden = alertMode;
        confirmButton.textContent = options.confirmLabel;
        pendingCallback = options.onResult;
        lastFocusedElement = document.activeElement;
        dialog.hidden = false;
        document.body.classList.add('robominer-dialog-open');
        confirmButton.focus();
    }

    window.robominerConfirm = function(dialogMessage, onResult) {
        openDialog({
            alert: false,
            title: 'Confirm',
            message: dialogMessage,
            confirmLabel: 'Confirm',
            onResult: onResult
        });
    };

    window.robominerAlert = function(dialogMessage, onDismiss) {
        openDialog({
            alert: true,
            title: 'Notice',
            message: dialogMessage,
            confirmLabel: 'OK',
            onResult: onDismiss || null
        });
    };

    cancelButton.addEventListener('click', function() {
        finish(false);
    });
    backdrop.addEventListener('click', function() {
        if (!alertMode) {
            finish(false);
        }
    });
    confirmButton.addEventListener('click', function() {
        finish(true);
    });
    document.addEventListener('keydown', function(event) {
        if (dialog.hidden) {
            return;
        }
        if (event.key === 'Escape') {
            event.preventDefault();
            finish(alertMode ? true : false);
        }
    });
})();
</script>"#
}

fn app_shell_header(current_form: &str, username: &str, hud_markup: Option<&str>) -> String {
    let menu_link = |form: &str, href: &str, label: &str| {
        nav_link(current_form == form, href, label)
    };

    format!(
        r#"<header class="app-shell-header">
    <div class="app-shell-inner">
        <a class="app-shell-home" href="miningQueue">RoboMiner</a>
        {}
        <input type="checkbox" id="app-shell-nav-toggle" class="app-shell-nav-toggle">
        <label for="app-shell-nav-toggle" class="app-shell-menu-toggle">
            <span class="app-shell-menu-toggle-icon" aria-hidden="true"></span>
            <span class="app-shell-menu-toggle-text">Menu</span>
        </label>
        <div class="app-shell-nav-panel" id="app-shell-nav-panel">
            <nav class="app-shell-nav" aria-label="Main navigation">
                <ul class="app-shell-menu">
                    <li class="app-shell-group">
                        <span class="app-shell-group-label">Play</span>
                        <ul class="app-shell-group-links">
                            {}
                            {}
                            {}
                        </ul>
                    </li>
                    <li class="app-shell-group">
                        <span class="app-shell-group-label">Build</span>
                        <ul class="app-shell-group-links">
                            {}
                            {}
                            {}
                        </ul>
                    </li>
                    <li class="app-shell-group">
                        <span class="app-shell-group-label">Compete</span>
                        <ul class="app-shell-group-links">
                            {}
                            {}
                            {}
                        </ul>
                    </li>
                </ul>
            </nav>
            <nav class="app-shell-account" aria-label="Account">
                <ul class="app-shell-account-links">
                    {}
                    {}
                    {}
                </ul>
            </nav>
        </div>
    </div>
</header>"#,
        hud_markup.unwrap_or(""),
        menu_link("miningQueue", "miningQueue", "Mining queue"),
        menu_link("miningResults", "miningResults", "Mining results"),
        menu_link("miningAreaOverview", "miningAreaOverview", "Areas"),
        menu_link("editCode", "editCode", "Edit code"),
        menu_link("robot", "robot", "Robots"),
        menu_link("shop", "shop", "Shop"),
        menu_link("leaderboard", "leaderboard", "Leaderboard"),
        menu_link("achievements", "achievements", "Achievements"),
        menu_link("activity", "activity", "Activity"),
        nav_link(current_form == "help", "help", "Help"),
        account_nav_link(current_form == "account", username),
        r#"<li><a class="app-shell-link" href="logoff">Log off</a></li>"#,
    )
}

fn nav_link(selected: bool, href: &str, label: &str) -> String {
    let class_name = if selected {
        "app-shell-link app-shell-link-active"
    } else {
        "app-shell-link"
    };
    let aria_current = if selected {
        r#" aria-current="page""#
    } else {
        ""
    };

    format!(
        r#"<li><a class="{class_name}" href="{href}"{aria_current}>{}</a></li>"#,
        escape_html(label)
    )
}

fn account_nav_link(selected: bool, username: &str) -> String {
    let class_name = if selected {
        "app-shell-link app-shell-link-active"
    } else {
        "app-shell-link"
    };
    let aria_current = if selected {
        r#" aria-current="page""#
    } else {
        ""
    };

    format!(
        r#"<li><a class="{class_name} app-shell-account-link" href="account"{aria_current} title="{}"><span class="app-shell-account-label">Account</span><span class="app-shell-account-user">{}</span></a></li>"#,
        escape_html(username),
        escape_html(username)
    )
}

pub(crate) fn page_footer() -> &'static str {
    r#"<footer class="app-shell-footer">
    <div class="app-shell-inner app-shell-footer-inner">
        <p class="app-shell-footer-brand">RoboMiner · <a class="app-shell-footer-link" href="https://opensource.org/license/mit" rel="license noopener noreferrer" target="_blank">MIT</a> OR <a class="app-shell-footer-link" href="https://www.apache.org/licenses/LICENSE-2.0" rel="license noopener noreferrer" target="_blank">Apache-2.0</a></p>
        <nav class="app-shell-footer-nav" aria-label="Footer">
            <a class="app-shell-footer-link" href="help">Help</a>
            <a class="app-shell-footer-link" href="miningQueue">Mining queue</a>
        </nav>
    </div>
</footer>"#
}

pub(crate) fn escape_html(value: &str) -> String {
    value
        .replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
        .replace('\'', "&#39;")
}

pub(crate) fn escape_js_string(value: &str) -> String {
    value
        .replace('\\', "\\\\")
        .replace('\'', "\\'")
        .replace('\n', "\\n")
        .replace('\r', "\\r")
        .replace('<', "\\x3c")
        .replace('>', "\\x3e")
        .replace('&', "\\x26")
}

pub(crate) fn format_utc_millis(millis: i64) -> String {
    let seconds = millis.div_euclid(1000);
    let days = seconds.div_euclid(86_400);
    let seconds_of_day = seconds.rem_euclid(86_400);
    let (year, month, day) = civil_from_days(days);
    let hour = seconds_of_day / 3_600;
    let minute = (seconds_of_day % 3_600) / 60;
    let second = seconds_of_day % 60;

    format!("{year:04}-{month:02}-{day:02} {hour:02}:{minute:02}:{second:02} UTC")
}

pub(crate) fn format_relative_time_millis(event_millis: i64, now_millis: i64) -> String {
    let diff_seconds = (now_millis - event_millis).div_euclid(1000);
    if diff_seconds < 0 {
        return format_utc_millis(event_millis);
    }
    if diff_seconds < 45 {
        return "just now".to_string();
    }
    if diff_seconds < 90 {
        return "1 minute ago".to_string();
    }
    let minutes = diff_seconds / 60;
    if minutes < 45 {
        return format!("{minutes} minutes ago");
    }
    if minutes < 90 {
        return "1 hour ago".to_string();
    }
    let hours = minutes / 60;
    if hours < 24 {
        return if hours == 1 {
            "1 hour ago".to_string()
        } else {
            format!("{hours} hours ago")
        };
    }
    if hours < 48 {
        return "1 day ago".to_string();
    }
    let days = hours / 24;
    if days < 7 {
        return if days == 1 {
            "1 day ago".to_string()
        } else {
            format!("{days} days ago")
        };
    }
    if days < 14 {
        return "1 week ago".to_string();
    }
    let weeks = days / 7;
    if weeks < 5 {
        return if weeks == 1 {
            "1 week ago".to_string()
        } else {
            format!("{weeks} weeks ago")
        };
    }

    format_utc_millis(event_millis)
}

pub(crate) fn render_claimed_ore_rewards_banner(
    banner_class: &str,
    claimed: &robominer_db::ClaimedUserResults,
    include_results_link: bool,
) -> String {
    if claimed.claimed_queues == 0 {
        return String::new();
    }

    let mut reward_markup = String::new();
    if claimed.ore_rewards.is_empty() {
        reward_markup.push_str("No ore added to your wallet.");
    } else {
        reward_markup.push_str(r#"<span class="claim-banner-rewards">"#);
        for (index, reward) in claimed.ore_rewards.iter().enumerate() {
            if index > 0 {
                reward_markup.push_str(", ");
            }
            reward_markup.push_str(&format!(
                r#"<span class="claim-banner-reward"><span class="claim-banner-reward-ore">{}</span><span class="claim-banner-reward-amount">+{}</span></span>"#,
                escape_html(&reward.ore_name),
                reward.reward
            ));
        }
        reward_markup.push_str("</span>");
    }

    let results_link = if include_results_link {
        r#" <a href="miningResults">View results</a>"#
    } else {
        ""
    };

    format!(
        r#"<p class="{banner_class}"><span class="claim-banner-label">Added to wallet:</span> {reward_markup}{results_link}</p>"#
    )
}

pub(crate) fn format_period(seconds: i32) -> String {
    if seconds % 3600 == 0 && seconds > 3600 {
        format!("{} hours", seconds / 3600)
    } else if seconds % 60 == 0 && seconds > 60 {
        format!("{} minutes", seconds / 60)
    } else {
        format!("{seconds} seconds")
    }
}

fn civil_from_days(days_since_epoch: i64) -> (i64, i64, i64) {
    let z = days_since_epoch + 719_468;
    let era = if z >= 0 { z } else { z - 146_096 }.div_euclid(146_097);
    let day_of_era = z - era * 146_097;
    let year_of_era = (day_of_era - day_of_era / 1_460 + day_of_era / 36_524
        - day_of_era / 146_096)
        .div_euclid(365);
    let mut year = year_of_era + era * 400;
    let day_of_year = day_of_era - (365 * year_of_era + year_of_era / 4 - year_of_era / 100);
    let month_prime = (5 * day_of_year + 2).div_euclid(153);
    let day = day_of_year - (153 * month_prime + 2).div_euclid(5) + 1;
    let month = month_prime + if month_prime < 10 { 3 } else { -9 };
    year += if month <= 2 { 1 } else { 0 };

    (year, month, day)
}

#[cfg(test)]
mod tests {
    use super::{format_relative_time_millis, layout, render_claimed_ore_rewards_banner};
    use crate::app_shell::render_app_shell_hud;
    use robominer_db::{
        AppShellHudRecord, ClaimedOreRewardRecord, ClaimedUserResults, UserOreAssetStateRecord,
    };

    #[test]
    fn relative_time_formats_recent_intervals() {
        let now = 3_600_000;
        assert_eq!(format_relative_time_millis(now, now), "just now");
        assert_eq!(format_relative_time_millis(now - 30_000, now), "just now");
        assert_eq!(format_relative_time_millis(now - 120_000, now), "2 minutes ago");
        assert_eq!(format_relative_time_millis(now - 3_600_000, now), "1 hour ago");
        assert_eq!(format_relative_time_millis(now - 86_400_000, now), "1 day ago");
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
}
