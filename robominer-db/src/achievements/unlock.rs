use crate::assert_sql_safe;
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

    if predecessors.is_empty() {
        return Ok(true);
    }

    let predecessor_ids: Vec<i64> = predecessors.iter().map(|(id, _)| *id).collect();
    let placeholders = crate::in_placeholders(predecessor_ids.len());
    let query = format!(
        "SELECT achievementId, stepsClaimed \
         FROM UserAchievement \
         WHERE userId = ? AND achievementId IN ({placeholders})"
    );
    let mut query_builder = sqlx::query_as::<_, (i64, i32)>(assert_sql_safe(query)).bind(user_id);
    for predecessor_id in &predecessor_ids {
        query_builder = query_builder.bind(predecessor_id);
    }
    let claimed_steps: std::collections::HashMap<i64, i32> = query_builder
        .fetch_all(&mut **transaction)
        .await?
        .into_iter()
        .collect();

    for (predecessor_id, predecessor_step) in predecessors {
        let steps_claimed = claimed_steps
            .get(&predecessor_id)
            .copied()
            .unwrap_or_default();
        if !predecessor_step_met(steps_claimed, predecessor_step) {
            return Ok(false);
        }
    }

    Ok(true)
}

pub(crate) fn predecessor_step_met(steps_claimed: i32, required_step: i32) -> bool {
    steps_claimed >= required_step
}
