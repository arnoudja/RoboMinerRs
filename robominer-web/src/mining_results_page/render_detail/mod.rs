use std::collections::HashMap;

use crate::html::{EscapedHtml, format_utc_millis, html_attr};
use crate::mining_results_page::MiningResultsPageState;

mod breakdown;
mod score;

use breakdown::render_mining_result_breakdown;

pub(super) fn render_mining_results_detail_section(
    body: &mut String,
    state: &MiningResultsPageState,
    robot_names: &HashMap<i64, &str>,
    ore_result_map: &HashMap<i64, Vec<&robominer_db::MiningResultOreStateRecord>>,
    action_result_map: &HashMap<i64, Vec<&robominer_db::MiningResultActionStateRecord>>,
    area_ore_map: &HashMap<i64, Vec<&robominer_db::MiningResultAreaOreRecord>>,
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
        let area_ores = area_ore_map
            .get(&result.mining_area_id)
            .map(Vec::as_slice)
            .unwrap_or(&[]);
        let robot_name = robot_names
            .get(&result.robot_id)
            .copied()
            .unwrap_or("Robot");
        render_mining_result_detail_panel(
            body,
            result,
            robot_name,
            ore_results,
            action_results,
            area_ores,
            Some(result.mining_queue_id) == state.selected_mining_queue_id,
        );
    }
    body.push_str("</div></div>");
}

fn render_mining_result_detail_panel(
    body: &mut String,
    result: &robominer_db::MiningResultStateRecord,
    robot_name: &str,
    ore_results: &[&robominer_db::MiningResultOreStateRecord],
    action_results: &[&robominer_db::MiningResultActionStateRecord],
    area_ores: &[&robominer_db::MiningResultAreaOreRecord],
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
        result.robot_id,
        html_attr(&result.mining_area_name),
        result.mining_end_time_millis,
        result.total_reward,
        result.score
    ));
    body.push_str(r#"<header class="mining-results-detail-header">"#);
    body.push_str(&format!(
        r#"<div><h2 class="mining-results-detail-title">{}</h2><p class="mining-results-detail-subtitle">{} · Ended {} · Score {:.1}</p></div>"#,
        EscapedHtml::from(result.mining_area_name.as_str()),
        EscapedHtml::from(robot_name),
        EscapedHtml::from(format_utc_millis(result.mining_end_time_millis)),
        result.score
    ));
    body.push_str(&render_mining_result_replay_action(result));
    body.push_str("</header>");
    render_mining_result_breakdown(body, result, ore_results, action_results, area_ores);
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
