use sqlx::MySqlPool;

pub async fn reconcile_successor_unlocks(
    pool: &MySqlPool,
    user_id: i64,
) -> Result<(), sqlx::Error> {
    let mut transaction = pool.begin().await?;
    reconcile_successor_unlocks_in_transaction(&mut transaction, user_id).await?;
    transaction.commit().await?;
    Ok(())
}

async fn reconcile_successor_unlocks_in_transaction(
    transaction: &mut sqlx::Transaction<'_, sqlx::MySql>,
    user_id: i64,
) -> Result<(), sqlx::Error> {
    let successor_ids = sqlx::query_scalar::<_, i64>(
        "SELECT DISTINCT successorId \
         FROM AchievementPredecessor \
         WHERE NOT EXISTS \
           (SELECT 1 \
            FROM UserAchievement \
            WHERE UserAchievement.userId = ? \
              AND UserAchievement.achievementId = AchievementPredecessor.successorId)",
    )
    .bind(user_id)
    .fetch_all(&mut **transaction)
    .await?;

    for successor_id in successor_ids {
        grant_successor_if_eligible(transaction, user_id, successor_id).await?;
    }

    Ok(())
}

pub(super) async fn grant_successor_if_eligible(
    transaction: &mut sqlx::Transaction<'_, sqlx::MySql>,
    user_id: i64,
    successor_id: i64,
) -> Result<(), sqlx::Error> {
    if successor_requirements_met(transaction, user_id, successor_id).await? {
        sqlx::query(
            "INSERT IGNORE INTO UserAchievement (userId, achievementId, stepsClaimed) \
             VALUES (?, ?, 0)",
        )
        .bind(user_id)
        .bind(successor_id)
        .execute(&mut **transaction)
        .await?;
    }

    Ok(())
}

async fn successor_requirements_met(
    transaction: &mut sqlx::Transaction<'_, sqlx::MySql>,
    user_id: i64,
    achievement_id: i64,
) -> Result<bool, sqlx::Error> {
    let predecessors = sqlx::query_as::<_, (i64, i32)>(
        "SELECT predecessorId, predecessorStep \
         FROM AchievementPredecessor \
         WHERE successorId = ?",
    )
    .bind(achievement_id)
    .fetch_all(&mut **transaction)
    .await?;

    for (predecessor_id, predecessor_step) in predecessors {
        let steps_claimed: Option<i32> = sqlx::query_scalar(
            "SELECT stepsClaimed \
             FROM UserAchievement \
             WHERE userId = ? AND achievementId = ?",
        )
        .bind(user_id)
        .bind(predecessor_id)
        .fetch_optional(&mut **transaction)
        .await?;

        if !predecessor_step_met(steps_claimed.unwrap_or_default(), predecessor_step) {
            return Ok(false);
        }
    }

    Ok(true)
}

pub(crate) fn predecessor_step_met(steps_claimed: i32, required_step: i32) -> bool {
    steps_claimed >= required_step
}
