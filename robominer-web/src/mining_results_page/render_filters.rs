use std::collections::{HashMap, HashSet};

use crate::html::{EscapedHtml, html_attr};
use crate::mining_area_atlas::{
    MiningAreaAtlasLinkTarget, mining_area_atlas_url, render_mining_area_atlas_ore_link,
};
use crate::mining_results_page::MiningResultsPageState;

pub(super) fn render_mining_results_summary(body: &mut String) {
    body.push_str(r#"<section class="mining-results-summary" aria-label="Recent mining results">"#);
    body.push_str(r#"<div class="mining-results-summary-heading">"#);
    body.push_str(r#"<h1 class="mining-results-page-title">Mining results</h1>"#);
    body.push_str(r#"<p class="mining-results-capacity">Showing last completed runs</p>"#);
    body.push_str("</div></section>");
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

pub(super) fn render_mining_results_wallet_delta(
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
            EscapedHtml::from(ore_name.as_str()).to_string()
        };
        body.push_str(&format!(
            r#"<li class="mining-results-wallet-delta-item"><span class="mining-results-wallet-delta-ore">{}</span><span class="mining-results-wallet-delta-amount">+{}</span></li>"#,
            ore_label,
            reward
        ));
    }
    body.push_str("</ul></section>");
}

pub(super) fn render_mining_results_filters(body: &mut String, state: &MiningResultsPageState) {
    let unique_areas = mining_result_unique_areas(&state.results);

    body.push_str(r#"<section class="mining-results-filters" aria-label="Result filters">"#);
    body.push_str(&format!(
        r#"<p class="mining-results-atlas-helper">Find stronger yields in the <a class="mining-results-atlas-link" href="{}">area atlas</a>.</p>"#,
        html_attr(&mining_area_atlas_url(
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
            EscapedHtml::from(robot.robot_name.as_str())
        ));
    }
    body.push_str("</select></label>");
    body.push_str(
        r#"<label class="mining-results-filter-label" for="miningResultsAreaFilter">Area <select id="miningResultsAreaFilter" class="tableitem mining-results-filter-select">"#,
    );
    body.push_str(r#"<option value="">All areas</option>"#);
    for area_name in &unique_areas {
        body.push_str(&format!(
            r#"<option value="{}">{}</option>"#,
            html_attr(area_name),
            EscapedHtml::from(area_name.as_str())
        ));
    }
    body.push_str("</select></label>");
    body.push_str(
        r#"<label class="mining-results-filter-label" for="miningResultsSortFilter">Sort <select id="miningResultsSortFilter" class="tableitem mining-results-filter-select"><option value="newest" selected>Newest first</option><option value="reward">Highest reward</option><option value="score">Highest score</option></select></label>"#,
    );
    body.push_str("</div></section>");
}
