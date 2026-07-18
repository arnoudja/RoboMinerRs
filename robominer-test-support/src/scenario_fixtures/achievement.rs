use robominer_db::MySqlPool;

use crate::{insert_row_id, unique_prefix};

pub struct AchievementScenario {
    pub user_id: i64,
    pub ore_id: i64,
    pub achievement_id: i64,
}

impl AchievementScenario {
    pub async fn attach_to_user(pool: &MySqlPool, prefix: &str, user_id: i64) -> Self {
        let ore_id = insert_row_id(
            pool,
            sqlx::query("INSERT INTO Ore (oreName) VALUES (?)").bind(format!("{prefix}-ore")),
        )
        .await;
        let achievement_id = insert_row_id(
            pool,
            sqlx::query("INSERT INTO Achievement (title, description) VALUES (?, ?)")
                .bind(format!("{prefix}-achievement"))
                .bind("achievement claim test"),
        )
        .await;

        sqlx::query(
            "INSERT INTO AchievementStep \
             (achievementId, step, achievementPoints, miningQueueReward, robotReward, \
              oreId, maxOreReward, maxDepotReward) \
             VALUES (?, 1, 7, 2, 0, ?, 80, 40)",
        )
        .bind(achievement_id)
        .bind(ore_id)
        .execute(pool)
        .await
        .expect("failed to insert achievement step");

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
        }
    }

    pub async fn create_standalone(pool: &MySqlPool) -> Self {
        let prefix = unique_prefix("rust-achievement");
        let user_id = insert_row_id(
            pool,
            sqlx::query("INSERT INTO User (username, email, password) VALUES (?, ?, ?)")
                .bind(format!("{prefix}-user"))
                .bind(format!("{prefix}@example.invalid"))
                .bind("test-password"),
        )
        .await;
        Self::attach_to_user(pool, &prefix, user_id).await
    }

    pub async fn assert_claimed(
        &self,
        pool: &MySqlPool,
        expected_points: i32,
        expected_queue_size: i32,
    ) {
        let steps_claimed: i32 = sqlx::query_scalar(
            "SELECT stepsClaimed FROM UserAchievement WHERE userId = ? AND achievementId = ?",
        )
        .bind(self.user_id)
        .bind(self.achievement_id)
        .fetch_one(pool)
        .await
        .expect("failed to load claimed steps");
        assert_eq!(steps_claimed, 1);

        let (achievement_points, mining_queue_size): (i32, i32) =
            sqlx::query_as("SELECT achievementPoints, miningQueueSize FROM User WHERE id = ?")
                .bind(self.user_id)
                .fetch_one(pool)
                .await
                .expect("failed to load user rewards");
        assert_eq!(achievement_points, expected_points);
        assert_eq!(mining_queue_size, expected_queue_size);

        let (max_allowed, depot_max_allowed): (i32, i32) = sqlx::query_as(
            "SELECT maxAllowed, depotMaxAllowed FROM UserOreAsset WHERE userId = ? AND oreId = ?",
        )
        .bind(self.user_id)
        .bind(self.ore_id)
        .fetch_one(pool)
        .await
        .expect("failed to load ore asset caps");
        assert_eq!(max_allowed, 80);
        assert_eq!(depot_max_allowed, 40);
    }

    pub async fn cleanup(&self, pool: &MySqlPool, delete_user: bool) {
        let _ = sqlx::query("DELETE FROM UserOreAsset WHERE userId = ?")
            .bind(self.user_id)
            .execute(pool)
            .await;
        let _ = sqlx::query("DELETE FROM UserAchievement WHERE userId = ?")
            .bind(self.user_id)
            .execute(pool)
            .await;
        let _ = sqlx::query("DELETE FROM AchievementStep WHERE achievementId = ?")
            .bind(self.achievement_id)
            .execute(pool)
            .await;
        let _ = sqlx::query("DELETE FROM Achievement WHERE id = ?")
            .bind(self.achievement_id)
            .execute(pool)
            .await;
        if delete_user {
            let _ = sqlx::query("DELETE FROM Robot WHERE userId = ?")
                .bind(self.user_id)
                .execute(pool)
                .await;
            let _ = sqlx::query("DELETE FROM User WHERE id = ?")
                .bind(self.user_id)
                .execute(pool)
                .await;
        }
        let _ = sqlx::query("DELETE FROM Ore WHERE id = ?")
            .bind(self.ore_id)
            .execute(pool)
            .await;
    }
}
