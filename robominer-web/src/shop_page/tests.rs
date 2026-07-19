use std::collections::HashMap;
use std::path::PathBuf;

use crate::session::format_authenticated_cookie;
use crate::{Request, ServerConfig};

use super::render::render_shop_page;
use super::{
    ShopPageState, default_shop_tier_id, robot_part_transaction_rejection_message, shop_page,
};

fn authenticated_request(path: &str) -> Request {
    Request {
        method: "GET".to_string(),
        path: path.to_string(),
        query: HashMap::new(),
        form: HashMap::new(),
        form_values: HashMap::new(),
        headers: HashMap::from([(
            "cookie".to_string(),
            format_authenticated_cookie(42, "Player"),
        )]),
    }
}

fn sample_shop_state(message: Option<String>) -> ShopPageState {
    ShopPageState {
        message,
        selected_part_type_id: 10,
        selected_tier_id: 2,
        selected_part_id: 100,
        ores: vec![
            robominer_db::OreRecord {
                id: 1,
                ore_name: "Ore <One>".to_string(),
            },
            robominer_db::OreRecord {
                id: 2,
                ore_name: "Ore & Two".to_string(),
            },
        ],
        part_types: vec![robominer_db::RobotPartTypeRecord {
            id: 10,
            type_name: "Type <A>".to_string(),
        }],
        parts: vec![robominer_db::ShopRobotPartCatalogRecord {
            robot_part_id: 100,
            type_id: 10,
            tier_id: 2,
            tier_name: "Ore & Two".to_string(),
            part_name: "Part <X> 'Q'".to_string(),
            ore_capacity: 5,
            mining_capacity: 6,
            battery_capacity: 7,
            memory_capacity: 8,
            cpu_capacity: 9,
            forward_capacity: 10,
            backward_capacity: 4,
            rotate_capacity: 90,
            recharge_time: 120,
            scan_time: 0,
            scan_distance: 0,
            weight: 11,
            volume: 12,
            power_usage: 13,
        }],
        costs: vec![robominer_db::ShopRobotPartCostRecord {
            robot_part_id: 100,
            ore_id: 2,
            ore_name: "Ore & Two".to_string(),
            amount: 30,
        }],
        part_states: vec![robominer_db::ShopRobotPartStateRecord {
            robot_part_id: 100,
            total_owned: 2,
            assigned: 1,
            unassigned: 1,
            can_buy: true,
            can_sell: true,
        }],
        ore_assets: vec![robominer_db::UserOreAssetStateRecord {
            ore_id: 2,
            ore_name: "Ore & Two".to_string(),
            amount: 40,
            max_allowed: 100,
            depot_max_allowed: 0,
        }],
    }
}

#[tokio::test(flavor = "current_thread")]
async fn shop_requires_database_configuration() {
    let config = ServerConfig {
        static_root: PathBuf::from("robominer-web/static"),
        database_pool: None,
        allow_signup: true,
        trust_proxy: false,
    };

    let response = shop_page(&authenticated_request("/shop"), &config).await;
    let body = String::from_utf8(response.body).expect("message should be utf-8");

    assert_eq!(response.status, 503);
    assert!(body.contains("ROBOMINER_DATABASE_URL"));
}

