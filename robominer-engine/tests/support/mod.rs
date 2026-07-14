#![allow(dead_code, unused_imports)]

use std::process::{Command, Output};

pub use sqlx::Row;
pub use robominer_test_support::{
    cleanup_claimed_queue_fixture, cleanup_created_user, insert_claimed_mining_queue,
    insert_cli_robot as insert_robot, insert_mining_queue, insert_row_id,
    insert_user_with_credentials, unique_prefix,
};
use sqlx::MySqlPool;

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

mod fixtures;
pub use fixtures::*;
