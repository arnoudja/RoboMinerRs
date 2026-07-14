mod support;

use robominer_db::{MySqlPool, claim_user_results};
use robominer_test_support::{load_fixture, update_golden_enabled, write_fixture};
use serde::{Deserialize, Serialize};
use sqlx::Row;
use support::{ClaimScenario, ore_index, queue_index};

const MANIFEST_DIR: &str = env!("CARGO_MANIFEST_DIR");
const FIXTURE_SUBDIR: &str = "claim";
const UPDATE_ENV_VAR: &str = "UPDATE_CLAIM_GOLDEN";

const SCENARIOS: &[&str] = &[
    "single_queue_tax25",
    "dual_queue_batch_claim",
    "skips_unfinished_queue",
    "claim_cap_limited",
    "claim_zero_tax",
    "claim_multiple_ore_types",
];

#[derive(Debug, Serialize, Deserialize, PartialEq)]
struct GoldenOreResult {
    ore_index: usize,
    amount: i32,
    tax: i32,
}

#[derive(Debug, Serialize, Deserialize, PartialEq)]
struct GoldenQueueState {
    queue_index: usize,
    claimed: bool,
    ore_results: Vec<GoldenOreResult>,
}

#[derive(Debug, Serialize, Deserialize, PartialEq)]
struct GoldenUserOreAsset {
    ore_index: usize,
    amount: i32,
    max_allowed: i32,
}

#[derive(Debug, Serialize, Deserialize, PartialEq)]
struct GoldenRobotLifetimeResult {
    ore_index: usize,
    amount: i32,
    tax: i32,
}

#[derive(Debug, Serialize, Deserialize, PartialEq)]
struct GoldenMiningAreaLifetimeResult {
    ore_index: usize,
    total_amount: i64,
    total_container_size: i64,
}

#[derive(Debug, Serialize, Deserialize, PartialEq)]
struct GoldenClaimFixture {
    scenario: String,
    claimed_queues: u64,
    queues: Vec<GoldenQueueState>,
    user_ore_assets: Vec<GoldenUserOreAsset>,
    robot_total_mining_runs: i32,
    robot_lifetime_results: Vec<GoldenRobotLifetimeResult>,
    mining_area_lifetime_results: Vec<GoldenMiningAreaLifetimeResult>,
    pending_robot_changes_remaining: i64,
}

async fn build_fixture(pool: &MySqlPool, scenario: &ClaimScenario) -> GoldenClaimFixture {
    let claim = claim_user_results(pool, scenario.user_id)
        .await
        .expect("golden scenario claim should succeed");

    GoldenClaimFixture {
        scenario: scenario.name.to_string(),
        claimed_queues: claim.claimed_queues,
        queues: load_queue_states(pool, scenario).await,
        user_ore_assets: load_user_ore_assets(pool, scenario).await,
        robot_total_mining_runs: load_robot_total_mining_runs(pool, scenario.robot_id).await,
        robot_lifetime_results: load_robot_lifetime_results(pool, scenario).await,
        mining_area_lifetime_results: load_mining_area_lifetime_results(pool, scenario).await,
        pending_robot_changes_remaining: load_pending_robot_changes_count(pool, scenario.robot_id)
            .await,
    }
}

async fn load_queue_states(pool: &MySqlPool, scenario: &ClaimScenario) -> Vec<GoldenQueueState> {
    let mut queues = Vec::new();

    for queue_id in &scenario.queue_ids {
        let claimed: i8 = sqlx::query_scalar("SELECT claimed FROM MiningQueue WHERE id = ?")
            .bind(queue_id)
            .fetch_one(pool)
            .await
            .expect("failed to load queue claimed flag");

        let ore_rows = sqlx::query(
            "SELECT oreId, amount, COALESCE(tax, 0) AS tax \
             FROM MiningOreResult \
             WHERE miningQueueId = ? \
             ORDER BY oreId",
        )
        .bind(queue_id)
        .fetch_all(pool)
        .await
        .expect("failed to load queue ore results");

        queues.push(GoldenQueueState {
            queue_index: queue_index(scenario, *queue_id),
            claimed: claimed != 0,
            ore_results: ore_rows
                .into_iter()
                .map(|row| GoldenOreResult {
                    ore_index: ore_index(scenario, row.get("oreId")),
                    amount: row.get("amount"),
                    tax: row.get("tax"),
                })
                .collect(),
        });
    }

    queues.sort_by_key(|queue| queue.queue_index);
    queues
}

