use super::format::escape_html;

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

/// Success/error banner used by shop, robot, edit-code, and achievements pages.
pub(crate) fn render_status_banner(body: &mut String, class_prefix: &str, message: Option<&str>) {
    let Some(message) = message else {
        return;
    };
    let banner_class = if message.starts_with("Unable") {
        format!("{class_prefix}-banner {class_prefix}-banner-error")
    } else {
        format!("{class_prefix}-banner {class_prefix}-banner-success")
    };
    body.push_str(&format!(
        r#"<p class="{banner_class}">{}</p>"#,
        escape_html(message)
    ));
}
