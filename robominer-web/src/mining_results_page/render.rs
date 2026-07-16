use std::collections::{HashMap, HashSet};

use crate::html::{escape_html, format_utc_millis, layout};
use crate::mining_area_atlas::{
    MiningAreaAtlasLinkTarget, mining_area_atlas_url, render_mining_area_atlas_ore_link,
};
use crate::mining_results_page::MiningResultsPageState;

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
        let unique_areas = mining_result_unique_areas(&state.results);
        render_mining_results_filters(&mut body, state, &result_map, &unique_areas);
        body.push_str(r#"<div class="mining-results-deck">"#);
        body.push_str(
            r#"<section class="mining-results-log" aria-labelledby="mining-results-log-title">"#,
        );
        body.push_str(
            r#"<h2 id="mining-results-log-title" class="mining-results-section-title">Recent runs</h2><p class="mining-results-log-hint">Select a run to inspect payout and rally details.</p>"#,
        );
        body.push_str(r#"<div class="mining-results-log-groups">"#);
        for robot in &state.robots {
            body.push_str(&format!(
                r#"<section class="mining-results-robot-group" data-robot-id="{}">"#,
                robot.robot_id
            ));
            body.push_str(&format!(
                r#"<h3 class="mining-results-robot-title">{}</h3>"#,
                escape_html(&robot.robot_name)
            ));
            if let Some(results) = result_map.get(&robot.robot_id) {
                body.push_str(r#"<div class="mining-results-run-cards">"#);
                for result in results.iter().copied() {
                    let ore_results = ore_result_map
                        .get(&result.mining_queue_id)
                        .map(Vec::as_slice)
                        .unwrap_or(&[]);
                    render_mining_result_log_card(
                        &mut body,
                        result,
                        ore_results,
                        Some(result.mining_queue_id) == state.selected_mining_queue_id,
                    );
                }
                body.push_str("</div>");
            } else {
                body.push_str(&format!(
                    r#"<p class="mining-results-robot-empty">No recent runs for {}.</p>"#,
                    escape_html(&robot.robot_name)
                ));
            }
            body.push_str("</section>");
        }
        body.push_str(
            r#"<p id="miningResultsFilterEmpty" class="mining-results-filter-empty" hidden>No runs match the current filters.</p>"#,
        );
        body.push_str("</div></section>");
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
                &mut body,
                result,
                result.robot_id,
                robot_name,
                ore_results,
                action_results,
                Some(result.mining_queue_id) == state.selected_mining_queue_id,
            );
        }
        body.push_str("</div></div></div>");
        body.push_str(super::scripts::MINING_RESULTS_PAGE_SCRIPT);
    }

    body.push_str("</div>");

    layout(
        "RoboMiner - Mining results",
        "miningResults",
        &username,
        hud,
        &body,
    )
}

