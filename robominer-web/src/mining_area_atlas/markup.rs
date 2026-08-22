use std::collections::HashMap;

use crate::html::escape_html;

use super::costs::{area_costs_affordable, render_area_entry_costs};
use super::script::render_mining_area_atlas_script;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum MiningAreaAtlasMode {
    StandalonePage,
}

pub(crate) fn render_mining_area_atlas(
    body: &mut String,
    mode: MiningAreaAtlasMode,
    ores: &[robominer_db::MiningAreaOverviewOreRecord],
    areas: &[robominer_db::MiningAreaOverviewAreaRecord],
    ore_averages: &[robominer_db::MiningAreaOverviewOreAverageRecord],
    costs: &[robominer_db::MiningQueuePageAreaCostRecord],
    ore_assets: &[robominer_db::UserOreAssetStateRecord],
) {
    render_mining_area_atlas_markup(body, mode, ores, areas, ore_averages, costs, ore_assets);
    render_mining_area_atlas_script(body);
}

fn sort_ores_by_id_descending(
    ores: &[robominer_db::MiningAreaOverviewOreRecord],
) -> Vec<robominer_db::MiningAreaOverviewOreRecord> {
    let mut sorted = ores.to_vec();
    sorted.sort_by_key(|ore| std::cmp::Reverse(ore.ore_id));
    sorted
}

pub(crate) fn render_mining_area_atlas_markup(
    body: &mut String,
    mode: MiningAreaAtlasMode,
    ores: &[robominer_db::MiningAreaOverviewOreRecord],
    areas: &[robominer_db::MiningAreaOverviewAreaRecord],
    ore_averages: &[robominer_db::MiningAreaOverviewOreAverageRecord],
    costs: &[robominer_db::MiningQueuePageAreaCostRecord],
    ore_assets: &[robominer_db::UserOreAssetStateRecord],
) {
    let ores = sort_ores_by_id_descending(ores);

    let mut average_map = HashMap::new();
    for average in ore_averages {
        average_map.insert(
            (average.mining_area_id, average.ore_id),
            average.average_ore_per_run,
        );
    }

    let mut cost_map: HashMap<i64, Vec<&robominer_db::MiningQueuePageAreaCostRecord>> =
        HashMap::new();
    for cost in costs {
        cost_map.entry(cost.mining_area_id).or_default().push(cost);
    }

    let ore_amount_map: HashMap<i64, i32> = ore_assets
        .iter()
        .map(|asset| (asset.ore_id, asset.amount))
        .collect();

    if mode == MiningAreaAtlasMode::StandalonePage {
        render_mining_area_atlas_header(body, mode);
    }

    if areas.is_empty() {
        body.push_str(
            r#"<p class="mining-area-atlas-empty">No mining areas are available yet.</p>"#,
        );
    } else {
        render_mining_area_atlas_controls(body, &ores);
        render_mining_area_atlas_matrix(
            body,
            mode,
            &ores,
            areas,
            &average_map,
            &cost_map,
            &ore_amount_map,
        );
    }

    body.push_str(
        r#"<p class="mining-area-atlas-footnote">Averages reflect historic ore mined per claimed run, not guaranteed results.</p>"#,
    );
}

