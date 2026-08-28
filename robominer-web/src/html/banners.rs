use super::format::escape_html;

pub(crate) const CLAIM_PENDING_RESULTS_FIELD: &str = "claimPendingResults";

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

pub(crate) fn render_pending_claim_banner(
    banner_class: &str,
    pending_count: u64,
    form_action: &str,
) -> String {
    if pending_count == 0 {
        return String::new();
    }

    let label = if pending_count == 1 {
        "1 mining result ready to claim".to_string()
    } else {
        format!("{pending_count} mining results ready to claim")
    };

    format!(
        r#"<form class="claim-pending-form {banner_class}" action="{}" method="post"><p class="claim-pending-copy"><span class="claim-banner-label">{label}.</span></p><button type="submit" class="claim-pending-button" name="{CLAIM_PENDING_RESULTS_FIELD}" value="1">Claim rewards</button></form>"#,
        escape_html(form_action),
    )
}

/// Success banner after POST claim, otherwise a pending-claim form when results are waiting.
pub(crate) fn render_mining_claim_ui(
    banner_class: &str,
    form_action: &str,
    pending_count: u64,
    claimed: &robominer_db::ClaimedUserResults,
    include_results_link: bool,
) -> String {
    if claimed.claimed_queues > 0 {
        return render_claimed_ore_rewards_banner(banner_class, claimed, include_results_link);
    }
    render_pending_claim_banner(banner_class, pending_count, form_action)
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
