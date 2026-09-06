use crate::html::{EscapedHtml, local_absolute_time_html};

use super::score::render_mining_result_score_breakdown;

pub(super) fn render_mining_result_breakdown(
    body: &mut String,
    result: &robominer_db::MiningResultStateRecord,
    ore_results: &[&robominer_db::MiningResultOreStateRecord],
    action_results: &[&robominer_db::MiningResultActionStateRecord],
    area_ores: &[&robominer_db::MiningResultAreaOreRecord],
) {
    body.push_str(r#"<div class="mining-results-run-breakdown">"#);
    body.push_str(r#"<section class="mining-results-breakdown-section"><h3 class="mining-results-breakdown-title">Payout</h3><dl class="mining-results-payout-list">"#);
    body.push_str(&format!(
        r#"<div class="mining-results-payout-item"><dt>Mined</dt><dd>{}</dd></div><div class="mining-results-payout-item"><dt><span class="mining-results-tax-label" title="Tax is deducted before ore is added to your wallet. Container tax applies to cargo still in the robot; depot tax applies to ore already banked in the depot.">Tax</span></dt><dd>{}</dd></div><div class="mining-results-payout-item"><dt>Net</dt><dd class="mining-results-payout-net">+{}</dd></div><div class="mining-results-payout-item"><dt>Score</dt><dd>{:.1}</dd></div>"#,
        result.total_ore_mined,
        result.total_tax,
        result.total_reward,
        result.score
    ));
    body.push_str("</dl></section>");

    if !ore_results.is_empty() {
        body.push_str(r#"<section class="mining-results-breakdown-section"><h3 class="mining-results-breakdown-title">Ore breakdown</h3><ul class="mining-results-ore-list">"#);
        let mut sorted_ores: Vec<_> = ore_results.to_vec();
        sorted_ores.sort_by_key(|ore_result| std::cmp::Reverse(ore_result.ore_id));
        for ore_result in sorted_ores {
            body.push_str(&format!(
                r#"<li><span class="mining-results-ore-name">{}</span><span class="mining-results-ore-values">{} mined · {} tax · +{} net</span></li>"#,
                EscapedHtml::from(ore_result.ore_name.as_str()),
                ore_result.amount,
                ore_result.tax,
                ore_result.reward,
            ));
        }
        body.push_str("</ul></section>");
    }

    render_mining_result_score_breakdown(body, result, ore_results, area_ores);

    let total_actions: i32 = action_results.iter().map(|action| action.amount).sum();
    if !action_results.is_empty() {
        body.push_str(r#"<section class="mining-results-breakdown-section"><h3 class="mining-results-breakdown-title">Actions</h3><ul class="mining-results-action-list">"#);
        let mut sorted_actions: Vec<_> = action_results.to_vec();
        sorted_actions.sort_by_key(|action| std::cmp::Reverse(action.amount));
        for action in sorted_actions {
            let percentage = if total_actions == 0 {
                0.0
            } else {
                f64::from(action.amount) * 100.0 / f64::from(total_actions)
            };
            body.push_str(&format!(
                r#"<li><span class="mining-results-action-name">{}</span><span class="mining-results-action-values">{} · {:.1}%</span></li>"#,
                action_name(action.action_type),
                action.amount,
                percentage
            ));
        }
        body.push_str(&format!(
            r#"</ul><p class="mining-results-action-total">Total actions: {}</p></section>"#,
            total_actions
        ));
    }

    body.push_str(r#"<section class="mining-results-breakdown-section"><h3 class="mining-results-breakdown-title">Timeline</h3><ul class="mining-results-timeline-list">"#);
    body.push_str(&format!(
        r#"<li><span class="mining-results-timeline-label">Queued</span><span class="mining-results-timeline-value">{}</span></li><li><span class="mining-results-timeline-label">Mining end</span><span class="mining-results-timeline-value">{}</span></li></ul></section></div>"#,
        local_absolute_time_html(result.creation_time_millis),
        local_absolute_time_html(result.mining_end_time_millis)
    ));
}

fn action_name(action_type: i32) -> &'static str {
    match action_type {
        0 => "Scan",
        1 => "Wait on CPU",
        2 => "Move forward",
        3 => "Move backward",
        4 => "Rotate right",
        5 => "Rotate left",
        6 => "Mine",
        7 => "Dump",
        _ => "",
    }
}
