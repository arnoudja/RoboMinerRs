//! Shared helpers for `robominer-engine` CLI integration tests.
//!
//! Each `tests/*.rs` binary compiles this module; `dead_code` covers helpers only
//! exercised by sibling binaries.

#![allow(dead_code)]

use std::process::{Command, Output};

use robominer_test_support::{insert_user_with_credentials, unique_prefix};
use sqlx::MySqlPool;

#[allow(unused_imports)] // trait in scope via `use support::*` for `.try_get`
pub use sqlx::Row;

/// DB/fixture helpers still imported through `use support::*` in test binaries.
#[allow(unused_imports)]
pub use robominer_test_support::{
    AchievementCliFixture as TestAchievementFixture,
    CancelMiningQueueFixture as TestCancelMiningQueueFixture,
    ClaimResultsFixture as TestClaimResultsFixture,
    EnqueueMiningFixture as TestEnqueueMiningFixture, PoolFixture as TestPoolFixture,
    ProgramSourceFixture as TestProgramSourceFixture, RallyFixture as TestRallyFixture,
    RobotConfigFixture as TestRobotConfigFixture, ShopFixture as TestShopFixture,
    cleanup_claimed_queue_fixture, cleanup_created_user, ensure_default_robot_parts,
    insert_ai_robot, insert_claimed_mining_queue, insert_cli_robot as insert_robot,
    insert_robot_config_part, insert_row_id, insert_user_robot_part_asset,
    parse_created_program_source_id,
};

pub fn run_engine(args: &[String]) -> Output {
    Command::new(env!("CARGO_BIN_EXE_robominer-engine"))
        .args(args)
        .output()
        .expect("failed to execute robominer-engine")
}

pub fn output_text(output: &Output) -> (String, String) {
    (
        String::from_utf8_lossy(&output.stdout).into_owned(),
        String::from_utf8_lossy(&output.stderr).into_owned(),
    )
}

pub fn find_queue_state_line(output: &str, mining_queue_id: i64) -> Vec<&str> {
    let prefix = format!("{mining_queue_id}\t");
    output
        .lines()
        .find(|line| line.starts_with(&prefix))
        .unwrap_or_else(|| panic!("expected queue state for mining queue {mining_queue_id}"))
        .split('\t')
        .collect()
}

pub fn find_score_state_line(output: &str, robot_id: i64, mining_area_id: i64) -> Vec<&str> {
    let prefix = format!("{robot_id}\t{mining_area_id}\t");
    output
        .lines()
        .find(|line| line.starts_with(&prefix))
        .unwrap_or_else(|| {
            panic!("expected score state for robot {robot_id} and mining area {mining_area_id}")
        })
        .split('\t')
        .collect()
}

pub fn find_prefixed_line<'a>(output: &'a str, prefix: &str) -> Vec<&'a str> {
    output
        .lines()
        .find(|line| line.starts_with(prefix))
        .unwrap_or_else(|| panic!("expected output line with prefix {prefix:?}\nstdout:\n{output}"))
        .split('\t')
        .collect()
}

pub fn unique_test_prefix(prefix: &str) -> String {
    unique_prefix(prefix)
}

pub async fn insert_test_user(
    pool: &MySqlPool,
    username: &str,
    email: &str,
    password_hash: &str,
) -> i64 {
    insert_user_with_credentials(pool, username, email, password_hash).await
}
