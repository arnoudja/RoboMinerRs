pub use sqlx::MySqlPool;
use sqlx::mysql::MySqlPoolOptions;

mod initial_ore_wallet_max {
    include!("../../robominer-domain/src/initial_ore_wallet_max.rs");
}

pub use initial_ore_wallet_max::INITIAL_ORE_WALLET_MAX;

pub const SCORE_HISTORY_FACTOR: f64 = 5.0;
pub const SCORE_START_FACTOR: f64 = 1.4;

mod achievements;
mod activity;
mod app_shell;
mod assets;
mod catalog;
mod config;
mod leaderboard;
mod mappers;
mod mining_areas;
mod mining_queue;
mod password;
mod pool;
mod program_sources;
mod rally;
mod results;
mod robots;
mod shop;
mod types;
mod users;

pub use achievements::*;
pub use activity::*;
pub use app_shell::*;
pub use assets::*;
pub use catalog::*;
pub use config::*;
pub use leaderboard::*;
pub use mining_areas::*;
pub use mining_queue::*;
pub use pool::*;
pub use program_sources::*;
pub use rally::*;
pub use results::*;
pub use robots::*;
pub use shop::*;
pub use types::*;
pub use users::*;

pub async fn connect(database_url: &str) -> Result<MySqlPool, sqlx::Error> {
    MySqlPoolOptions::new()
        .max_connections(5)
        .connect(database_url)
        .await
}

#[cfg(test)]
mod tests {
    use crate::mappers::{
        MiningRallyQueueRow, PoolItemRow, mining_rally_queue_rows, next_pool_rally_item_rows,
    };

    #[test]
    fn next_pool_rally_items_keep_only_lowest_runs_done_cohort() {
        let rows = vec![
            pool_item_row(1, 900, 11, 50.0, 2),
            pool_item_row(2, 900, 12, 80.0, 2),
            pool_item_row(3, 900, 13, 120.0, 3),
        ];

        let items = next_pool_rally_item_rows(rows);

        assert_eq!(
            items.iter().map(|item| item.id).collect::<Vec<_>>(),
            vec![1, 2]
        );
    }

    #[test]
    fn next_pool_rally_items_allow_empty_pools() {
        assert!(next_pool_rally_item_rows(Vec::new()).is_empty());
    }

    #[test]
    fn next_mining_rally_queue_keeps_first_robot_per_user_before_cap() {
        let rows = vec![
            mining_rally_queue_row(1, 100, 11, 501, 5),
            mining_rally_queue_row(2, 100, 12, 502, 6),
            mining_rally_queue_row(3, 100, 13, 501, 7),
            mining_rally_queue_row(4, 100, 14, 503, 8),
            mining_rally_queue_row(5, 100, 15, 504, 9),
            mining_rally_queue_row(6, 100, 16, 505, 10),
        ];

        let queue = mining_rally_queue_rows(rows);

        assert_eq!(
            queue
                .iter()
                .map(|record| (record.queue.id, record.user_id))
                .collect::<Vec<_>>(),
            vec![(1, 501), (2, 502), (4, 503), (5, 504)]
        );
    }

    fn pool_item_row(
        id: i64,
        pool_id: i64,
        robot_id: i64,
        total_score: f64,
        runs_done: i32,
    ) -> PoolItemRow {
        (
            id,
            pool_id,
            robot_id,
            format!("mine({id});"),
            total_score,
            runs_done,
        )
    }

    fn mining_rally_queue_row(
        id: i64,
        mining_area_id: i64,
        robot_id: i64,
        user_id: i64,
        seconds_left: i32,
    ) -> MiningRallyQueueRow {
        (
            id,
            mining_area_id,
            robot_id,
            user_id,
            None,
            None,
            None,
            false,
            seconds_left,
        )
    }
}
