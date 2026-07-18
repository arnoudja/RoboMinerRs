use robominer_db::MySqlPool;

use crate::{insert_row_id, unique_prefix};

pub struct AchievementCliFixture {
    pub user_id: i64,
    pub ore_id: i64,
    pub achievement_id: i64,
    pub achievement_step: i32,
    pub successor_achievement_id: std::cell::Cell<Option<i64>>,
}

impl AchievementCliFixture {
    pub async fn create(pool: &MySqlPool, robot_reward: bool) -> Self {
        Self::create_with_requirement(pool, robot_reward, 0).await
    }

    pub async fn create_unmet(pool: &MySqlPool) -> Self {
        Self::create_with_requirement(pool, false, 10).await
    }

    pub async fn create_with_requirement(
        pool: &MySqlPool,
        robot_reward: bool,
        required_ore_amount: i32,
    ) -> Self {
        let prefix = unique_prefix("rust-achievement-cli");

        let user_id = insert_row_id(
            pool,
            sqlx::query("INSERT INTO User (username, email, password) VALUES (?, ?, ?)")
                .bind(format!("{prefix}-user"))
                .bind(format!("{prefix}@example.invalid"))
                .bind("test-password"),
        )
        .await;
        let ore_id = insert_row_id(
            pool,
            sqlx::query("INSERT INTO Ore (oreName) VALUES (?)").bind(format!("{prefix}-ore")),
        )
        .await;
        let achievement_id = insert_row_id(
            pool,
            sqlx::query("INSERT INTO Achievement (title, description) VALUES (?, ?)")
                .bind(format!("{prefix}-achievement"))
                .bind("test achievement"),
        )
        .await;
        sqlx::query(
            "INSERT INTO AchievementStep \
             (achievementId, step, achievementPoints, miningQueueReward, robotReward, \
              oreId, maxOreReward, maxDepotReward) \
             VALUES (?, 1, 7, 2, ?, ?, 80, 40)",
        )
        .bind(achievement_id)
        .bind(if robot_reward { 1 } else { 0 })
        .bind(ore_id)
        .execute(pool)
        .await
        .expect("failed to insert achievement step");
        if required_ore_amount > 0 {
            sqlx::query(
                "INSERT INTO AchievementStepMiningTotalRequirement \
                 (achievementId, step, oreId, amount) \
                 VALUES (?, 1, ?, ?)",
            )
            .bind(achievement_id)
            .bind(ore_id)
            .bind(required_ore_amount)
            .execute(pool)
            .await
            .expect("failed to insert achievement total requirement");
        }
        insert_row_id(
            pool,
            sqlx::query(
                "INSERT INTO UserAchievement (userId, achievementId, stepsClaimed) \
                 VALUES (?, ?, 0)",
            )
            .bind(user_id)
            .bind(achievement_id),
        )
        .await;

        Self {
            user_id,
            ore_id,
            achievement_id,
            achievement_step: 1,
            successor_achievement_id: std::cell::Cell::new(None),
        }
    }

    pub fn claim_args(&self, database_url: &str) -> Vec<String> {
        vec![
            "--database-url".to_string(),
            database_url.to_string(),
            "claim-achievement-step".to_string(),
            "--user-id".to_string(),
            self.user_id.to_string(),
            "--achievement-id".to_string(),
            self.achievement_id.to_string(),
        ]
    }

    pub async fn add_successor(&self, pool: &MySqlPool) {
        let successor_achievement_id = insert_row_id(
            pool,
            sqlx::query("INSERT INTO Achievement (title, description) VALUES (?, ?)")
                .bind(format!("successor-{}", self.achievement_id))
                .bind("successor achievement"),
        )
        .await;
        insert_row_id(
            pool,
            sqlx::query(
                "INSERT INTO AchievementPredecessor \
                 (predecessorId, predecessorStep, successorId) \
                 VALUES (?, 1, ?)",
            )
            .bind(self.achievement_id)
            .bind(successor_achievement_id),
        )
        .await;
        self.successor_achievement_id
            .set(Some(successor_achievement_id));
    }

    pub async fn assert_claimed_rewards(&self, pool: &MySqlPool) {
        let (achievement_points, mining_queue_size): (i32, i32) =
            sqlx::query_as("SELECT achievementPoints, miningQueueSize FROM User WHERE id = ?")
                .bind(self.user_id)
                .fetch_one(pool)
                .await
                .expect("failed to load user rewards");
        assert_eq!(achievement_points, 7);
        assert_eq!(mining_queue_size, 2);

        let steps_claimed: i32 = sqlx::query_scalar(
            "SELECT stepsClaimed FROM UserAchievement WHERE userId = ? AND achievementId = ?",
        )
        .bind(self.user_id)
        .bind(self.achievement_id)
        .fetch_one(pool)
        .await
        .expect("failed to load claimed steps");
        assert_eq!(steps_claimed, 1);

        let (max_allowed, depot_max_allowed): (i32, i32) = sqlx::query_as(
            "SELECT maxAllowed, depotMaxAllowed FROM UserOreAsset WHERE userId = ? AND oreId = ?",
        )
        .bind(self.user_id)
        .bind(self.ore_id)
        .fetch_one(pool)
        .await
        .expect("failed to load ore max reward");
        assert_eq!(max_allowed, 80);
        assert_eq!(depot_max_allowed, 40);
    }

