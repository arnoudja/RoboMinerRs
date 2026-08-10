//! Shared fixtures for `shop_page` unit tests.

use std::collections::HashMap;

use crate::Request;
use crate::session::format_authenticated_cookie;

use super::super::ShopPageState;

pub(super) fn authenticated_request(path: &str) -> Request {
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

pub(super) fn sample_shop_state(message: Option<String>) -> ShopPageState {
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
            depot_max_allowed: 250,
        }],
    }
}
