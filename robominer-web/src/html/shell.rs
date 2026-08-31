use super::format::EscapedHtml;
use crate::routes::AppRoute;

pub(super) fn app_shell_header(
    current_form: &str,
    username: &str,
    hud_markup: Option<&str>,
) -> String {
    let menu_link =
        |form: &str, href: &str, label: &str| nav_link(current_form == form, href, label);

    format!(
        r#"<header class="app-shell-header">
    <div class="app-shell-inner">
        <a class="app-shell-home" href="{}">RoboMiner</a>
        {}
        <input type="checkbox" id="app-shell-nav-toggle" class="app-shell-nav-toggle">
        <label for="app-shell-nav-toggle" class="app-shell-menu-toggle" aria-controls="app-shell-nav-panel" aria-expanded="false">
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
                    <li><form class="app-shell-logoff-form" action="{}" method="post"><button type="submit" class="app-shell-link app-shell-logoff-button">Log off</button></form></li>
                </ul>
            </nav>
        </div>
    </div>
</header>"#,
        AppRoute::MiningQueue.href(),
        hud_markup.unwrap_or(""),
        menu_link(
            AppRoute::MiningQueue.href(),
            AppRoute::MiningQueue.href(),
            "Mining queue"
        ),
        menu_link(
            AppRoute::MiningResults.href(),
            AppRoute::MiningResults.href(),
            "Mining results"
        ),
        menu_link(
            AppRoute::MiningAreaOverview.href(),
            AppRoute::MiningAreaOverview.href(),
            "Areas"
        ),
        menu_link(
            AppRoute::EditCode.href(),
            AppRoute::EditCode.href(),
            "Edit code"
        ),
        menu_link(AppRoute::Robot.href(), AppRoute::Robot.href(), "Robots"),
        menu_link(AppRoute::Shop.href(), AppRoute::Shop.href(), "Shop"),
        menu_link(
            AppRoute::Leaderboard.href(),
            AppRoute::Leaderboard.href(),
            "Leaderboard"
        ),
        menu_link(
            AppRoute::Achievements.href(),
            AppRoute::Achievements.href(),
            "Achievements"
        ),
        menu_link(
            AppRoute::Activity.href(),
            AppRoute::Activity.href(),
            "Activity"
        ),
        nav_link(
            current_form == AppRoute::Help.href(),
            AppRoute::Help.href(),
            "Help"
        ),
        account_nav_link(current_form == AppRoute::Account.href(), username),
        AppRoute::Logoff.href(),
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
        EscapedHtml::from(label)
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

    let safe_username = EscapedHtml::from_untrusted(username);
    format!(
        r#"<li><a class="{class_name} app-shell-account-link" href="{}"{aria_current} title="{safe_username}"><span class="app-shell-account-label">Account</span><span class="app-shell-account-user">{safe_username}</span></a></li>"#,
        AppRoute::Account.href()
    )
}

pub(crate) fn page_footer() -> String {
    format!(
        r#"<footer class="app-shell-footer">
    <div class="app-shell-inner app-shell-footer-inner">
        <p class="app-shell-footer-brand"><a class="app-shell-footer-link" href="https://github.com/arnoudja/RoboMinerRs" rel="noopener noreferrer" target="_blank">RoboMiner</a> <a class="app-shell-footer-link" href="https://github.com/arnoudja/RoboMinerRs/commits/master/" rel="noopener noreferrer" target="_blank">v{}</a> · <a class="app-shell-footer-link" href="https://opensource.org/license/mit" rel="license noopener noreferrer" target="_blank">MIT</a> OR <a class="app-shell-footer-link" href="https://www.apache.org/licenses/LICENSE-2.0" rel="license noopener noreferrer" target="_blank">Apache-2.0</a></p>
        <nav class="app-shell-footer-nav" aria-label="Footer">
            <a class="app-shell-footer-link" href="{}">Help</a>
        </nav>
    </div>
</footer>"#,
        env!("CARGO_PKG_VERSION"),
        AppRoute::Help.href()
    )
}
