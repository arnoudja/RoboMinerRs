use super::format::escape_html;

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