#[test]
fn shop_rendering_filters_selection_state_and_escapes_fields() {
    let html = render_shop_page(
        "Player".to_string(),
        None,
        &sample_shop_state(Some("Unable to buy <part>".to_string())),
    );

    assert!(!html.contains(r#"<script src="js/shop.js"></script>"#));
    assert!(html.contains(r#"class="shop-page" data-filter-storage-key="#));
    assert!(html.contains("function readStoredShopFilters()"));
    assert!(html.contains("function restoreShopFiltersFromStorage()"));
    assert!(html.contains("function writeStoredShopFilters()"));
    assert!(html.contains("window.sessionStorage.setItem(STORAGE_KEY"));
    assert!(html.contains(r#"class="shop-wallet""#));
    assert!(html.contains(r#"class="shop-deck""#));
    assert!(html.contains(r#"class="shop-detail""#));
    assert!(html.contains(r#"id="shopPartDetails100""#));
    assert!(html.contains(r#"class="shop-detail-panel shop-detail-panel-active""#));
    assert!(html.contains(r#"class="shop-banner shop-banner-error""#));
    assert!(html.contains("Unable to buy &lt;part&gt;"));
    assert!(html.contains("Type &lt;A&gt;"));
    assert!(html.contains("Ore &amp; Two quality"));
    assert!(html.contains("Part &lt;X&gt; &#39;Q&#39;"));
    assert!(html.contains(r#"id="robotPartTypeRow10_2_1""#));
    assert!(
        html.contains(
            r#"class="shop-part-card shop shop-part-card-compact shop-part-card-active""#
        )
    );
    assert!(html.contains(r#"<input type="hidden" name="buyRobotPartId" value="100"/>"#));
    assert!(html.contains(r#"<input type="hidden" name="sellRobotPartId" value="100"/>"#));
    assert!(html.contains(r#"<input type="hidden" name="selectedRobotPartId" value="100"/>"#));
    assert!(
        html.contains(
            r#"<button type="submit" class="shop-btn shop-btn-primary">Buy part</button>"#
        )
    );
    assert!(html.contains(
        r#"<button type="submit" class="shop-btn shop-btn-danger">Sell unassigned</button>"#
    ));
    assert!(html.contains(
        r#"<button type="submit" class="shop-btn shop-btn-danger">Sell all unassigned</button>"#
    ));
    assert!(
        html.contains(r#"class="shop-action-form shop-sell-all-form" data-unassigned-count="1""#)
    );
    assert!(html.contains("function confirmShopSell(event)"));
    assert!(html.contains("robominerConfirm('Sell 1 unassigned '"));
    assert!(html.contains("robominerConfirm(sellAllMessage"));
    assert!(html.contains(
        "if (form.getAttribute('data-robominer-confirmed') === '1') {\n            form.removeAttribute('data-robominer-confirmed');\n            return;\n        }\n        event.preventDefault();"
    ));
    assert!(html.contains(r#"class="shop-part-owned-badge">Owned: 2</span>"#));
    assert!(!html.contains(r#"<button type="submit">Show items</button>"#));
    assert!(html.contains("function applyShopFilters()"));
    assert!(html.contains("function selectShopPart(partId, updateUrl)"));
    assert!(html.contains("function collectShopQueryParams()"));
    assert!(html.contains("function syncShopFormState()"));
    assert!(html.contains("function shopUrlPartId()"));
    assert!(html.contains(r#"data-can-buy="1""#));
    assert!(html.contains(">2 minutes<"));
    assert!(html.contains(r#"class="sufficientbalance">(40)"#));
    assert!(html.contains(
        r#"name="selectedRobotPartTypeId" class="tableitem shop-filter-select"><option value="10" selected>"#
    ));
    assert!(html.contains(r#"<option value="2" selected>Ore &amp; Two quality</option>"#));
    assert!(
        html.contains(r#">Ore &amp; Two</span><span class="shop-wallet-amount">40/100</span>"#)
    );
    assert!(html.contains(
        r#"href="miningAreaOverview?sort=ore&amp;oreId=2">Areas rich in Ore &amp; Two</a>"#
    ));
    assert!(html.contains(r#"Compare all areas</a>.</p>"#));
    assert!(html.contains(r#"class="shop-atlas-helper""#));
    assert!(html.contains(r#"class="page-help-hint""#));
    assert!(html.contains(r#"href="helpMechanics">Read the mechanics guide</a>"#));
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

    assert!(html.contains(r#"<option value="1" selected>Cerbonium quality</option>"#));
    assert!(!html.contains("Ore &amp; Two quality</option>"));
}

#[test]
fn shop_shows_disabled_buy_and_sell_with_reasons() {
    let mut state = sample_shop_state(None);
    state.part_states = vec![robominer_db::ShopRobotPartStateRecord {
        robot_part_id: 100,
        total_owned: 2,
        assigned: 2,
        unassigned: 0,
        can_buy: false,
        can_sell: false,
    }];
    state.ore_assets = vec![robominer_db::UserOreAssetStateRecord {
        ore_id: 2,
        ore_name: "Iron".to_string(),
        amount: 10,
        max_allowed: 100,
        depot_max_allowed: 0,
    }];

    let html = render_shop_page("Player".to_string(), None, &state);

    assert!(html.contains(r#"name="buyRobotPartId" value="100""#));
    assert!(html.contains(r#"<button type="submit" class="shop-btn shop-btn-primary" disabled"#));
    assert!(html.contains("Need 20 more Ore &amp; Two."));
    assert!(html.contains(r#"Areas rich in Ore &amp; Two</a>"#));
    assert!(html.contains(r#"<button type="submit" class="shop-btn shop-btn-danger" disabled"#));
    assert!(html.contains("All units are assigned to robots."));
}

#[test]
fn shop_inventory_sorts_sellable_parts_first() {
    let mut state = sample_shop_state(None);
    state.parts = vec![
        robominer_db::ShopRobotPartCatalogRecord {
            robot_part_id: 100,
            type_id: 10,
            tier_id: 2,
            tier_name: "Ore & Two".to_string(),
            part_name: "Part Z".to_string(),
            ore_capacity: 5,
            mining_capacity: 6,
            battery_capacity: 7,
            memory_capacity: 8,
            cpu_capacity: 9,
            forward_capacity: 0,
            backward_capacity: 0,
            rotate_capacity: 0,
            recharge_time: 0,
            scan_time: 0,
            scan_distance: 0,
            weight: 11,
            volume: 12,
            power_usage: 13,
        },
        robominer_db::ShopRobotPartCatalogRecord {
            robot_part_id: 101,
            type_id: 10,
            tier_id: 2,
            tier_name: "Ore & Two".to_string(),
            part_name: "Part A".to_string(),
            ore_capacity: 5,
            mining_capacity: 6,
            battery_capacity: 7,
            memory_capacity: 8,
            cpu_capacity: 9,
            forward_capacity: 0,
            backward_capacity: 0,
            rotate_capacity: 0,
            recharge_time: 0,
            scan_time: 0,
            scan_distance: 0,
            weight: 11,
            volume: 12,
            power_usage: 13,
        },
    ];
    state.part_states = vec![
        robominer_db::ShopRobotPartStateRecord {
            robot_part_id: 100,
            total_owned: 1,
            assigned: 1,
            unassigned: 0,
            can_buy: false,
            can_sell: false,
        },
        robominer_db::ShopRobotPartStateRecord {
            robot_part_id: 101,
            total_owned: 2,
            assigned: 0,
            unassigned: 2,
            can_buy: false,
            can_sell: true,
        },
    ];

    let html = render_shop_page("Player".to_string(), None, &state);
    let part_a_pos = html
        .find(r#"<td class="shop-inventory-name">Part A</td>"#)
        .expect("Part A inventory row should appear");
    let part_z_pos = html
        .find(r#"<td class="shop-inventory-name">Part Z</td>"#)
        .expect("Part Z inventory row should appear");
    assert!(
        part_a_pos < part_z_pos,
        "sellable inventory rows should appear before assigned-only rows (A at {part_a_pos}, Z at {part_z_pos})"
    );
    assert!(
        html.contains(r#"class="shop-action-form shop-sell-all-form" data-unassigned-count="2""#)
    );
}

#[test]
fn shop_sell_all_unassigned_is_disabled_without_stock() {
    let mut state = sample_shop_state(None);
    state.part_states = vec![robominer_db::ShopRobotPartStateRecord {
        robot_part_id: 100,
        total_owned: 2,
        assigned: 2,
        unassigned: 0,
        can_buy: false,
        can_sell: false,
    }];

    let html = render_shop_page("Player".to_string(), None, &state);

    assert!(html.contains(r#"<button type="submit" class="shop-btn shop-btn-danger" disabled title="No unassigned robot parts to sell.">Sell all unassigned</button>"#));
    assert!(
        html.contains(r#"class="shop-action-form shop-sell-all-form" data-unassigned-count="0""#)
    );
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
    state.selected_part_type_id = super::ENGINE_PART_TYPE_ID;
    state.selected_tier_id = 1;
    state.part_types.push(robominer_db::RobotPartTypeRecord {
        id: super::ENGINE_PART_TYPE_ID,
        type_name: "Engine".to_string(),
    });
    state.parts = vec![robominer_db::ShopRobotPartCatalogRecord {
        robot_part_id: 601,
        type_id: super::ENGINE_PART_TYPE_ID,
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

    assert!(html.contains(
        r#"<span class="shop-part-highlight-label">Forward</span><span class="shop-part-highlight-value">18</span>"#
    ));
    assert!(html.contains("Engine power</dt><dd>18 forward, 8 backward, 75 rotate</dd>"));
}

#[test]
fn shop_memory_module_catalog_cards_show_memory_size() {
    let mut state = sample_shop_state(None);
    state.selected_part_type_id = super::MEMORY_MODULE_PART_TYPE_ID;
    state.selected_tier_id = 1;
    state.part_types.push(robominer_db::RobotPartTypeRecord {
        id: super::MEMORY_MODULE_PART_TYPE_ID,
        type_name: "Memory module".to_string(),
    });
    state.parts = vec![robominer_db::ShopRobotPartCatalogRecord {
        robot_part_id: 401,
        type_id: super::MEMORY_MODULE_PART_TYPE_ID,
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

    assert!(html.contains(
        r#"<span class="shop-part-highlight-label">Memory</span><span class="shop-part-highlight-value">16</span>"#
    ));
    assert!(html.contains("Memory size:</dt><dd>16</dd>"));
}

#[test]
fn shop_scanner_catalog_cards_show_scan_distance() {
    let mut state = sample_shop_state(None);
    state.selected_part_type_id = super::ORE_SCANNER_PART_TYPE_ID;
    state.selected_tier_id = 1;
    state.part_types.push(robominer_db::RobotPartTypeRecord {
        id: super::ORE_SCANNER_PART_TYPE_ID,
        type_name: "Ore scanner".to_string(),
    });
    state.parts = vec![robominer_db::ShopRobotPartCatalogRecord {
        robot_part_id: 701,
        type_id: super::ORE_SCANNER_PART_TYPE_ID,
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

    assert!(html.contains(
        r#"<span class="shop-part-highlight-label">Scan</span><span class="shop-part-highlight-value">50</span>"#
    ));
    assert!(html.contains("Scan time:</dt><dd>6 cycles"));
    assert!(!html.contains(r#"<span class="shop-part-highlight-value">6 cyc</span>"#));
}

#[test]
fn shop_transaction_rejection_messages_match_engine_output() {
    assert_eq!(
        robot_part_transaction_rejection_message(
            robominer_db::RobotPartTransactionRejection::InsufficientFunds
        ),
        "insufficient funds to pay robot part costs"
    );
    assert_eq!(
        robot_part_transaction_rejection_message(
            robominer_db::RobotPartTransactionRejection::NoUnassignedRobotPart
        ),
        "no unassigned robot part is available"
    );
}
