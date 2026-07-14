use robominer_db::MySqlPool;
use sqlx::Row;

use crate::{
    insert_cli_robot, insert_row_id, unique_prefix,
};

pub struct PoolFixture {
    pub user_id: i64,
    pub ore_id: i64,
    pub ore_price_id: i64,
    pub ai_robot_id: i64,
    pub pool_robot_id: i64,
    pub mining_area_id: i64,
    pub pool_id: i64,
    pub pool_item_id: i64,
}

impl PoolFixture {
    pub async fn create(pool: &MySqlPool) -> Self {
        let prefix = unique_prefix("rust-pool-db");

        let ore_id = insert_row_id(
            pool,
            sqlx::query("INSERT INTO Ore (oreName) VALUES (?)").bind(format!("{prefix}-ore")),
        )
        .await;
        let ore_price_id = insert_row_id(
            pool,
            sqlx::query("INSERT INTO OrePrice (description) VALUES (?)")
                .bind(format!("{prefix}-price")),
        )
        .await;
        let user_id = insert_row_id(
            pool,
            sqlx::query("INSERT INTO User (username, email, password) VALUES (?, ?, ?)")
                .bind(format!("{prefix}-user"))
                .bind(format!("{prefix}@example.invalid"))
                .bind("test-password"),
        )
        .await;
        let ai_robot_id =
            insert_cli_robot(pool, user_id, &format!("{prefix}-ai"), "rotate(90);").await;
        let pool_robot_id = insert_cli_robot(
            pool,
            user_id,
            &format!("{prefix}-pool"),
            "move(1.5); while (mine());",
        )
        .await;
        let mining_area_id = insert_row_id(
            pool,
            sqlx::query(
                "INSERT INTO MiningArea \
                 (areaName, orePriceId, sizeX, sizeY, maxMoves, miningTime, taxRate, aiRobotId) \
                 VALUES (?, ?, 4, 4, 10, 1, 0, ?)",
            )
            .bind(format!("{prefix}-area"))
            .bind(ore_price_id)
            .bind(ai_robot_id),
        )
        .await;

        insert_row_id(
            pool,
            sqlx::query(
                "INSERT INTO MiningAreaOreSupply (miningAreaId, oreId, supply, radius) \
                 VALUES (?, ?, 10, 1)",
            )
            .bind(mining_area_id)
            .bind(ore_id),
        )
        .await;

        let pool_id = insert_row_id(
            pool,
            sqlx::query("INSERT INTO Pool (miningAreaId, requiredRuns) VALUES (?, 3)")
                .bind(mining_area_id),
        )
        .await;
        let pool_item_id = insert_row_id(
            pool,
            sqlx::query(
                "INSERT INTO PoolItem \
                 (poolId, robotId, sourceCode, totalScore, runsDone) \
                 VALUES (?, ?, 'move(1.5); while (mine());', 10.0, 2)",
            )
            .bind(pool_id)
            .bind(pool_robot_id),
        )
        .await;

        insert_row_id(
            pool,
            sqlx::query(
                "INSERT INTO PoolItemMiningTotals (poolItemId, oreId, totalMined) \
                 VALUES (?, ?, 5)",
            )
            .bind(pool_item_id)
            .bind(ore_id),
        )
        .await;

        Self {
            user_id,
            ore_id,
            ore_price_id,
            ai_robot_id,
            pool_robot_id,
            mining_area_id,
            pool_id,
            pool_item_id,
        }
    }

    pub async fn assert_persisted(&self, pool: &MySqlPool) {
        let item = sqlx::query("SELECT totalScore, runsDone FROM PoolItem WHERE id = ?")
            .bind(self.pool_item_id)
            .fetch_one(pool)
            .await
            .expect("failed to load persisted pool item");
        let total_score: f64 = item.try_get("totalScore").unwrap();
        let runs_done: i32 = item.try_get("runsDone").unwrap();
        assert_eq!(total_score, 17.25);
        assert_eq!(runs_done, 3);

        let total_mined: i64 = sqlx::query_scalar(
            "SELECT totalMined FROM PoolItemMiningTotals WHERE poolItemId = ? AND oreId = ?",
        )
        .bind(self.pool_item_id)
        .bind(self.ore_id)
        .fetch_one(pool)
        .await
        .expect("failed to load persisted pool ore total");
        assert_eq!(total_mined, 9);
    }

    pub async fn assert_cli_persisted(&self, pool: &MySqlPool) {
        let item = sqlx::query("SELECT totalScore, runsDone FROM PoolItem WHERE id = ?")
            .bind(self.pool_item_id)
            .fetch_one(pool)
            .await
            .expect("failed to load persisted pool item");
        let total_score: f64 = item.try_get("totalScore").unwrap();
        let runs_done: i32 = item.try_get("runsDone").unwrap();
        assert!(total_score > 10.0);
        assert_eq!(runs_done, 3);

        let total_mined: i64 = sqlx::query_scalar(
            "SELECT totalMined FROM PoolItemMiningTotals WHERE poolItemId = ? AND oreId = ?",
        )
        .bind(self.pool_item_id)
        .bind(self.ore_id)
        .fetch_one(pool)
        .await
        .expect("failed to load persisted pool ore total");
        assert!(total_mined > 5);
    }

    pub async fn cleanup(&self, pool: &MySqlPool) {
        let _ = sqlx::query("DELETE FROM PoolItemMiningTotals WHERE poolItemId = ?")
            .bind(self.pool_item_id)
            .execute(pool)
            .await;
        let _ = sqlx::query("DELETE FROM PoolItem WHERE id = ?")
            .bind(self.pool_item_id)
            .execute(pool)
            .await;
        let _ = sqlx::query("DELETE FROM Pool WHERE id = ?")
            .bind(self.pool_id)
            .execute(pool)
            .await;
        let _ = sqlx::query("DELETE FROM MiningAreaOreSupply WHERE miningAreaId = ?")
            .bind(self.mining_area_id)
            .execute(pool)
            .await;
        let _ = sqlx::query("DELETE FROM MiningArea WHERE id = ?")
            .bind(self.mining_area_id)
            .execute(pool)
            .await;
        let _ = sqlx::query("DELETE FROM Robot WHERE id IN (?, ?)")
            .bind(self.ai_robot_id)
            .bind(self.pool_robot_id)
            .execute(pool)
            .await;
        let _ = sqlx::query("DELETE FROM User WHERE id = ?")
            .bind(self.user_id)
            .execute(pool)
            .await;
        let _ = sqlx::query("DELETE FROM OrePrice WHERE id = ?")
            .bind(self.ore_price_id)
            .execute(pool)
            .await;
        let _ = sqlx::query("DELETE FROM Ore WHERE id = ?")
            .bind(self.ore_id)
            .execute(pool)
            .await;
    }
}

