#![allow(clippy::unwrap_used, clippy::expect_used)]
#![cfg_attr(test, allow(clippy::unwrap_used, clippy::expect_used))]
//! Shared DB fixtures, golden helpers, and scenario builders for integration
//! tests (`publish = false`).

mod db_fixtures;
mod golden;
mod loadout_records;
mod scenario_fixtures;
mod test_db;

pub use db_fixtures::*;
pub use golden::{
    assert_or_update_golden, assert_or_update_golden_async, fixture_path, load_fixture,
    round_golden_coord, round_golden_score, update_golden_enabled, write_fixture,
};
pub use loadout_records::*;
pub use scenario_fixtures::*;
pub use test_db::require_test_db;
