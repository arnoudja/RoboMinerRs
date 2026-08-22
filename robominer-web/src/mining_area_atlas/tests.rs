use std::collections::HashMap;

use crate::html::{assert_contains_all, assert_html_contains, assert_html_not_contains};

use super::costs::{area_costs_affordable, render_area_entry_costs};
use super::links::{
    MiningAreaAtlasLinkTarget, mining_area_atlas_url, mining_area_atlas_url_for_ore,
    render_mining_area_atlas_ore_link,
};
use super::markup::{MiningAreaAtlasMode, render_mining_area_atlas, yield_cell_class};

#[test]
fn yield_cell_class_buckets_averages() {
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
    assert_html_contains(
        &render_area_entry_costs(&costs, &affordable),
        r#"<span class="mining-area-atlas-cost-affordable">30 Iron ✓</span>"#,
    );

    let unaffordable = HashMap::from([(2, 10)]);
    assert!(!area_costs_affordable(&costs, &unaffordable));
    assert_html_contains(
        &render_area_entry_costs(&costs, &unaffordable),
        r#"<span class="mining-area-atlas-cost-unaffordable">Need 20 more Iron.</span>"#,
    );
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

    assert_html_contains(
        &render_area_entry_costs(&cost_refs, &ore_amounts),
        r#"<span class="mining-area-atlas-cost-affordable">10 Iron ✓</span><br><span class="mining-area-atlas-cost-unaffordable">Need 15 more Gold.</span>"#,
    );
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

    assert_contains_all(
        &link,
        &[
            r#"href="miningAreaOverview?sort=ore&amp;oreId=2""#,
            "Areas rich in Ore &amp; Two",
        ],
    );
}

#[test]
fn render_mining_area_atlas_orders_ore_columns_by_descending_ore_id() {
    let mut body = String::new();
    render_mining_area_atlas(
        &mut body,
        MiningAreaAtlasMode::StandalonePage,
        &[
            robominer_db::MiningAreaOverviewOreRecord {
                ore_id: 1,
                ore_name: "Iron".to_string(),
            },
            robominer_db::MiningAreaOverviewOreRecord {
                ore_id: 3,
                ore_name: "Gold".to_string(),
            },
            robominer_db::MiningAreaOverviewOreRecord {
                ore_id: 2,
                ore_name: "Silver".to_string(),
            },
        ],
        &[robominer_db::MiningAreaOverviewAreaRecord {
            mining_area_id: 10,
            area_name: "Area A".to_string(),
            total_average_ore_per_run: 12.0,
        }],
        &[],
        &[],
        &[],
    );

    let header = body
        .split("<tbody")
        .next()
        .expect("table header should precede body rows");
    let gold_pos = header.find("Gold").expect("Gold column header");
    let silver_pos = header.find("Silver").expect("Silver column header");
    let iron_pos = header.find("Iron").expect("Iron column header");
    assert!(gold_pos < silver_pos);
    assert!(silver_pos < iron_pos);
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
            total_average_ore_per_run: 12.0,
        }],
        &[robominer_db::MiningAreaOverviewOreAverageRecord {
            mining_area_id: 10,
            ore_id: 1,
            average_ore_per_run: 12.0,
        }],
        &[],
        &[],
    );

    assert_contains_all(
        &body,
        &[
            "mining-area-atlas-area-link",
            r#"href="miningQueue?infoMiningAreaId=10""#,
        ],
    );
    assert_html_not_contains(&body, "mining-area-atlas-area-select");
}
