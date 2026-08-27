use crate::html::{assert_contains_all, assert_html_contains};

use super::super::render::render_shop_page;
use super::fixtures::sample_shop_state;

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

    assert_contains_all(
        &html,
        &[
            r#"name="buyRobotPartId" value="100""#,
            r#"<button type="submit" class="shop-btn shop-btn-primary" disabled"#,
            "Need 20 more Ore &amp; Two.",
            r#"Areas rich in Ore &amp; Two</a>"#,
            r#"<button type="submit" class="shop-btn shop-btn-danger" disabled"#,
            "All units are assigned to robots.",
        ],
    );
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
    assert_html_contains(
        &html,
        r#"class="shop-action-form shop-sell-all-form" data-unassigned-count="2""#,
    );
    assert_html_contains(
        &html,
        r#"<div class="shop-inventory-table-wrap"><table class="shop-inventory-table">"#,
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

    assert_contains_all(
        &html,
        &[
            r#"<button type="submit" class="shop-btn shop-btn-danger" disabled title="No unassigned robot parts to sell.">Sell all unassigned</button>"#,
            r#"class="shop-action-form shop-sell-all-form" data-unassigned-count="0""#,
        ],
    );
}

#[test]
fn shop_transaction_rejection_messages_match_engine_output() {
    assert_eq!(
        robominer_domain::rejection_messages::robot_part_transaction_rejection_message(
            robominer_db::RobotPartTransactionRejection::InsufficientFunds
        ),
        "insufficient funds to pay robot part costs"
    );
    assert_eq!(
        robominer_domain::rejection_messages::robot_part_transaction_rejection_message(
            robominer_db::RobotPartTransactionRejection::NoUnassignedRobotPart
        ),
        "no unassigned robot part is available"
    );
}
