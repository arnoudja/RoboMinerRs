use std::collections::HashMap;

use crate::html::escape_html;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum MiningAreaAtlasMode {
    StandalonePage,
}

pub(crate) fn render_mining_area_atlas(
    body: &mut String,
    mode: MiningAreaAtlasMode,
    ores: &[robominer_db::MiningAreaOverviewOreRecord],
    areas: &[robominer_db::MiningAreaOverviewAreaRecord],
    percentages: &[robominer_db::MiningAreaOverviewPercentageRecord],
    costs: &[robominer_db::MiningQueuePageAreaCostRecord],
    ore_assets: &[robominer_db::UserOreAssetStateRecord],
) {
    render_mining_area_atlas_markup(body, mode, ores, areas, percentages, costs, ore_assets);
    render_mining_area_atlas_script(body, mode);
}

pub(crate) fn render_mining_area_atlas_markup(
    body: &mut String,
    mode: MiningAreaAtlasMode,
    ores: &[robominer_db::MiningAreaOverviewOreRecord],
    areas: &[robominer_db::MiningAreaOverviewAreaRecord],
    percentages: &[robominer_db::MiningAreaOverviewPercentageRecord],
    costs: &[robominer_db::MiningQueuePageAreaCostRecord],
    ore_assets: &[robominer_db::UserOreAssetStateRecord],
) {
    let mut percentage_map = HashMap::new();
    for percentage in percentages {
        percentage_map.insert(
            (percentage.mining_area_id, percentage.ore_id),
            percentage.percentage,
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
        render_mining_area_atlas_controls(body, ores);
        render_mining_area_atlas_matrix(
            body,
            mode,
            ores,
            areas,
            &percentage_map,
            &cost_map,
            &ore_amount_map,
        );
    }

    body.push_str(
        r#"<p class="mining-area-atlas-footnote">Percentages reflect historic rally yields, not guaranteed results.</p>"#,
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
    percentage_map: &HashMap<(i64, i64), f64>,
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
            let percentage = percentage_map
                .get(&(area.mining_area_id, ore.ore_id))
                .copied()
                .unwrap_or(0.0);
            ore_yield_attrs.push_str(&format!(
                r#" data-ore-yield-{}="{}""#,
                ore.ore_id, percentage
            ));
        }
        body.push_str(&format!(
            r#"<tr class="mining-area-atlas-row" data-area-id="{}" data-area-name="{}" data-total-yield="{}" data-affordable="{}"{ore_yield_attrs}><th scope="row" class="mining-area-atlas-area-col">"#,
            area.mining_area_id,
            escape_html(&area.area_name),
            area.total_percentage,
            if affordable { "1" } else { "0" },
        ));
        render_mining_area_atlas_area_cell(body, mode, area);
        body.push_str(&format!(
            r#"</th><td class="mining-area-atlas-cost-cell">{cost_markup}</td><td class="{}">{:.1}%</td>"#,
            yield_cell_class(area.total_percentage),
            area.total_percentage
        ));
        for ore in ores {
            let percentage = percentage_map
                .get(&(area.mining_area_id, ore.ore_id))
                .copied()
                .unwrap_or(0.0);
            body.push_str(&format!(
                r#"<td class="{}">{:.1}%</td>"#,
                yield_cell_class(percentage),
                percentage
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

pub(crate) fn render_mining_area_atlas_script(body: &mut String, mode: MiningAreaAtlasMode) {
    let _ = mode;
    let sync_url = r#"        if (window.history && window.history.replaceState) {
            window.history.replaceState(null, '', query ? 'miningAreaOverview?' + query : 'miningAreaOverview');
        }"#;

    body.push_str(&format!(
        r#"<script>
(function() {{
    function miningAreaAtlasUrlParam(name) {{
        var search = window.location.search;
        if (!search) {{
            return null;
        }}
        var params = search.substring(1).split('&');
        for (var index = 0; index < params.length; index += 1) {{
            var pair = params[index].split('=');
            if (decodeURIComponent(pair[0]) === name && pair[1]) {{
                return decodeURIComponent(pair[1]);
            }}
        }}
        return null;
    }}

    function collectMiningAreaAtlasQueryParams() {{
        var params = [];
        var sortSelect = document.getElementById('miningAreaAtlasSort');
        var oreSelect = document.getElementById('miningAreaAtlasOreSort');
        var affordableOnly = document.getElementById('miningAreaAtlasAffordableOnly');
        if (sortSelect && sortSelect.value) {{
            params.push(encodeURIComponent('sort') + '=' + encodeURIComponent(sortSelect.value));
        }}
        if (sortSelect && sortSelect.value === 'ore' && oreSelect && oreSelect.value) {{
            params.push(encodeURIComponent('oreId') + '=' + encodeURIComponent(oreSelect.value));
        }}
        if (affordableOnly && affordableOnly.checked) {{
            params.push(encodeURIComponent('affordable') + '=1');
        }}
        return params.join('&');
    }}

    function syncMiningAreaAtlasUrl() {{
        var query = collectMiningAreaAtlasQueryParams();
{sync_url}
    }}

    function compareAtlasRows(left, right, sortBy, oreId) {{
        if (sortBy === 'name') {{
            return left.getAttribute('data-area-name').localeCompare(right.getAttribute('data-area-name'));
        }}
        if (sortBy === 'ore' && oreId) {{
            var leftYield = Number(left.getAttribute('data-ore-yield-' + oreId)) || 0;
            var rightYield = Number(right.getAttribute('data-ore-yield-' + oreId)) || 0;
            return rightYield - leftYield;
        }}
        var leftTotal = Number(left.getAttribute('data-total-yield')) || 0;
        var rightTotal = Number(right.getAttribute('data-total-yield')) || 0;
        return rightTotal - leftTotal;
    }}

    function updateOreSortVisibility() {{
        var sortSelect = document.getElementById('miningAreaAtlasSort');
        var oreField = document.getElementById('miningAreaAtlasOreField');
        if (!sortSelect || !oreField) {{
            return;
        }}
        oreField.hidden = sortSelect.value !== 'ore';
    }}

    function applyMiningAreaAtlasControls() {{
        var sortSelect = document.getElementById('miningAreaAtlasSort');
        var oreSelect = document.getElementById('miningAreaAtlasOreSort');
        var affordableOnly = document.getElementById('miningAreaAtlasAffordableOnly');
        var tbody = document.getElementById('miningAreaAtlasRows');
        if (!sortSelect || !tbody) {{
            return;
        }}
        updateOreSortVisibility();
        var sortBy = sortSelect.value || 'total';
        var oreId = oreSelect ? oreSelect.value : '';
        var rows = Array.prototype.slice.call(tbody.querySelectorAll('.mining-area-atlas-row'));
        rows.sort(function(left, right) {{
            return compareAtlasRows(left, right, sortBy, oreId);
        }});
        for (var rowIndex = 0; rowIndex < rows.length; rowIndex += 1) {{
            tbody.appendChild(rows[rowIndex]);
        }}
        var visibleCount = 0;
        for (var filterIndex = 0; filterIndex < rows.length; filterIndex += 1) {{
            var row = rows[filterIndex];
            var hide = affordableOnly && affordableOnly.checked && row.getAttribute('data-affordable') !== '1';
            row.classList.toggle('mining-area-atlas-filter-hidden', hide);
            if (!hide) {{
                visibleCount += 1;
            }}
        }}
        var empty = document.getElementById('miningAreaAtlasFilterEmpty');
        if (empty) {{
            empty.hidden = visibleCount > 0;
        }}
        syncMiningAreaAtlasUrl();
    }}

    var sortSelect = document.getElementById('miningAreaAtlasSort');
    var oreSelect = document.getElementById('miningAreaAtlasOreSort');
    var affordableOnly = document.getElementById('miningAreaAtlasAffordableOnly');
    if (sortSelect) {{
        var preferredSort = miningAreaAtlasUrlParam('sort');
        if (preferredSort) {{
            for (var sortIndex = 0; sortIndex < sortSelect.options.length; sortIndex += 1) {{
                if (sortSelect.options[sortIndex].value === preferredSort) {{
                    sortSelect.value = preferredSort;
                    break;
                }}
            }}
        }}
    }}
    if (oreSelect) {{
        var preferredOreId = miningAreaAtlasUrlParam('oreId');
        if (preferredOreId) {{
            for (var oreIndex = 0; oreIndex < oreSelect.options.length; oreIndex += 1) {{
                if (oreSelect.options[oreIndex].value === preferredOreId) {{
                    oreSelect.value = preferredOreId;
                    break;
                }}
            }}
        }}
    }}
    if (affordableOnly) {{
        affordableOnly.checked = miningAreaAtlasUrlParam('affordable') === '1';
    }}
    applyMiningAreaAtlasControls();
    if (sortSelect) {{
        sortSelect.addEventListener('change', applyMiningAreaAtlasControls);
    }}
    if (oreSelect) {{
        oreSelect.addEventListener('change', applyMiningAreaAtlasControls);
    }}
    if (affordableOnly) {{
        affordableOnly.addEventListener('change', applyMiningAreaAtlasControls);
    }}
}})();
</script>"#
    ));
}

pub(super) fn area_costs_affordable(
    costs: &[&robominer_db::MiningQueuePageAreaCostRecord],
    ore_amount_map: &HashMap<i64, i32>,
) -> bool {
    costs
        .iter()
        .all(|cost| ore_amount_map.get(&cost.ore_id).copied().unwrap_or(0) >= cost.amount)
}

pub(super) fn render_area_entry_costs(
    costs: &[&robominer_db::MiningQueuePageAreaCostRecord],
    ore_amount_map: &HashMap<i64, i32>,
) -> String {
    if costs.is_empty() {
        return r#"<span class="mining-area-atlas-cost-affordable">Free</span>"#.to_string();
    }
    costs
        .iter()
        .map(|cost| {
            let have = ore_amount_map.get(&cost.ore_id).copied().unwrap_or(0);
            if have >= cost.amount {
                format!(
                    r#"<span class="mining-area-atlas-cost-affordable">{} {} ✓</span>"#,
                    cost.amount,
                    escape_html(&cost.ore_name)
                )
            } else {
                let need = cost.amount - have;
                format!(
                    r#"<span class="mining-area-atlas-cost-unaffordable">Need {} more {}.</span>"#,
                    need,
                    escape_html(&cost.ore_name)
                )
            }
        })
        .collect::<Vec<_>>()
        .join("<br>")
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum MiningAreaAtlasLinkTarget {
    StandalonePage,
}

pub(crate) fn mining_area_atlas_url(
    _target: MiningAreaAtlasLinkTarget,
    ore_id: Option<i64>,
    affordable_only: bool,
) -> String {
    let mut params = Vec::new();
    if let Some(ore_id) = ore_id {
        params.push("sort=ore".to_string());
        params.push(format!("oreId={ore_id}"));
    }
    if affordable_only {
        params.push("affordable=1".to_string());
    }
    let base = "miningAreaOverview";
    if params.is_empty() {
        base.to_string()
    } else {
        format!("{base}?{}", params.join("&"))
    }
}

pub(crate) fn mining_area_atlas_url_for_ore(
    ore_id: i64,
    target: MiningAreaAtlasLinkTarget,
) -> String {
    mining_area_atlas_url(target, Some(ore_id), false)
}

pub(crate) fn mining_area_atlas_ore_link_label(ore_name: &str) -> String {
    format!("Areas rich in {ore_name}")
}

pub(crate) fn render_mining_area_atlas_ore_link(
    ore_id: i64,
    ore_name: &str,
    target: MiningAreaAtlasLinkTarget,
    class: &str,
) -> String {
    format!(
        r#"<a class="{class}" href="{}">{}</a>"#,
        escape_html(&mining_area_atlas_url_for_ore(ore_id, target)),
        escape_html(&mining_area_atlas_ore_link_label(ore_name)),
    )
}

pub(super) fn yield_cell_class(percentage: f64) -> &'static str {
    if percentage >= 20.0 {
        "mining-area-atlas-yield-high"
    } else if percentage >= 5.0 {
        "mining-area-atlas-yield-mid"
    } else if percentage > 0.0 {
        "mining-area-atlas-yield-low"
    } else {
        "mining-area-atlas-yield-zero"
    }
}

#[cfg(test)]
mod tests {
    use std::collections::HashMap;

    use super::{
        MiningAreaAtlasLinkTarget, MiningAreaAtlasMode, area_costs_affordable,
        mining_area_atlas_url, mining_area_atlas_url_for_ore, render_area_entry_costs,
        render_mining_area_atlas, render_mining_area_atlas_ore_link, yield_cell_class,
    };

    #[test]
    fn yield_cell_class_buckets_percentages() {
        assert_eq!(yield_cell_class(25.0), "mining-area-atlas-yield-high");
        assert_eq!(yield_cell_class(5.0), "mining-area-atlas-yield-mid");
        assert_eq!(yield_cell_class(1.0), "mining-area-atlas-yield-low");
        assert_eq!(yield_cell_class(0.0), "mining-area-atlas-yield-zero");
    }

    #[test]
    fn area_entry_cost_label_reports_affordability() {
        let cost = robominer_db::MiningQueuePageAreaCostRecord {
            mining_area_id: 10,
            ore_id: 2,
            ore_name: "Iron".to_string(),
            amount: 30,
        };
        let costs = vec![&cost];
        let affordable = HashMap::from([(2, 40)]);

        assert!(area_costs_affordable(&costs, &affordable));
        assert!(
            render_area_entry_costs(&costs, &affordable)
                .contains(r#"<span class="mining-area-atlas-cost-affordable">30 Iron ✓</span>"#)
        );

        let unaffordable = HashMap::from([(2, 10)]);
        assert!(!area_costs_affordable(&costs, &unaffordable));
        assert!(render_area_entry_costs(&costs, &unaffordable).contains(
            r#"<span class="mining-area-atlas-cost-unaffordable">Need 20 more Iron.</span>"#
        ));
    }

    #[test]
    fn render_area_entry_costs_colors_each_line_by_affordability() {
        let costs = vec![
            robominer_db::MiningQueuePageAreaCostRecord {
                mining_area_id: 10,
                ore_id: 1,
                ore_name: "Iron".to_string(),
                amount: 10,
            },
            robominer_db::MiningQueuePageAreaCostRecord {
                mining_area_id: 10,
                ore_id: 2,
                ore_name: "Gold".to_string(),
                amount: 20,
            },
        ];
        let cost_refs: Vec<_> = costs.iter().collect();
        let ore_amounts = HashMap::from([(1, 15), (2, 5)]);

        assert!(render_area_entry_costs(&cost_refs, &ore_amounts).contains(
            r#"<span class="mining-area-atlas-cost-affordable">10 Iron ✓</span><br><span class="mining-area-atlas-cost-unaffordable">Need 15 more Gold.</span>"#
        ));
    }

    #[test]
    fn mining_area_atlas_url_for_overview_and_ore_sort() {
        assert_eq!(
            mining_area_atlas_url(MiningAreaAtlasLinkTarget::StandalonePage, None, false),
            "miningAreaOverview"
        );
        assert_eq!(
            mining_area_atlas_url_for_ore(2, MiningAreaAtlasLinkTarget::StandalonePage),
            "miningAreaOverview?sort=ore&oreId=2"
        );
    }

    #[test]
    fn render_mining_area_atlas_ore_link_escapes_fields() {
        let link = render_mining_area_atlas_ore_link(
            2,
            "Ore & Two",
            MiningAreaAtlasLinkTarget::StandalonePage,
            "shop-atlas-link",
        );

        assert!(link.contains(r#"href="miningAreaOverview?sort=ore&amp;oreId=2""#));
        assert!(link.contains("Areas rich in Ore &amp; Two"));
    }

    #[test]
    fn render_mining_area_atlas_uses_area_links() {
        let mut body = String::new();
        render_mining_area_atlas(
            &mut body,
            MiningAreaAtlasMode::StandalonePage,
            &[robominer_db::MiningAreaOverviewOreRecord {
                ore_id: 1,
                ore_name: "Iron".to_string(),
            }],
            &[robominer_db::MiningAreaOverviewAreaRecord {
                mining_area_id: 10,
                area_name: "Area A".to_string(),
                total_percentage: 12.0,
            }],
            &[robominer_db::MiningAreaOverviewPercentageRecord {
                mining_area_id: 10,
                ore_id: 1,
                percentage: 12.0,
            }],
            &[],
            &[],
        );

        assert!(body.contains("mining-area-atlas-area-link"));
        assert!(!body.contains("mining-area-atlas-area-select"));
        assert!(body.contains(r#"href="miningQueue?infoMiningAreaId=10""#));
    }
}
