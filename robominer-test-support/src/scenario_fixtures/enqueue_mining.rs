use robominer_db::MySqlPool;

use crate::{insert_cli_robot, insert_row_id, unique_prefix};

pub struct EnqueueMiningFixture {
    pub user_id: i64,
    pub other_user_id: i64,
    pub ore_id: i64,
    pub ore_price_id: i64,
    pub ai_robot_id: i64,
    pub robot_id: i64,
    pub mining_area_id: i64,
}

impl EnqueueMiningFixture {
    pub async fn create(
        pool: &MySqlPool,
        queue_size: i32,
        user_ore_amount: i32,
        mining_cost: i32,
        grant_area: bool,
    ) -> Self {
        let prefix = unique_prefix("rust-enqueue-cli");

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
                .bind(mining_cost),
        )
        .await;

        let user_id = insert_row_id(
            pool,
            sqlx::query(
                "INSERT INTO User (username, email, password, miningQueueSize) \
                 VALUES (?, ?, ?, ?)",
            )
            .bind(format!("{prefix}-user"))
            .bind(format!("{prefix}@example.invalid"))
            .bind("test-password")
            .bind(queue_size),
        )
        .await;
        let other_user_id = insert_row_id(
            pool,
            sqlx::query(
                "INSERT INTO User (username, email, password, miningQueueSize) \
                 VALUES (?, ?, ?, ?)",
            )
            .bind(format!("{prefix}-other-user"))
            .bind(format!("{prefix}-other@example.invalid"))
            .bind("test-password")
            .bind(queue_size),
        )
        .await;

        insert_row_id(
            pool,
            sqlx::query(
                "INSERT INTO UserOreAsset (userId, oreId, amount, maxAllowed) \
                 VALUES (?, ?, ?, 1000)",
            )
            .bind(user_id)
            .bind(ore_id)
            .bind(user_ore_amount),
        )
        .await;
        insert_row_id(
            pool,
            sqlx::query(
                "INSERT INTO UserOreAsset (userId, oreId, amount, maxAllowed) \
                 VALUES (?, ?, ?, 1000)",
            )
            .bind(other_user_id)
            .bind(ore_id)
            .bind(user_ore_amount),
        )
        .await;

        let ai_robot_id =
            insert_cli_robot(pool, user_id, &format!("{prefix}-ai"), "rotate(90);").await;
        let robot_id = insert_cli_robot(pool, user_id, &format!("{prefix}-robot"), "mine();").await;
        let mining_area_id = insert_row_id(
            pool,
            sqlx::query(
                "INSERT INTO MiningArea \
                 (areaName, orePriceId, sizeX, sizeY, maxMoves, miningTime, taxRate, aiRobotId) \
                 VALUES (?, ?, 4, 4, 1, 1, 0, ?)",
            )
            .bind(format!("{prefix}-area"))
            .bind(ore_price_id)
            .bind(ai_robot_id),
        )
        .await;

        if grant_area {
            sqlx::query("INSERT INTO UserMiningArea (userId, miningAreaId) VALUES (?, ?)")
                .bind(user_id)
                .bind(mining_area_id)
                .execute(pool)
                .await
                .expect("failed to grant mining area");
            sqlx::query("INSERT INTO UserMiningArea (userId, miningAreaId) VALUES (?, ?)")
                .bind(other_user_id)
                .bind(mining_area_id)
                .execute(pool)
                .await
                .expect("failed to grant mining area to other user");
        }

        Self {
            user_id,
            other_user_id,
            ore_id,
            ore_price_id,
            ai_robot_id,
            robot_id,
            mining_area_id,
        }
    }

    pub async fn assert_queue_and_asset(
        &self,
        pool: &MySqlPool,
        expected_queue_count: i64,
        expected_ore_amount: i32,
    ) {
        let queue_count: i64 =
            sqlx::query_scalar("SELECT COUNT(*) FROM MiningQueue WHERE robotId = ?")
                .bind(self.robot_id)
                .fetch_one(pool)
                .await
                .expect("failed to load queue count");
        assert_eq!(queue_count, expected_queue_count);

        let user_ore_amount: i32 =
            sqlx::query_scalar("SELECT amount FROM UserOreAsset WHERE userId = ? AND oreId = ?")
                .bind(self.user_id)
                .bind(self.ore_id)
                .fetch_one(pool)
                .await
                .expect("failed to load user ore asset");
        assert_eq!(user_ore_amount, expected_ore_amount);
    }

    pub async fn cleanup(&self, pool: &MySqlPool) {
        let _ = sqlx::query("DELETE FROM MiningQueue WHERE robotId = ?")
            .bind(self.robot_id)
            .execute(pool)
            .await;
        let _ = sqlx::query("DELETE FROM UserMiningArea WHERE miningAreaId = ?")
            .bind(self.mining_area_id)
            .execute(pool)
            .await;
        let _ = sqlx::query("DELETE FROM MiningArea WHERE id = ?")
            .bind(self.mining_area_id)
            .execute(pool)
            .await;
        let _ = sqlx::query("DELETE FROM Robot WHERE id IN (?, ?)")
            .bind(self.ai_robot_id)
            .bind(self.robot_id)
            .execute(pool)
            .await;
        let _ = sqlx::query("DELETE FROM UserOreAsset WHERE userId IN (?, ?)")
            .bind(self.user_id)
            .bind(self.other_user_id)
            .execute(pool)
            .await;
        let _ = sqlx::query("DELETE FROM User WHERE id IN (?, ?)")
            .bind(self.user_id)
            .bind(self.other_user_id)
            .execute(pool)
            .await;
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
