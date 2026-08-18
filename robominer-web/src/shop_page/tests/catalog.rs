use crate::html::{assert_contains_all, assert_html_contains, assert_html_not_contains};

use super::super::default_shop_tier_id;
use super::super::render::render_shop_page;
use super::fixtures::sample_shop_state;

#[test]
fn shop_rendering_filters_selection_state_and_escapes_fields() {
    let html = render_shop_page(
        "Player".to_string(),
        None,
        &sample_shop_state(Some("Unable to buy <part>".to_string())),
    );

    assert_contains_all(
        &html,
        &[
            r#"class="shop-page" data-filter-storage-key="#,
            r#"src="js/shop/page.js?v="#,
            r#"class="page-wallet shop-wallet""#,
            r#"class="shop-deck""#,
            r#"class="shop-detail""#,
            r#"id="shopPartDetails100""#,
            r#"class="shop-detail-panel shop-detail-panel-active""#,
            r#"class="shop-banner shop-banner-error""#,
            "Unable to buy &lt;part&gt;",
            "Type &lt;A&gt;",
            "Ore &amp; Two quality",
            "Part &lt;X&gt; &#39;Q&#39;",
            r#"id="robotPartTypeRow10_2_1""#,
            r#"class="shop-part-card shop shop-part-card-compact shop-part-card-active""#,
            r#"<input type="hidden" name="buyRobotPartId" value="100"/>"#,
            r#"<input type="hidden" name="sellRobotPartId" value="100"/>"#,
            r#"<input type="hidden" name="selectedRobotPartId" value="100"/>"#,
            r#"<button type="submit" class="shop-btn shop-btn-primary">Buy part</button>"#,
            r#"<button type="submit" class="shop-btn shop-btn-danger">Sell unassigned</button>"#,
            r#"<button type="submit" class="shop-btn shop-btn-danger">Sell all unassigned</button>"#,
            r#"class="shop-action-form shop-sell-all-form" data-unassigned-count="1""#,
            r#"class="shop-part-owned-badge">Owned: 2</span>"#,
            r#"data-can-buy="1""#,
            ">2 minutes<",
            r#"class="sufficientbalance">(40)"#,
            r#"name="selectedRobotPartTypeId" class="tableitem shop-filter-select"><option value="10" selected>"#,
            r#"<option value="2" selected>Ore &amp; Two quality</option>"#,
            r#">Ore &amp; Two</span><span class="page-wallet-amount">40/100</span>"#,
            r#"class="page-wallet-depot">depot 250</span>"#,
            r#"href="miningAreaOverview?sort=ore&amp;oreId=2">Areas rich in Ore &amp; Two</a>"#,
            r#"Compare all areas</a>.</p>"#,
            r#"class="shop-atlas-helper""#,
            r#"class="page-help-hint""#,
            r#"href="helpMechanics">Read the mechanics guide</a>"#,
        ],
    );
    for absent in [
        r#"<script src="js/shop.js"></script>"#,
        "function confirmShopSell(event)",
        r#"<button type="submit">Show items</button>"#,
    ] {
        assert_html_not_contains(&html, absent);
    }
}

#[test]
fn default_shop_tier_id_selects_highest_quality_ore() {
    let ores = vec![
        robominer_db::OreRecord {
            id: 1,
            ore_name: "Cerbonium".to_string(),
        },
        robominer_db::OreRecord {
            id: 3,
            ore_name: "Lithabine".to_string(),
        },
    ];

    assert_eq!(default_shop_tier_id(&ores), Some(3));
    assert_eq!(default_shop_tier_id(&[]), None);
}

#[test]
fn shop_quality_filter_lists_only_mineable_ores() {
    let mut state = sample_shop_state(None);
    state.ores = vec![robominer_db::OreRecord {
        id: 1,
        ore_name: "Cerbonium".to_string(),
    }];
    state.selected_tier_id = 1;

    let html = render_shop_page("Player".to_string(), None, &state);

    assert_html_contains(
        &html,
        r#"<option value="1" selected>Cerbonium quality</option>"#,
    );
    assert_html_not_contains(&html, "Ore &amp; Two quality</option>");
}

