use std::collections::HashMap;

use crate::html::{escape_html, format_utc_millis};
use crate::mining_results_page::{
    MINING_RESULTS_INITIAL_VISIBLE, MINING_RESULTS_LOAD_MORE_STEP, MiningResultsPageState,
};

pub(super) fn render_mining_results_log_section(
    body: &mut String,
    state: &MiningResultsPageState,
    robot_names: &HashMap<i64, &str>,
    ore_result_map: &HashMap<i64, Vec<&robominer_db::MiningResultOreStateRecord>>,
) {
    body.push_str(
        r#"<section class="mining-results-log" aria-labelledby="mining-results-log-title">"#,
    );
    body.push_str(
        r#"<h2 id="mining-results-log-title" class="mining-results-section-title">Recent runs</h2><p class="mining-results-log-hint">Select a run to inspect payout and rally details.</p>"#,
    );
    body.push_str(&format!(
        r#"<div class="mining-results-run-cards" data-initial-visible="{MINING_RESULTS_INITIAL_VISIBLE}" data-load-more-step="{MINING_RESULTS_LOAD_MORE_STEP}">"#
    ));
    for result in &state.results {
        let ore_results = ore_result_map
            .get(&result.mining_queue_id)
            .map(Vec::as_slice)
            .unwrap_or(&[]);
        let robot_name = robot_names
            .get(&result.robot_id)
            .copied()
            .unwrap_or("Robot");
        render_mining_result_log_card(
            body,
            result,
            robot_name,
            ore_results,
            Some(result.mining_queue_id) == state.selected_mining_queue_id,
        );
    }
    body.push_str("</div>");
    body.push_str(
        r#"<p id="miningResultsLoadMoreWrap" class="mining-results-load-more-wrap" hidden><button type="button" id="miningResultsLoadMore" class="mining-results-load-more">Load more runs</button></p>"#,
    );
    body.push_str(
        r#"<p id="miningResultsFilterEmpty" class="mining-results-filter-empty" hidden>No runs match the current filters.</p>"#,
    );
    body.push_str("</section>");
}

fn render_mining_result_log_card(
    body: &mut String,
    result: &robominer_db::MiningResultStateRecord,
    robot_name: &str,
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
    body.push_str(r#"<span class="mining-results-run-heading-main">"#);
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
        r#"<span class="mining-results-run-robot">{}</span>"#,
        escape_html(robot_name)
    ));
    body.push_str("</span>");
    body.push_str(&format!(
        r#"<span class="mining-results-run-stats"><span class="mining-results-run-reward">+{} net</span><span class="mining-results-run-score">Score {:.1}</span><span class="mining-results-run-ended">Ended {}</span></span>"#,
        result.total_reward,
        result.score,
        escape_html(&format_utc_millis(result.mining_end_time_millis))
    ));
    body.push_str("</button>");
}

fn mining_result_ore_summary(ore_results: &[&robominer_db::MiningResultOreStateRecord]) -> String {
    if ore_results.is_empty() {
        return String::new();
    }
    let mut ordered: Vec<_> = ore_results.to_vec();
    ordered.sort_by(|left, right| {
        right
            .amount
            .cmp(&left.amount)
            .then_with(|| left.ore_name.cmp(&right.ore_name))
    });
    if ordered.len() == 1 {
        return ordered[0].ore_name.clone();
    }
    ordered
        .iter()
        .map(|ore_result| ore_result.ore_name.as_str())
        .collect::<Vec<_>>()
        .join(" · ")
}
