use super::super::render_filters::{mining_result_unique_areas, mining_result_wallet_deltas};
use super::super::selected_mining_queue_id;
use super::fixtures::mining_result;

#[test]
fn mining_result_unique_areas_are_sorted_and_deduped() {
    let results = vec![
        mining_result(10, "Beta", 1.0, 1),
        mining_result(11, "Alpha", 2.0, 2),
        mining_result(12, "Beta", 3.0, 3),
    ];

    assert_eq!(
        mining_result_unique_areas(&results),
        vec!["Alpha".to_string(), "Beta".to_string()]
    );
}

#[test]
fn mining_result_wallet_deltas_aggregate_net_ore_rewards() {
    let ore_results = vec![
        robominer_db::MiningResultOreStateRecord {
            mining_queue_id: 10,
            ore_id: 1,
            ore_name: "Iron".to_string(),
            amount: 10,
            tax: 1,
            reward: 9,
        },
        robominer_db::MiningResultOreStateRecord {
            mining_queue_id: 11,
            ore_id: 1,
            ore_name: "Iron".to_string(),
            amount: 5,
            tax: 0,
            reward: 5,
        },
        robominer_db::MiningResultOreStateRecord {
            mining_queue_id: 11,
            ore_id: 2,
            ore_name: "Copper".to_string(),
            amount: 3,
            tax: 0,
            reward: 3,
        },
    ];

    assert_eq!(
        mining_result_wallet_deltas(&ore_results),
        vec![("Copper".to_string(), 3), ("Iron".to_string(), 14)]
    );
}

#[test]
fn selected_mining_queue_id_prefers_valid_run_from_url() {
    let results = vec![
        mining_result(10, "A", 1.0, 1),
        mining_result(11, "B", 2.0, 2),
    ];

    assert_eq!(selected_mining_queue_id(&results, Some(11)), Some(11));
    assert_eq!(selected_mining_queue_id(&results, Some(99)), Some(10));
    assert_eq!(selected_mining_queue_id(&results, None), Some(10));
}
