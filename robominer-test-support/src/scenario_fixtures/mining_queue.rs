use robominer_db::MySqlPool;

use crate::{
    insert_mining_queue, insert_robot, insert_row_id, insert_user_ore_asset, unique_prefix,
};

pub struct RobotMiningAreaFixture {
    pub user_id: i64,
    pub robot_id: i64,
    pub ai_robot_id: i64,
    pub mining_area_id: i64,
    pub area_name: String,
    pub ore_id: i64,
    pub ore_price_id: i64,
}

impl RobotMiningAreaFixture {
    pub async fn create(
        pool: &MySqlPool,
        prefix: &str,
        user_id: i64,
        mining_time: i32,
        user_ore_amount: Option<i32>,
    ) -> Self {
        let area_name = format!("{prefix}-area");
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
        insert_row_id(
            pool,
            sqlx::query("INSERT INTO OrePriceAmount (orePriceId, oreId, amount) VALUES (?, ?, ?)")
                .bind(ore_price_id)
                .bind(ore_id)
                .bind(1),
        )
        .await;

        if let Some(amount) = user_ore_amount {
            insert_user_ore_asset(pool, user_id, ore_id, amount, 1000).await;
        }

        let ai_robot_id =
            insert_robot(pool, user_id, &format!("{prefix}-ai"), "rotate(90);", 1).await;
        let robot_id = insert_robot(pool, user_id, &format!("{prefix}-robot"), "mine();", 1).await;
        let mining_area_id = insert_row_id(
            pool,
            sqlx::query(
                "INSERT INTO MiningArea \
                 (areaName, orePriceId, sizeX, sizeY, maxMoves, miningTime, taxRate, aiRobotId) \
                 VALUES (?, ?, 4, 4, 1, ?, 0, ?)",
            )
            .bind(&area_name)
            .bind(ore_price_id)
            .bind(mining_time)
            .bind(ai_robot_id),
        )
        .await;

        sqlx::query("INSERT INTO UserMiningArea (userId, miningAreaId) VALUES (?, ?)")
            .bind(user_id)
            .bind(mining_area_id)
            .execute(pool)
            .await
            .expect("failed to grant mining area");

        Self {
            user_id,
            robot_id,
            ai_robot_id,
            mining_area_id,
            area_name,
            ore_id,
            ore_price_id,
        }
    }

    pub async fn insert_queue_pair(&self, pool: &MySqlPool) -> (i64, i64) {
        let active_queue_id = insert_mining_queue(pool, self.mining_area_id, self.robot_id).await;
        let queued_queue_id = insert_mining_queue(pool, self.mining_area_id, self.robot_id).await;
        (active_queue_id, queued_queue_id)
    }

    pub async fn cleanup(&self, pool: &MySqlPool, delete_user: bool) {
        let _ = sqlx::query("DELETE FROM MiningQueue WHERE robotId = ?")
            .bind(self.robot_id)
            .execute(pool)
            .await;
        let _ = sqlx::query("DELETE FROM UserMiningArea WHERE userId = ? AND miningAreaId = ?")
            .bind(self.user_id)
            .bind(self.mining_area_id)
            .execute(pool)
            .await;
        let _ = sqlx::query("DELETE FROM MiningArea WHERE id = ?")
            .bind(self.mining_area_id)
            .execute(pool)
            .await;
        let _ = sqlx::query("DELETE FROM Robot WHERE id IN (?, ?)")
            .bind(self.robot_id)
            .bind(self.ai_robot_id)
            .execute(pool)
            .await;
        let _ = sqlx::query("DELETE FROM UserOreAsset WHERE userId = ?")
            .bind(self.user_id)
            .execute(pool)
            .await;
        if delete_user {
            let _ = sqlx::query("DELETE FROM User WHERE id = ?")
                .bind(self.user_id)
                .execute(pool)
                .await;
        }
        let _ = sqlx::query("DELETE FROM OrePriceAmount WHERE orePriceId = ?")
            .bind(self.ore_price_id)
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

pub struct IdleMiningAreaFixture {
    pub prefix: String,
    pub user_id: i64,
    pub inner: RobotMiningAreaFixture,
}

impl IdleMiningAreaFixture {
    pub async fn create(pool: &MySqlPool, user_id: i64, user_ore_amount: i32) -> Self {
        let prefix = unique_prefix("rust-queue-idle");
        let inner =
            RobotMiningAreaFixture::create(pool, &prefix, user_id, 1, Some(user_ore_amount)).await;
        Self {
            prefix,
            user_id,
            inner,
        }
    }
}

pub struct QueuedMiningAreaFixture {
    pub prefix: String,
    pub user_id: i64,
    pub inner: RobotMiningAreaFixture,
    pub active_queue_id: i64,
    pub queued_queue_id: i64,
}

impl QueuedMiningAreaFixture {
    pub async fn create(pool: &MySqlPool, user_id: i64) -> Self {
        let prefix = unique_prefix("rust-queue-queued");
        let inner = RobotMiningAreaFixture::create(pool, &prefix, user_id, 10, None).await;
        let (active_queue_id, queued_queue_id) = inner.insert_queue_pair(pool).await;
        Self {
            prefix,
            user_id,
            inner,
            active_queue_id,
            queued_queue_id,
        }
    }
}