    pub async fn assert_successor_unlocked(&self, pool: &MySqlPool) {
        let successor_achievement_id = self
            .successor_achievement_id
            .get()
            .expect("successor should be configured");
        let count: i64 = sqlx::query_scalar(
            "SELECT COUNT(*) FROM UserAchievement WHERE userId = ? AND achievementId = ?",
        )
        .bind(self.user_id)
        .bind(successor_achievement_id)
        .fetch_one(pool)
        .await
        .expect("failed to count successor achievement");
        assert_eq!(count, 1);
    }

    pub async fn assert_successor_not_unlocked(&self, pool: &MySqlPool) {
        let successor_achievement_id = self
            .successor_achievement_id
            .get()
            .expect("successor should be configured");
        let count: i64 = sqlx::query_scalar(
            "SELECT COUNT(*) FROM UserAchievement WHERE userId = ? AND achievementId = ?",
        )
        .bind(self.user_id)
        .bind(successor_achievement_id)
        .fetch_one(pool)
        .await
        .expect("failed to count successor achievement");
        assert_eq!(count, 0);
    }

    pub async fn assert_robot_reward(&self, pool: &MySqlPool) {
        let robot_count: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM Robot WHERE userId = ?")
            .bind(self.user_id)
            .fetch_one(pool)
            .await
            .expect("failed to count robot reward");
        assert_eq!(robot_count, 1);
    }

    pub async fn assert_not_claimed(&self, pool: &MySqlPool) {
        let steps_claimed: i32 = sqlx::query_scalar(
            "SELECT stepsClaimed FROM UserAchievement WHERE userId = ? AND achievementId = ?",
        )
        .bind(self.user_id)
        .bind(self.achievement_id)
        .fetch_one(pool)
        .await
        .expect("failed to load claimed steps");
        assert_eq!(steps_claimed, 0);
    }

    pub async fn cleanup(&self, pool: &MySqlPool) {
        let _ = sqlx::query(
            "DELETE RobotLifetimeResult FROM RobotLifetimeResult \
             INNER JOIN Robot ON Robot.id = RobotLifetimeResult.robotId \
             WHERE Robot.userId = ?",
        )
        .bind(self.user_id)
        .execute(pool)
        .await;
        let _ = sqlx::query("DELETE FROM Robot WHERE userId = ?")
            .bind(self.user_id)
            .execute(pool)
            .await;
        let _ = sqlx::query("DELETE FROM ProgramSource WHERE userId = ?")
            .bind(self.user_id)
            .execute(pool)
            .await;
        let _ = sqlx::query("DELETE FROM UserRobotPartAsset WHERE userId = ?")
            .bind(self.user_id)
            .execute(pool)
            .await;
        let _ = sqlx::query("DELETE FROM UserOreAsset WHERE userId = ?")
            .bind(self.user_id)
            .execute(pool)
            .await;
        let _ = sqlx::query("DELETE FROM UserAchievement WHERE userId = ?")
            .bind(self.user_id)
            .execute(pool)
            .await;
        if let Some(successor_achievement_id) = self.successor_achievement_id.get() {
            let _ = sqlx::query("DELETE FROM AchievementPredecessor WHERE successorId = ?")
                .bind(successor_achievement_id)
                .execute(pool)
                .await;
            let _ = sqlx::query("DELETE FROM Achievement WHERE id = ?")
                .bind(successor_achievement_id)
                .execute(pool)
                .await;
        }
        let _ = sqlx::query(
            "DELETE FROM AchievementStepMiningTotalRequirement \
             WHERE achievementId = ? AND step = ?",
        )
        .bind(self.achievement_id)
        .bind(self.achievement_step)
        .execute(pool)
        .await;
        let _ = sqlx::query(
            "DELETE FROM AchievementStepMiningScoreRequirement \
             WHERE achievementId = ? AND step = ?",
        )
        .bind(self.achievement_id)
        .bind(self.achievement_step)
        .execute(pool)
        .await;
        let _ = sqlx::query("DELETE FROM AchievementStep WHERE achievementId = ? AND step = ?")
            .bind(self.achievement_id)
            .bind(self.achievement_step)
            .execute(pool)
            .await;
        let _ = sqlx::query("DELETE FROM Achievement WHERE id = ?")
            .bind(self.achievement_id)
            .execute(pool)
            .await;
        let _ = sqlx::query("DELETE FROM User WHERE id = ?")
            .bind(self.user_id)
            .execute(pool)
            .await;
        let _ = sqlx::query("DELETE FROM Ore WHERE id = ?")
            .bind(self.ore_id)
            .execute(pool)
            .await;
    }
}
