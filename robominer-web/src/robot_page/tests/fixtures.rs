//! Shared fixtures for `robot_page` unit tests.

use std::collections::HashMap;

use crate::Request;
use crate::session::format_authenticated_cookie;

use super::super::RobotPageState;

pub(super) fn sample_robot_state(message: Option<String>) -> RobotPageState {
    RobotPageState {
        selected_robot_id: 7,
        program_sources: vec![robominer_db::ProgramSourceRecord {
            id: 11,
            user_id: 1,
            source_name: "Source <One>".to_string(),
            source_code: Some("move();".to_string()),
            verified: true,
            compiled_size: 12,
            error_description: String::new(),
        }],
        robots: vec![robominer_db::RobotConfigStateRecord {
            robot_id: 7,
            robot_name: "Bot <One>".to_string(),
            program_source_id: 11,
            ore_container_id: 101,
            ore_container_name: "Container & Current".to_string(),
            mining_unit_id: 201,
            mining_unit_name: "Mining Unit".to_string(),
            battery_id: 301,
            battery_name: "Battery".to_string(),
            memory_module_id: 401,
            memory_module_name: "Memory <Current>".to_string(),
            cpu_id: 501,
            cpu_name: "CPU".to_string(),
            engine_id: 601,
            engine_name: "Engine".to_string(),
            ore_scanner_id: 701,
            ore_scanner_name: "Ore Scanner".to_string(),
            recharge_time: 120,
            max_ore: 10,
            mining_speed: 2,
            max_turns: 50,
            memory_size: 20,
            cpu_speed: 3,
            forward_speed: 1.234,
            backward_speed: 2.345,
            rotate_speed: 90,
            robot_size: 1.987,
            scan_time: 6,
            scan_distance: 5,
            change_pending: false,
        }],
        part_assets: vec![
            robominer_db::RobotConfigPartAssetStateRecord {
                type_id: 1,
                robot_part_id: 102,
                part_name: "Container <Spare>".to_string(),
                memory_capacity: 0,
                unassigned: 1,
            },
            robominer_db::RobotConfigPartAssetStateRecord {
                type_id: 1,
                robot_part_id: 103,
                part_name: "Container Hidden".to_string(),
                memory_capacity: 0,
                unassigned: 0,
            },
            robominer_db::RobotConfigPartAssetStateRecord {
                type_id: 4,
                robot_part_id: 401,
                part_name: "Memory <Current>".to_string(),
                memory_capacity: 20,
                unassigned: 0,
            },
            robominer_db::RobotConfigPartAssetStateRecord {
                type_id: 4,
                robot_part_id: 402,
                part_name: "Memory & Spare".to_string(),
                memory_capacity: 30,
                unassigned: 1,
            },
        ],
        message,
        claimed_results: robominer_db::ClaimedUserResults {
            claimed_queues: 0,
            ore_rewards: vec![],
        },
    }
}

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
