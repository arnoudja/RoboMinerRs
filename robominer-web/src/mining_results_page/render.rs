use std::collections::HashMap;

use crate::html::layout;
use crate::mining_results_page::MiningResultsPageState;
use crate::static_assets::PageStylesheet;

use super::render_detail::render_mining_results_detail_section;
use super::render_filters::{
    render_mining_results_claim_banner, render_mining_results_filters,
    render_mining_results_summary, render_mining_results_wallet_delta,
};
use super::render_log::render_mining_results_log_section;

pub(super) fn render_mining_results_page(
    username: String,
    hud: Option<&str>,
    state: &MiningResultsPageState,
) -> String {
    let mut result_map: HashMap<i64, Vec<&robominer_db::MiningResultStateRecord>> = HashMap::new();
    for result in &state.results {
        result_map.entry(result.robot_id).or_default().push(result);
    }

    let robot_names: HashMap<i64, &str> = state
        .robots
        .iter()
        .map(|robot| (robot.robot_id, robot.robot_name.as_str()))
        .collect();

    let mut ore_result_map: HashMap<i64, Vec<&robominer_db::MiningResultOreStateRecord>> =
        HashMap::new();
    for ore_result in &state.ore_results {
        ore_result_map
            .entry(ore_result.mining_queue_id)
            .or_default()
            .push(ore_result);
    }

    let mut action_result_map: HashMap<i64, Vec<&robominer_db::MiningResultActionStateRecord>> =
        HashMap::new();
    for action_result in &state.action_results {
        action_result_map
            .entry(action_result.mining_queue_id)
            .or_default()
            .push(action_result);
    }

    let mut body = String::from(r#"<div class="mining-results-page">"#);
    render_mining_results_summary(&mut body);
    render_mining_results_wallet_delta(&mut body, &state.ore_results, !state.results.is_empty());
    render_mining_results_claim_banner(&mut body, state);

    if state.results.is_empty() {
        body.push_str(
            r#"<p class="mining-results-empty">No recent mining results. <a href="miningQueue">Check the mining queue</a> to schedule runs.</p>"#,
        );
    } else {
        render_mining_results_filters(&mut body, state);
        body.push_str(r#"<div class="mining-results-deck">"#);
        render_mining_results_log_section(&mut body, state, &result_map, &ore_result_map);
        render_mining_results_detail_section(
            &mut body,
            state,
            &robot_names,
            &ore_result_map,
            &action_result_map,
        );
        body.push_str("</div>");
        body.push_str(&super::scripts::mining_results_page_script_tag());
    }

    body.push_str("</div>");

    layout(
        "RoboMiner - Mining results",
        "miningResults",
        &username,
        hud,
        &body,
        &[PageStylesheet::MiningResults],
    )
}
