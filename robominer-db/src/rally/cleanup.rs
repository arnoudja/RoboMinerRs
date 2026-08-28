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

    if old_items.is_empty() {
        return Ok(summary);
    }

    let queue_ids: Vec<i64> = old_items.iter().map(|(id, _)| *id).collect();
    let placeholders = crate::in_placeholders(queue_ids.len());
    let delete_query = format!("DELETE FROM MiningQueue WHERE id IN ({placeholders})");
    let mut delete_builder = sqlx::query(&delete_query);
    for queue_id in &queue_ids {
        delete_builder = delete_builder.bind(queue_id);
    }
    delete_builder.execute(&mut **transaction).await?;
    summary.queues_deleted = queue_ids.len() as u64;

    for (_, rally_result_id) in old_items {
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
