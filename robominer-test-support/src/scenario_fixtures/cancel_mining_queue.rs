use robominer_db::MySqlPool;

use crate::{
    insert_cli_robot, insert_mining_queue, insert_row_id, insert_user_with_credentials,
    unique_prefix,
};

pub struct CancelMiningQueueFixture {
    pub user_id: i64,
    pub other_user_id: i64,
    pub ore_id: i64,
    pub ore_price_id: i64,
    pub ai_robot_id: i64,
    pub robot_id: i64,
    pub mining_area_id: i64,
    pub active_queue_id: i64,
    pub queued_queue_id: i64,
    pub rally_result_id: std::cell::Cell<Option<i64>>,
    pub rally_backed_queue_id: std::cell::Cell<Option<i64>>,
}

impl CancelMiningQueueFixture {
    pub async fn create(pool: &MySqlPool) -> Self {
        let prefix = unique_prefix("rust-cancel-queue-cli");
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
        let user_id = insert_user_with_credentials(
            pool,
            &format!("{prefix}-user"),
            &format!("{prefix}@example.invalid"),
            "test-password",
        )
        .await;
        let other_user_id = insert_user_with_credentials(
            pool,
            &format!("{prefix}-other-user"),
            &format!("{prefix}-other@example.invalid"),
            "test-password",
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
                 VALUES (?, ?, 4, 4, 1, 10, 0, ?)",
            )
            .bind(format!("{prefix}-area"))
            .bind(ore_price_id)
            .bind(ai_robot_id),
        )
        .await;
        let active_queue_id = insert_mining_queue(pool, mining_area_id, robot_id).await;
        let queued_queue_id = insert_mining_queue(pool, mining_area_id, robot_id).await;

        Self {
            user_id,
            other_user_id,
            ore_id,
            ore_price_id,
            ai_robot_id,
            robot_id,
            mining_area_id,
            active_queue_id,
            queued_queue_id,
            rally_result_id: std::cell::Cell::new(None),
            rally_backed_queue_id: std::cell::Cell::new(None),
        }
    }

    pub async fn add_rally_backed_queue(&self, pool: &MySqlPool) -> i64 {
        let rally_result_id = insert_row_id(
            pool,
            sqlx::query("INSERT INTO RallyResult (resultData) VALUES ('test-rally-result')"),
        )
        .await;
        let queue_id = insert_row_id(
            pool,
            sqlx::query(
                "INSERT INTO MiningQueue \
                 (miningAreaId, robotId, rallyResultId, playerNumber, miningEndTime) \
                 VALUES (?, ?, ?, 0, TIMESTAMPADD(SECOND, 10, NOW()))",
            )
            .bind(self.mining_area_id)
            .bind(self.robot_id)
            .bind(rally_result_id),
        )
        .await;
        self.rally_result_id.set(Some(rally_result_id));
        self.rally_backed_queue_id.set(Some(queue_id));
        queue_id
    }

    pub async fn assert_queue_exists(&self, pool: &MySqlPool, queue_id: i64, expected: bool) {
        let count: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM MiningQueue WHERE id = ?")
            .bind(queue_id)
            .fetch_one(pool)
            .await
            .expect("failed to count mining queue item");
        assert_eq!(count > 0, expected);
    }

    pub async fn cleanup(&self, pool: &MySqlPool) {
        let _ = sqlx::query("DELETE FROM MiningQueue WHERE miningAreaId = ?")
            .bind(self.mining_area_id)
            .execute(pool)
            .await;
        let _ = sqlx::query("DELETE FROM RobotMiningAreaScore WHERE robotId = ?")
            .bind(self.robot_id)
            .execute(pool)
            .await;
        let _ = sqlx::query("DELETE FROM MiningAreaLifetimeResult WHERE miningAreaId = ?")
            .bind(self.mining_area_id)
            .execute(pool)
            .await;
        let _ = sqlx::query("DELETE FROM MiningAreaOreSupply WHERE miningAreaId = ?")
            .bind(self.mining_area_id)
            .execute(pool)
            .await;
        let _ = sqlx::query("DELETE FROM UserMiningArea WHERE miningAreaId = ?")
            .bind(self.mining_area_id)
            .execute(pool)
            .await;
        if let Some(rally_result_id) = self.rally_result_id.get() {
            let _ = sqlx::query("DELETE FROM RallyResult WHERE id = ?")
                .bind(rally_result_id)
                .execute(pool)
                .await;
        }
        let _ = sqlx::query("DELETE FROM MiningArea WHERE id = ?")
            .bind(self.mining_area_id)
            .execute(pool)
            .await;
        let _ = sqlx::query("DELETE FROM Robot WHERE id IN (?, ?)")
            .bind(self.ai_robot_id)
            .bind(self.robot_id)
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
