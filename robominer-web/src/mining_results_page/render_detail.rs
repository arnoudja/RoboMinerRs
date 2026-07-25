use std::collections::HashMap;

use crate::html::{escape_html, format_utc_millis};
use crate::mining_results_page::MiningResultsPageState;

pub(super) fn render_mining_results_detail_section(
    body: &mut String,
    state: &MiningResultsPageState,
    robot_names: &HashMap<i64, &str>,
    ore_result_map: &HashMap<i64, Vec<&robominer_db::MiningResultOreStateRecord>>,
    action_result_map: &HashMap<i64, Vec<&robominer_db::MiningResultActionStateRecord>>,
) {
    body.push_str(
        r#"<div class="mining-results-detail-area"><div class="mining-results-detail-panels">"#,
    );
    for result in &state.results {
        let ore_results = ore_result_map
            .get(&result.mining_queue_id)
            .map(Vec::as_slice)
            .unwrap_or(&[]);
        let action_results = action_result_map
            .get(&result.mining_queue_id)
            .map(Vec::as_slice)
            .unwrap_or(&[]);
        let robot_name = robot_names
            .get(&result.robot_id)
            .copied()
            .unwrap_or("Robot");
        render_mining_result_detail_panel(
            body,
            result,
            result.robot_id,
            robot_name,
            ore_results,
            action_results,
            Some(result.mining_queue_id) == state.selected_mining_queue_id,
        );
    }
    body.push_str("</div></div>");
}

fn render_mining_result_detail_panel(
    body: &mut String,
    result: &robominer_db::MiningResultStateRecord,
    robot_id: i64,
    robot_name: &str,
    ore_results: &[&robominer_db::MiningResultOreStateRecord],
    action_results: &[&robominer_db::MiningResultActionStateRecord],
    active: bool,
) {
    let active_class = if active {
        " mining-results-detail-panel-active"
    } else {
        ""
    };
    let hidden_attr = if active { "" } else { " hidden" };

    body.push_str(&format!(
        r#"<div class="mining-results-detail-panel{active_class}" id="miningResultDetails{}" data-run-id="{}" data-robot-id="{}" data-area-name="{}" data-sort-end="{}" data-sort-reward="{}" data-sort-score="{}"{hidden_attr}>"#,
        result.mining_queue_id,
        result.mining_queue_id,
        robot_id,
        escape_html(&result.mining_area_name),
        result.mining_end_time_millis,
        result.total_reward,
        result.score
    ));
    body.push_str(r#"<header class="mining-results-detail-header">"#);
    body.push_str(&format!(
        r#"<div><h2 class="mining-results-detail-title">{}</h2><p class="mining-results-detail-subtitle">{} · Ended {} · Score {:.1}</p></div>"#,
        escape_html(&result.mining_area_name),
        escape_html(robot_name),
        escape_html(&format_utc_millis(result.mining_end_time_millis)),
        result.score
    ));
    body.push_str(&render_mining_result_replay_action(result));
    body.push_str("</header>");
    render_mining_result_breakdown(body, result, ore_results, action_results);
    body.push_str("</div>");
}

fn render_mining_result_replay_action(result: &robominer_db::MiningResultStateRecord) -> String {
    if let Some(rally_result_id) = result.rally_result_id {
        return format!(
            r#"<a class="mining-results-replay-link mining-results-replay-link-primary" href="miningResults?rallyResultId={rally_result_id}" data-rally-result-id="{rally_result_id}">Replay rally</a>"#
        );
    }
    r#"<span class="mining-results-replay-disabled" title="No animation stored for this run.">Replay unavailable</span>"#
        .to_string()
}

fn render_mining_result_breakdown(
    body: &mut String,
    result: &robominer_db::MiningResultStateRecord,
    ore_results: &[&robominer_db::MiningResultOreStateRecord],
    action_results: &[&robominer_db::MiningResultActionStateRecord],
) {
    body.push_str(r#"<div class="mining-results-run-breakdown">"#);
    body.push_str(r#"<section class="mining-results-breakdown-section"><h3 class="mining-results-breakdown-title">Payout</h3><dl class="mining-results-payout-list">"#);
    body.push_str(&format!(
        r#"<div class="mining-results-payout-item"><dt>Mined</dt><dd>{}</dd></div><div class="mining-results-payout-item"><dt><span class="mining-results-tax-label" title="Tax is deducted before ore is added to your wallet.">Tax</span></dt><dd>{}</dd></div><div class="mining-results-payout-item"><dt>Net</dt><dd class="mining-results-payout-net">+{}</dd></div><div class="mining-results-payout-item"><dt>Score</dt><dd>{:.1}</dd></div>"#,
        result.total_ore_mined,
        result.total_tax,
        result.total_reward,
        result.score
    ));
    body.push_str("</dl></section>");

    if !ore_results.is_empty() {
        body.push_str(r#"<section class="mining-results-breakdown-section"><h3 class="mining-results-breakdown-title">Ore breakdown</h3><ul class="mining-results-ore-list">"#);
        for ore_result in ore_results {
            body.push_str(&format!(
                r#"<li><span class="mining-results-ore-name">{}</span><span class="mining-results-ore-values">{} mined · {} tax · +{} net</span></li>"#,
                escape_html(&ore_result.ore_name),
                ore_result.amount,
                ore_result.tax,
                ore_result.reward,
            ));
        }
        body.push_str("</ul></section>");
    }

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
        format_utc_millis(result.creation_time_millis),
        format_utc_millis(result.mining_end_time_millis)
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
