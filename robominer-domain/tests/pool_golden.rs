#![allow(clippy::unwrap_used, clippy::expect_used)]
#[path = "support/pool.rs"]
mod pool;

use pool::{PoolScenario, PoolScenarioId};
use robominer_domain::{run_pool_loadout_with_seed, simulation::completed_pool_rally_record};
use robominer_test_support::{assert_or_update_golden, round_golden_score};
use serde::{Deserialize, Serialize};

const MANIFEST_DIR: &str = env!("CARGO_MANIFEST_DIR");
const FIXTURE_SUBDIR: &str = "pool";
const UPDATE_ENV_VAR: &str = "UPDATE_POOL_GOLDEN";

#[derive(Debug, Serialize, Deserialize, PartialEq)]
struct GoldenPoolOreResult {
    ore_id: i64,
    amount: i32,
}

#[derive(Debug, Serialize, Deserialize, PartialEq)]
struct GoldenPoolItem {
    player_number: usize,
    pool_item_id: i64,
    robot_id: i64,
    score: f64,
    ore_results: Vec<GoldenPoolOreResult>,
}

#[derive(Debug, Serialize, Deserialize, PartialEq)]
struct GoldenCompletedPoolItem {
    pool_item_id: i64,
    score: f64,
    ore_results: Vec<GoldenPoolOreResult>,
}

#[derive(Debug, Serialize, Deserialize, PartialEq)]
struct GoldenPoolFixture {
    scenario: String,
    seed: u64,
    pool_id: i64,
    mining_area_id: i64,
    final_time: i32,
    items: Vec<GoldenPoolItem>,
    completed_items: Vec<GoldenCompletedPoolItem>,
}

fn build_fixture(scenario: &PoolScenario) -> GoldenPoolFixture {
    let outcome = run_pool_loadout_with_seed(&scenario.loadout, scenario.seed)
        .expect("golden pool scenario should run");
    let record = completed_pool_rally_record(&scenario.loadout, &outcome)
        .expect("golden pool completed record should map");

    GoldenPoolFixture {
        scenario: scenario.name.to_string(),
        seed: scenario.seed,
        pool_id: outcome.pool_id,
        mining_area_id: outcome.mining_area_id,
        final_time: outcome.final_time,
        items: outcome
            .items
            .iter()
            .map(|item| GoldenPoolItem {
                player_number: item.player_number,
                pool_item_id: item.pool_item_id,
                robot_id: item.robot_id,
                score: round_golden_score(item.score),
                ore_results: item
                    .ore_results
                    .iter()
                    .map(|ore| GoldenPoolOreResult {
                        ore_id: ore.ore_id,
                        amount: ore.amount,
                    })
                    .collect(),
            })
            .collect(),
        completed_items: record
            .items
            .iter()
            .map(|item| GoldenCompletedPoolItem {
                pool_item_id: item.pool_item_id,
                score: round_golden_score(item.score),
                ore_results: item
                    .ore_results
                    .iter()
                    .map(|ore| GoldenPoolOreResult {
                        ore_id: ore.ore_id,
                        amount: ore.amount,
                    })
                    .collect(),
            })
            .collect(),
    }
}

#[test]
fn pool_outcomes_match_golden_fixtures() {
    assert_or_update_golden(
        UPDATE_ENV_VAR,
        MANIFEST_DIR,
        FIXTURE_SUBDIR,
        PoolScenarioId::ALL,
        |id| id.as_str(),
        |id| (build_fixture(&id.build()), ()),
        |id, expected, actual, ()| {
            assert_eq!(
                expected,
                actual,
                "scenario {} pool outcome mismatch",
                id.as_str()
            );
        },
    );
}
