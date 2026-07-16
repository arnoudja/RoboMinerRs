use sqlx::MySqlPool;

use crate::ClaimedMiningQueueCleanupSummary;

use super::CLAIMED_MINING_QUEUE_RETENTION;

pub async fn cleanup_old_claimed_mining_queue_items_for_robot(
    pool: &MySqlPool,
    robot_id: i64,
) -> Result<ClaimedMiningQueueCleanupSummary, sqlx::Error> {
    let mut transaction = pool.begin().await?;
    let summary = cleanup_old_claimed_mining_queue_items(&mut transaction, robot_id).await?;
    transaction.commit().await?;

    Ok(summary)
}

pub(super) async fn cleanup_old_claimed_mining_queue_items(
    transaction: &mut sqlx::Transaction<'_, sqlx::MySql>,
    robot_id: i64,
) -> Result<ClaimedMiningQueueCleanupSummary, sqlx::Error> {
    let old_items = sqlx::query_as::<_, (i64, Option<i64>)>(
        "SELECT id, rallyResultId \
         FROM MiningQueue \
         WHERE robotId = ? \
           AND claimed = true \
         ORDER BY id DESC \
         LIMIT ?, 100000",
    )
    .bind(robot_id)
    .bind(CLAIMED_MINING_QUEUE_RETENTION)
    .fetch_all(&mut **transaction)
    .await?;

    let mut summary = ClaimedMiningQueueCleanupSummary::default();

    for (mining_queue_id, rally_result_id) in old_items {
        sqlx::query("DELETE FROM MiningQueue WHERE id = ?")
            .bind(mining_queue_id)
            .execute(&mut **transaction)
            .await?;
        summary.queues_deleted += 1;

        if let Some(rally_result_id) = rally_result_id
            && !rally_result_still_referenced(transaction, rally_result_id).await?
        {
            sqlx::query("DELETE FROM RallyResult WHERE id = ?")
                .bind(rally_result_id)
                .execute(&mut **transaction)
                .await?;
            summary.rally_results_deleted += 1;
        }
    }

    Ok(summary)
}

async fn rally_result_still_referenced(
    transaction: &mut sqlx::Transaction<'_, sqlx::MySql>,
    rally_result_id: i64,
) -> Result<bool, sqlx::Error> {
    let referenced: Option<i64> =
        sqlx::query_scalar("SELECT id FROM MiningQueue WHERE rallyResultId = ? LIMIT 1")
            .bind(rally_result_id)
            .fetch_optional(&mut **transaction)
            .await?;

    Ok(referenced.is_some())
}