#[test]
fn shop_part_costs_are_sorted_by_ore_id_descending() {
    let mut state = sample_shop_state(None);
    state.costs = vec![
        robominer_db::ShopRobotPartCostRecord {
            robot_part_id: 100,
            ore_id: 1,
            ore_name: "Cerbonium".to_string(),
            amount: 10,
        },
        robominer_db::ShopRobotPartCostRecord {
            robot_part_id: 100,
            ore_id: 3,
            ore_name: "Lithabine".to_string(),
            amount: 20,
        },
        robominer_db::ShopRobotPartCostRecord {
            robot_part_id: 100,
            ore_id: 2,
            ore_name: "Iron".to_string(),
            amount: 30,
        },
    ];
    state.ore_assets = vec![
        robominer_db::UserOreAssetStateRecord {
            ore_id: 1,
            ore_name: "Cerbonium".to_string(),
            amount: 100,
            max_allowed: 100,
            depot_max_allowed: 0,
        },
        robominer_db::UserOreAssetStateRecord {
            ore_id: 2,
            ore_name: "Iron".to_string(),
            amount: 100,
            max_allowed: 100,
            depot_max_allowed: 0,
        },
        robominer_db::UserOreAssetStateRecord {
            ore_id: 3,
            ore_name: "Lithabine".to_string(),
            amount: 100,
            max_allowed: 100,
            depot_max_allowed: 0,
        },
    ];

    let html = render_shop_page("Player".to_string(), None, &state);
    let list_start = html
        .find(r#"<ul class="shop-part-cost-list">"#)
        .expect("part cost list should render");
    let list_end = list_start
        + html[list_start..]
            .find("</ul>")
            .expect("part cost list should close");
    let list = &html[list_start..list_end];

    let lithabine = list
        .find("Lithabine")
        .expect("Lithabine cost should render");
    let iron = list.find("Iron").expect("Iron cost should render");
    let cerbonium = list
        .find("Cerbonium")
        .expect("Cerbonium cost should render");
    assert!(lithabine < iron);
    assert!(iron < cerbonium);
}

#[test]
fn shop_engine_catalog_cards_show_forward_power() {
    let mut state = sample_shop_state(None);
    state.selected_part_type_id = super::super::ENGINE_PART_TYPE_ID;
    state.selected_tier_id = 1;
    state.part_types.push(robominer_db::RobotPartTypeRecord {
        id: super::super::ENGINE_PART_TYPE_ID,
        type_name: "Engine".to_string(),
    });
    state.parts = vec![robominer_db::ShopRobotPartCatalogRecord {
        robot_part_id: 601,
        type_id: super::super::ENGINE_PART_TYPE_ID,
        tier_id: 1,
        tier_name: "Cerbonium".to_string(),
        part_name: "Standard Engine".to_string(),
        ore_capacity: 0,
        mining_capacity: 0,
        battery_capacity: 0,
        memory_capacity: 0,
        cpu_capacity: 0,
        forward_capacity: 18,
        backward_capacity: 8,
        rotate_capacity: 75,
        recharge_time: 0,
        scan_time: 0,
        scan_distance: 0,
        weight: 4,
        volume: 4,
        power_usage: 6,
    }];
    state.costs = vec![robominer_db::ShopRobotPartCostRecord {
        robot_part_id: 601,
        ore_id: 1,
        ore_name: "Cerbonium".to_string(),
        amount: 10,
    }];
    state.part_states = vec![robominer_db::ShopRobotPartStateRecord {
        robot_part_id: 601,
        total_owned: 0,
        assigned: 0,
        unassigned: 0,
        can_buy: true,
        can_sell: false,
    }];
    state.selected_part_id = 601;

    let html = render_shop_page("Player".to_string(), None, &state);

    assert_contains_all(
        &html,
        &[
            r#"<span class="shop-part-highlight-label">Forward</span><span class="shop-part-highlight-value">18</span>"#,
            "Engine power</dt><dd>18 forward, 8 backward, 75 rotate</dd>",
        ],
    );
}

#[test]
fn shop_memory_module_catalog_cards_show_memory_size() {
    let mut state = sample_shop_state(None);
    state.selected_part_type_id = super::super::MEMORY_MODULE_PART_TYPE_ID;
    state.selected_tier_id = 1;
    state.part_types.push(robominer_db::RobotPartTypeRecord {
        id: super::super::MEMORY_MODULE_PART_TYPE_ID,
        type_name: "Memory module".to_string(),
    });
    state.parts = vec![robominer_db::ShopRobotPartCatalogRecord {
        robot_part_id: 401,
        type_id: super::super::MEMORY_MODULE_PART_TYPE_ID,
        tier_id: 1,
        tier_name: "Cerbonium".to_string(),
        part_name: "Standard Memory Module".to_string(),
        ore_capacity: 0,
        mining_capacity: 0,
        battery_capacity: 0,
        memory_capacity: 16,
        cpu_capacity: 0,
        forward_capacity: 0,
        backward_capacity: 0,
        rotate_capacity: 0,
        recharge_time: 0,
        scan_time: 0,
        scan_distance: 0,
        weight: 1,
        volume: 1,
        power_usage: 1,
    }];
    state.costs = vec![robominer_db::ShopRobotPartCostRecord {
        robot_part_id: 401,
        ore_id: 1,
        ore_name: "Cerbonium".to_string(),
        amount: 10,
    }];
    state.part_states = vec![robominer_db::ShopRobotPartStateRecord {
        robot_part_id: 401,
        total_owned: 0,
        assigned: 0,
        unassigned: 0,
        can_buy: true,
        can_sell: false,
    }];
    state.selected_part_id = 401;

    let html = render_shop_page("Player".to_string(), None, &state);

    assert_contains_all(
        &html,
        &[
            r#"<span class="shop-part-highlight-label">Memory</span><span class="shop-part-highlight-value">16</span>"#,
            "Memory size:</dt><dd>16</dd>",
        ],
    );
}

#[test]
fn shop_scanner_catalog_cards_show_scan_distance() {
    let mut state = sample_shop_state(None);
    state.selected_part_type_id = super::super::ORE_SCANNER_PART_TYPE_ID;
    state.selected_tier_id = 1;
    state.part_types.push(robominer_db::RobotPartTypeRecord {
        id: super::super::ORE_SCANNER_PART_TYPE_ID,
        type_name: "Ore scanner".to_string(),
    });
    state.parts = vec![robominer_db::ShopRobotPartCatalogRecord {
        robot_part_id: 701,
        type_id: super::super::ORE_SCANNER_PART_TYPE_ID,
        tier_id: 1,
        tier_name: "Cerbonium".to_string(),
        part_name: "Standard Ore Scanner".to_string(),
        ore_capacity: 0,
        mining_capacity: 0,
        battery_capacity: 0,
        memory_capacity: 0,
        cpu_capacity: 0,
        forward_capacity: 0,
        backward_capacity: 0,
        rotate_capacity: 0,
        recharge_time: 0,
        scan_time: 6,
        scan_distance: 50,
        weight: 2,
        volume: 2,
        power_usage: 1,
    }];
    state.costs = vec![robominer_db::ShopRobotPartCostRecord {
        robot_part_id: 701,
        ore_id: 1,
        ore_name: "Cerbonium".to_string(),
        amount: 10,
    }];
    state.part_states = vec![robominer_db::ShopRobotPartStateRecord {
        robot_part_id: 701,
        total_owned: 0,
        assigned: 0,
        unassigned: 0,
        can_buy: true,
        can_sell: false,
    }];
    state.selected_part_id = 701;

    let html = render_shop_page("Player".to_string(), None, &state);

    assert_contains_all(
        &html,
        &[
            r#"<span class="shop-part-highlight-label">Scan</span><span class="shop-part-highlight-value">50</span>"#,
            "Scan time:</dt><dd>6 instructions",
        ],
    );
    assert_html_not_contains(
        &html,
        r#"<span class="shop-part-highlight-value">6 cyc</span>"#,
    );
}
