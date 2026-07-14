use robominer_db::AppShellHudRecord;

use crate::html::escape_html;
use crate::{Request, ServerConfig, block_on_database, request_user_id};

pub(crate) fn hud_markup(request: &Request, config: &ServerConfig) -> Option<String> {
    let user_id = request_user_id(request)?;
    let pool = config.database_pool.as_ref()?;
    let hud = block_on_database(robominer_domain::load_app_shell_hud(pool, user_id)).ok()?;
    Some(render_app_shell_hud(&hud))
}

pub(crate) fn render_app_shell_hud(hud: &AppShellHudRecord) -> String {
    let mut parts = Vec::new();

    parts.push(format!(
        r#"<a class="app-shell-hud-item app-shell-hud-queue" href="miningQueue" title="Mining queue slots used"><span class="app-shell-hud-label">Queue</span><span class="app-shell-hud-value">{}/{}</span></a>"#,
        hud.queue_used, hud.queue_capacity
    ));

    if hud.claimable_achievements_count > 0 {
        let label = if hud.claimable_achievements_count == 1 {
            "achievement to claim"
        } else {
            "achievements to claim"
        };
        parts.push(format!(
            r#"<a class="app-shell-hud-item app-shell-hud-achievements app-shell-hud-alert" href="achievements" title="Achievement steps ready to claim"><span class="app-shell-hud-value">{}</span><span class="app-shell-hud-label">{}</span></a>"#,
            hud.claimable_achievements_count, label
        ));
    }

    if !hud.ore_assets.is_empty() {
        let mut wallet =
            String::from(r#"<div class="app-shell-hud-wallet" aria-label="Ore wallet">"#);
        wallet.push_str(r#"<ul class="app-shell-hud-wallet-list">"#);
        for asset in hud.ore_assets.iter().take(3) {
            let full_class = if asset.amount >= asset.max_allowed {
                " app-shell-hud-wallet-full"
            } else {
                ""
            };
            wallet.push_str(&format!(
                r#"<li class="app-shell-hud-wallet-item{full_class}"><span class="app-shell-hud-wallet-ore">{}</span><span class="app-shell-hud-wallet-amount">{}/{}</span></li>"#,
                escape_html(&asset.ore_name),
                asset.amount,
                asset.max_allowed
            ));
        }
        if hud.ore_assets.len() > 3 {
            wallet.push_str(&format!(
                r#"<li class="app-shell-hud-wallet-more">+{} more</li>"#,
                hud.ore_assets.len() - 3
            ));
        }
        wallet.push_str("</ul></div>");
        parts.push(wallet);
    }

    format!(r#"<div class="app-shell-hud">{}</div>"#, parts.join(""))
}

#[cfg(test)]
mod tests {
    use robominer_db::{AppShellHudRecord, UserOreAssetStateRecord};

    use super::render_app_shell_hud;

    #[test]
    fn hud_renders_queue_and_wallet() {
        let html = render_app_shell_hud(&AppShellHudRecord {
            ore_assets: vec![
                UserOreAssetStateRecord {
                    ore_id: 1,
                    ore_name: "Iron".to_string(),
                    amount: 12,
                    max_allowed: 50,
                },
                UserOreAssetStateRecord {
                    ore_id: 2,
                    ore_name: "Gold".to_string(),
                    amount: 50,
                    max_allowed: 50,
                },
            ],
            queue_used: 2,
            queue_capacity: 8,
            claimable_achievements_count: 0,
        });

        assert!(html.contains(r#"class="app-shell-hud""#));
        assert!(html.contains(r#"app-shell-hud-value">2/8</span>"#));
        assert!(html.contains("Iron"));
        assert!(html.contains("app-shell-hud-wallet-full"));
    }

    #[test]
    fn hud_renders_queue_when_wallet_is_empty() {
        let html = render_app_shell_hud(&AppShellHudRecord {
            ore_assets: vec![],
            queue_used: 0,
            queue_capacity: 4,
            claimable_achievements_count: 0,
        });

        assert!(html.contains("0/4"));
        assert!(!html.contains("app-shell-hud-wallet"));
    }

    #[test]
    fn hud_renders_claimable_achievements_badge() {
        let html = render_app_shell_hud(&AppShellHudRecord {
            ore_assets: vec![],
            queue_used: 1,
            queue_capacity: 4,
            claimable_achievements_count: 2,
        });

        assert!(html.contains(r#"app-shell-hud-alert""#));
        assert!(html.contains(r#"href="achievements""#));
        assert!(
            html.contains(
                r#"2</span><span class="app-shell-hud-label">achievements to claim</span>"#
            )
        );
    }

    #[test]
    fn hud_omits_achievements_badge_when_nothing_is_claimable() {
        let html = render_app_shell_hud(&AppShellHudRecord {
            ore_assets: vec![],
            queue_used: 1,
            queue_capacity: 4,
            claimable_achievements_count: 0,
        });

        assert!(!html.contains("app-shell-hud-achievements"));
    }
}