fn render_mining_results_summary(body: &mut String) {
    body.push_str(r#"<section class="mining-results-summary" aria-label="Recent mining results">"#);
    body.push_str(r#"<div class="mining-results-summary-heading">"#);
    body.push_str(r#"<h1 class="mining-results-page-title">Mining results</h1>"#);
    body.push_str(r#"<p class="mining-results-capacity">Showing last completed runs</p>"#);
    body.push_str("</div></section>");
}

fn render_mining_results_claim_banner(body: &mut String, state: &MiningResultsPageState) {
    body.push_str(&crate::html::render_claimed_ore_rewards_banner(
        "mining-results-claim-banner",
        &state.claimed_results,
        false,
    ));
}

pub(super) fn mining_result_unique_areas(
    results: &[robominer_db::MiningResultStateRecord],
) -> Vec<String> {
    let mut areas: Vec<String> = results
        .iter()
        .map(|result| result.mining_area_name.clone())
        .collect::<HashSet<_>>()
        .into_iter()
        .collect();
    areas.sort();
    areas
}

pub(super) fn mining_result_wallet_deltas(
    ore_results: &[robominer_db::MiningResultOreStateRecord],
) -> Vec<(String, i32)> {
    let mut totals: HashMap<String, i32> = HashMap::new();
    for ore_result in ore_results {
        *totals.entry(ore_result.ore_name.clone()).or_default() += ore_result.reward;
    }
    let mut deltas: Vec<(String, i32)> = totals.into_iter().collect();
    deltas.sort_by(|left, right| left.0.cmp(&right.0));
    deltas
}

fn render_mining_results_wallet_delta(
    body: &mut String,
    ore_results: &[robominer_db::MiningResultOreStateRecord],
    show: bool,
) {
    if !show {
        return;
    }
    let deltas = mining_result_wallet_deltas(ore_results);
    if deltas.is_empty() {
        return;
    }
    body.push_str(
        r#"<section class="mining-results-wallet-delta" aria-label="Ore rewards from visible runs">"#,
    );
    body.push_str(r#"<span class="mining-results-wallet-delta-label">From these runs</span>"#);
    body.push_str(r#"<ul class="mining-results-wallet-delta-list">"#);
    for (ore_name, reward) in deltas {
        let ore_id = ore_results
            .iter()
            .find(|ore_result| ore_result.ore_name == ore_name)
            .map(|ore_result| ore_result.ore_id);
        let ore_label = if let Some(ore_id) = ore_id {
            render_mining_area_atlas_ore_link(
                ore_id,
                &ore_name,
                MiningAreaAtlasLinkTarget::StandalonePage,
                "mining-results-atlas-link",
            )
        } else {
            escape_html(&ore_name)
        };
        body.push_str(&format!(
            r#"<li class="mining-results-wallet-delta-item"><span class="mining-results-wallet-delta-ore">{}</span><span class="mining-results-wallet-delta-amount">+{}</span></li>"#,
            ore_label,
            reward
        ));
    }
    body.push_str("</ul></section>");
}

fn render_mining_results_filters(
    body: &mut String,
    state: &MiningResultsPageState,
    _result_map: &HashMap<i64, Vec<&robominer_db::MiningResultStateRecord>>,
    unique_areas: &[String],
) {
    body.push_str(r#"<section class="mining-results-filters" aria-label="Result filters">"#);
    body.push_str(&format!(
        r#"<p class="mining-results-atlas-helper">Find stronger yields in the <a class="mining-results-atlas-link" href="{}">area atlas</a>.</p>"#,
        escape_html(&mining_area_atlas_url(
            MiningAreaAtlasLinkTarget::StandalonePage,
            None,
            false,
        )),
    ));
    body.push_str(r#"<div class="mining-results-filter-form">"#);
    body.push_str(
        r#"<label class="mining-results-filter-label" for="miningResultsRobotFilter">Robot <select id="miningResultsRobotFilter" class="tableitem mining-results-filter-select">"#,
    );
    body.push_str(r#"<option value="">All robots</option>"#);
    for robot in &state.robots {
        body.push_str(&format!(
            r#"<option value="{}">{}</option>"#,
            robot.robot_id,
            escape_html(&robot.robot_name)
        ));
    }
    body.push_str("</select></label>");
    body.push_str(
        r#"<label class="mining-results-filter-label" for="miningResultsAreaFilter">Area <select id="miningResultsAreaFilter" class="tableitem mining-results-filter-select">"#,
    );
    body.push_str(r#"<option value="">All areas</option>"#);
    for area_name in unique_areas {
        body.push_str(&format!(
            r#"<option value="{}">{}</option>"#,
            escape_html(area_name),
            escape_html(area_name)
        ));
    }
    body.push_str("</select></label>");
    body.push_str(
        r#"<label class="mining-results-filter-label" for="miningResultsSortFilter">Sort <select id="miningResultsSortFilter" class="tableitem mining-results-filter-select"><option value="newest" selected>Newest first</option><option value="reward">Highest reward</option><option value="score">Highest score</option></select></label>"#,
    );
    body.push_str("</div></section>");
}

fn render_mining_result_log_card(
    body: &mut String,
    result: &robominer_db::MiningResultStateRecord,
    ore_results: &[&robominer_db::MiningResultOreStateRecord],
    active: bool,
) {
    let active_class = if active {
        " mining-results-run-card-active"
    } else {
        ""
    };
    let ore_summary = mining_result_ore_summary(ore_results);

    body.push_str(&format!(
        r#"<button type="button" class="mining-results-run-card{active_class}" data-run-id="{}" data-robot-id="{}" data-area-name="{}" data-sort-end="{}" data-sort-reward="{}" data-sort-score="{}">"#,
        result.mining_queue_id,
        result.robot_id,
        escape_html(&result.mining_area_name),
        result.mining_end_time_millis,
        result.total_reward,
        result.score
    ));
    body.push_str(r#"<span class="mining-results-run-heading">"#);
    body.push_str(&format!(
        r#"<span class="mining-results-run-area">{}</span>"#,
        escape_html(&result.mining_area_name)
    ));
    if !ore_summary.is_empty() {
        body.push_str(&format!(
            r#"<span class="mining-results-run-ores">{}</span>"#,
            escape_html(&ore_summary)
        ));
    }
    body.push_str("</span>");
    body.push_str(&format!(
        r#"<span class="mining-results-run-stats"><span class="mining-results-run-reward">+{} net</span><span class="mining-results-run-score">Score {:.1}</span><span class="mining-results-run-ended">Ended {}</span></span>"#,
        result.total_reward,
        result.score,
        escape_html(&format_utc_millis(result.mining_end_time_millis))
    ));
    body.push_str("</button>");
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

fn mining_result_ore_summary(ore_results: &[&robominer_db::MiningResultOreStateRecord]) -> String {
    if ore_results.is_empty() {
        return String::new();
    }
    if ore_results.len() == 1 {
        return ore_results[0].ore_name.clone();
    }
    ore_results
        .iter()
        .map(|ore_result| ore_result.ore_name.as_str())
        .collect::<Vec<_>>()
        .join(" · ")
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