async fn load_user_ore_assets(
    pool: &MySqlPool,
    scenario: &ClaimScenario,
) -> Vec<GoldenUserOreAsset> {
    let rows = sqlx::query(
        "SELECT oreId, amount, maxAllowed \
         FROM UserOreAsset \
         WHERE userId = ? \
         ORDER BY oreId",
    )
    .bind(scenario.user_id)
    .fetch_all(pool)
    .await
    .expect("failed to load user ore assets");

    rows.into_iter()
        .map(|row| GoldenUserOreAsset {
            ore_index: ore_index(scenario, row.get("oreId")),
            amount: row.get("amount"),
            max_allowed: row.get("maxAllowed"),
        })
        .collect()
}

async fn load_robot_total_mining_runs(pool: &MySqlPool, robot_id: i64) -> i32 {
    sqlx::query_scalar("SELECT totalMiningRuns FROM Robot WHERE id = ?")
        .bind(robot_id)
        .fetch_one(pool)
        .await
        .expect("failed to load robot total mining runs")
}

async fn load_robot_lifetime_results(
    pool: &MySqlPool,
    scenario: &ClaimScenario,
) -> Vec<GoldenRobotLifetimeResult> {
    let rows = sqlx::query(
        "SELECT oreId, amount, tax \
         FROM RobotLifetimeResult \
         WHERE robotId = ? \
         ORDER BY oreId",
    )
    .bind(scenario.robot_id)
    .fetch_all(pool)
    .await
    .expect("failed to load robot lifetime results");

    rows.into_iter()
        .map(|row| GoldenRobotLifetimeResult {
            ore_index: ore_index(scenario, row.get("oreId")),
            amount: row.get("amount"),
            tax: row.get("tax"),
        })
        .collect()
}

async fn load_mining_area_lifetime_results(
    pool: &MySqlPool,
    scenario: &ClaimScenario,
) -> Vec<GoldenMiningAreaLifetimeResult> {
    let mut results = Vec::new();

    for (ore_index, ore_id) in scenario.ore_ids.iter().enumerate() {
        let row = sqlx::query(
            "SELECT totalAmount, totalContainerSize \
             FROM MiningAreaLifetimeResult \
             WHERE miningAreaId = ? AND oreId = ?",
        )
        .bind(scenario.mining_area_id)
        .bind(ore_id)
        .fetch_one(pool)
        .await
        .expect("failed to load mining area lifetime result");

        results.push(GoldenMiningAreaLifetimeResult {
            ore_index,
            total_amount: row.get("totalAmount"),
            total_container_size: row.get("totalContainerSize"),
        });
    }

    results
}

async fn load_pending_robot_changes_count(pool: &MySqlPool, robot_id: i64) -> i64 {
    sqlx::query_scalar("SELECT COUNT(*) FROM PendingRobotChanges WHERE robotId = ?")
        .bind(robot_id)
        .fetch_one(pool)
        .await
        .expect("failed to count pending robot changes")
}

#[tokio::test]
async fn claim_outcomes_match_golden_fixtures() {
    let Ok(database_url) = std::env::var("ROBOMINER_DATABASE_URL") else {
        eprintln!("skipping claim golden test: ROBOMINER_DATABASE_URL is not set");
        return;
    };

    let pool = robominer_db::connect(&database_url)
        .await
        .expect("failed to connect to test database");

    if update_golden_enabled(UPDATE_ENV_VAR) {
        for name in SCENARIOS {
            support::scenario(name);
            let scenario = support::setup(&pool, name).await;
            write_fixture(
                MANIFEST_DIR,
                FIXTURE_SUBDIR,
                name,
                &build_fixture(&pool, &scenario).await,
            );
            support::cleanup(&pool, &scenario).await;
        }
        return;
    }

    for name in SCENARIOS {
        support::scenario(name);
        let scenario = support::setup(&pool, name).await;
        let expected: GoldenClaimFixture = load_fixture(MANIFEST_DIR, FIXTURE_SUBDIR, name);
        let actual = build_fixture(&pool, &scenario).await;

        assert_eq!(expected, actual, "scenario {name} claim outcome mismatch");
        support::cleanup(&pool, &scenario).await;
    }
}