fn render_mining_area_atlas_header(body: &mut String, _mode: MiningAreaAtlasMode) {
    body.push_str(r#"<header class="mining-area-atlas-header">"#);
    body.push_str(r#"<div class="mining-area-atlas-heading">"#);
    body.push_str(r#"<h1 class="mining-area-atlas-title">Mining area atlas</h1>"#);
    body.push_str(
        r#"<p class="mining-area-atlas-subtitle">Compare historic ore yields and entry costs across all areas.</p>"#,
    );
    body.push_str("</div>");
    body.push_str(r#"<div class="mining-area-atlas-header-actions">"#);
    body.push_str(r#"<a class="mining-area-atlas-back-link" href="miningQueue">Back to queue</a>"#);
    body.push_str("</div></header>");
}

fn render_mining_area_atlas_controls(
    body: &mut String,
    ores: &[robominer_db::MiningAreaOverviewOreRecord],
) {
    body.push_str(r#"<section class="mining-area-atlas-controls" aria-label="Atlas filters">"#);
    body.push_str(r#"<div class="mining-area-atlas-control-form">"#);
    body.push_str(
        r#"<label class="mining-area-atlas-control-label" for="miningAreaAtlasSort">Sort <select id="miningAreaAtlasSort" class="tableitem mining-area-atlas-control-select"><option value="total" selected>Highest total yield</option><option value="ore">Highest ore yield</option><option value="name">Area name</option></select></label>"#,
    );
    body.push_str(
        r#"<label class="mining-area-atlas-control-label" id="miningAreaAtlasOreField" for="miningAreaAtlasOreSort" hidden>Ore <select id="miningAreaAtlasOreSort" class="tableitem mining-area-atlas-control-select">"#,
    );
    for ore in ores {
        body.push_str(&format!(
            r#"<option value="{}">{}</option>"#,
            ore.ore_id,
            escape_html(&ore.ore_name)
        ));
    }
    body.push_str("</select></label>");
    body.push_str(
        r#"<label class="mining-area-atlas-control-checkbox"><input type="checkbox" id="miningAreaAtlasAffordableOnly" /> Affordable only</label>"#,
    );
    body.push_str("</div></section>");
}

fn render_mining_area_atlas_matrix(
    body: &mut String,
    mode: MiningAreaAtlasMode,
    ores: &[robominer_db::MiningAreaOverviewOreRecord],
    areas: &[robominer_db::MiningAreaOverviewAreaRecord],
    average_map: &HashMap<(i64, i64), f64>,
    cost_map: &HashMap<i64, Vec<&robominer_db::MiningQueuePageAreaCostRecord>>,
    ore_amount_map: &HashMap<i64, i32>,
) {
    body.push_str(
        r#"<section class="mining-area-atlas-matrix" aria-label="Area yield comparison">"#,
    );
    body.push_str(r#"<div class="mining-area-atlas-table-wrap">"#);
    body.push_str(r#"<table class="mining-area-atlas-table">"#);
    body.push_str(
        r#"<thead><tr><th scope="col" class="mining-area-atlas-area-col">Area</th><th scope="col">Entry cost</th><th scope="col">Total</th>"#,
    );
    for ore in ores {
        body.push_str(&format!(
            r#"<th scope="col">{}</th>"#,
            escape_html(&ore.ore_name)
        ));
    }
    body.push_str("</tr></thead><tbody id=\"miningAreaAtlasRows\">");

    for area in areas {
        let costs = cost_map
            .get(&area.mining_area_id)
            .map(Vec::as_slice)
            .unwrap_or(&[]);
        let affordable = area_costs_affordable(costs, ore_amount_map);
        let cost_markup = render_area_entry_costs(costs, ore_amount_map);
        let mut ore_yield_attrs = String::new();
        for ore in ores {
            let average = average_map
                .get(&(area.mining_area_id, ore.ore_id))
                .copied()
                .unwrap_or(0.0);
            ore_yield_attrs.push_str(&format!(r#" data-ore-yield-{}="{}""#, ore.ore_id, average));
        }
        body.push_str(&format!(
            r#"<tr class="mining-area-atlas-row" data-area-id="{}" data-area-name="{}" data-total-yield="{}" data-affordable="{}"{ore_yield_attrs}><th scope="row" class="mining-area-atlas-area-col">"#,
            area.mining_area_id,
            escape_html(&area.area_name),
            area.total_average_ore_per_run,
            if affordable { "1" } else { "0" },
        ));
        render_mining_area_atlas_area_cell(body, mode, area);
        body.push_str(&format!(
            r#"</th><td class="mining-area-atlas-cost-cell">{cost_markup}</td><td class="{}">{:.1}</td>"#,
            yield_cell_class(area.total_average_ore_per_run),
            area.total_average_ore_per_run
        ));
        for ore in ores {
            let average = average_map
                .get(&(area.mining_area_id, ore.ore_id))
                .copied()
                .unwrap_or(0.0);
            body.push_str(&format!(
                r#"<td class="{}">{:.1}</td>"#,
                yield_cell_class(average),
                average
            ));
        }
        body.push_str("</tr>");
    }

    body.push_str("</tbody></table></div>");
    body.push_str(
        r#"<p id="miningAreaAtlasFilterEmpty" class="mining-area-atlas-filter-empty" hidden>No areas match the current filters.</p>"#,
    );
    body.push_str("</section>");
}

fn render_mining_area_atlas_area_cell(
    body: &mut String,
    _mode: MiningAreaAtlasMode,
    area: &robominer_db::MiningAreaOverviewAreaRecord,
) {
    body.push_str(&format!(
        r#"<a class="mining-area-atlas-area-link" href="miningQueue?infoMiningAreaId={}">{}</a>"#,
        area.mining_area_id,
        escape_html(&area.area_name)
    ));
}

pub(crate) fn yield_cell_class(average_ore_per_run: f64) -> &'static str {
    if average_ore_per_run >= 20.0 {
        "mining-area-atlas-yield-high"
    } else if average_ore_per_run >= 5.0 {
        "mining-area-atlas-yield-mid"
    } else if average_ore_per_run > 0.0 {
        "mining-area-atlas-yield-low"
    } else {
        "mining-area-atlas-yield-zero"
    }
}
