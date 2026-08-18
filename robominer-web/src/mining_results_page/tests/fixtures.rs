//! Shared fixtures for `mining_results_page` unit tests.

use std::collections::HashMap;

use crate::Request;
use crate::session::format_authenticated_cookie;

use super::super::MiningResultsPageState;

pub(super) fn mining_result(
    mining_queue_id: i64,
    mining_area_name: &str,
    score: f64,
    total_ore_mined: i32,
) -> robominer_db::MiningResultStateRecord {
    robominer_db::MiningResultStateRecord {
        robot_id: 1,
        mining_queue_id,
        mining_area_id: 1,
        mining_area_name: mining_area_name.to_string(),
        score_ore_target: 30,
        rally_result_id: None,
        score,
        total_ore_mined,
        total_tax: 0,
        total_reward: total_ore_mined,
        creation_time_millis: 0,
        mining_end_time_millis: 0,
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

pub(super) fn sample_mining_results_state() -> MiningResultsPageState {
    MiningResultsPageState {
        robots: vec![
            robominer_db::MiningQueuePageRobotRecord {
                robot_id: 1,
                robot_name: "Bot <One>".to_string(),
                recharge_time: 60,
            },
            robominer_db::MiningQueuePageRobotRecord {
                robot_id: 2,
                robot_name: "Bot & Two".to_string(),
                recharge_time: 60,
            },
        ],
        results: vec![robominer_db::MiningResultStateRecord {
            robot_id: 1,
            mining_queue_id: 10,
            mining_area_id: 1,
            mining_area_name: "Area & One".to_string(),
            score_ore_target: 30,
            rally_result_id: Some(99),
            score: 610.0,
            total_ore_mined: 30,
            total_tax: 3,
            total_reward: 27,
            creation_time_millis: 0,
            mining_end_time_millis: 1_000,
        }],
        ore_results: vec![
            robominer_db::MiningResultOreStateRecord {
                mining_queue_id: 10,
                ore_id: 1,
                ore_name: "Ore <A>".to_string(),
                amount: 10,
                tax: 1,
                reward: 9,
            },
            robominer_db::MiningResultOreStateRecord {
                mining_queue_id: 10,
                ore_id: 2,
                ore_name: "Ore & B".to_string(),
                amount: 20,
                tax: 2,
                reward: 18,
            },
        ],
        action_results: vec![
            robominer_db::MiningResultActionStateRecord {
                mining_queue_id: 10,
                action_type: 0,
                amount: 2,
            },
            robominer_db::MiningResultActionStateRecord {
                mining_queue_id: 10,
                action_type: 2,
                amount: 3,
            },
            robominer_db::MiningResultActionStateRecord {
                mining_queue_id: 10,
                action_type: 6,
                amount: 1,
            },
        ],
        area_ore_slots: vec![
            robominer_db::MiningResultAreaOreSlotRecord {
                mining_area_id: 1,
                ore_id: 2,
                ore_name: "Ore & B".to_string(),
            },
            robominer_db::MiningResultAreaOreSlotRecord {
                mining_area_id: 1,
                ore_id: 1,
                ore_name: "Ore <A>".to_string(),
            },
        ],
        claimed_results: robominer_db::ClaimedUserResults {
            claimed_queues: 0,
            ore_rewards: vec![],
        },
        selected_mining_queue_id: Some(10),
    }
}
