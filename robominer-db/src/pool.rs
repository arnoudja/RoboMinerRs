use sqlx::MySqlPool;

use crate::mappers::{
    PoolItemRow, PoolRow, next_pool_rally_item_rows, pool_item_rows, pool_record,
};
use crate::{
    CompletedPoolItemOreRecord, CompletedPoolItemRecord, CompletedPoolRallyRecord, PoolItemRecord,
    PoolRecord,
};

pub async fn get_pool(pool: &MySqlPool, pool_id: i64) -> Result<Option<PoolRecord>, sqlx::Error> {
    sqlx::query_as::<_, PoolRow>(
        "SELECT id, miningAreaId, requiredRuns \
         FROM Pool \
         WHERE id = ?",
    )
    .bind(pool_id)
    .fetch_optional(pool)
    .await
    .map(|row| row.map(pool_record))
}

pub async fn list_pool_items(
    pool: &MySqlPool,
    pool_id: i64,
) -> Result<Vec<PoolItemRecord>, sqlx::Error> {
    sqlx::query_as::<_, PoolItemRow>(
        "SELECT id, poolId, robotId, sourceCode, totalScore, runsDone \
         FROM PoolItem \
         WHERE poolId = ? \
         ORDER BY id",
    )
    .bind(pool_id)
    .fetch_all(pool)
    .await
    .map(pool_item_rows)
}

pub async fn list_next_pool_rally_items(
    pool: &MySqlPool,
    pool_id: i64,
) -> Result<Vec<PoolItemRecord>, sqlx::Error> {
    let rows = sqlx::query_as::<_, PoolItemRow>(
        "SELECT id, poolId, robotId, sourceCode, totalScore, runsDone \
         FROM PoolItem \
         WHERE poolId = ? \
         ORDER BY runsDone ASC, totalScore DESC, id ASC \
         LIMIT 4",
    )
    .bind(pool_id)
    .fetch_all(pool)
    .await?;

    Ok(next_pool_rally_item_rows(rows))
}
pub async fn persist_completed_pool_rally(
    pool: &MySqlPool,
    rally: &CompletedPoolRallyRecord,
) -> Result<(), sqlx::Error> {
    let mut transaction = pool.begin().await?;

    for item in &rally.items {
        update_pool_item_for_completed_rally(&mut transaction, item).await?;

        for ore_result in item
            .ore_results
            .iter()
            .filter(|ore_result| ore_result.amount > 0)
        {
            upsert_pool_item_mining_total(&mut transaction, item.pool_item_id, ore_result).await?;
        }
    }

    transaction.commit().await?;

    Ok(())
}
async fn update_pool_item_for_completed_rally(
    transaction: &mut sqlx::Transaction<'_, sqlx::MySql>,
    item: &CompletedPoolItemRecord,
) -> Result<(), sqlx::Error> {
    sqlx::query(
        "UPDATE PoolItem \
         SET totalScore = totalScore + ?, runsDone = runsDone + 1 \
         WHERE id = ?",
    )
    .bind(item.score)
    .bind(item.pool_item_id)
    .execute(&mut **transaction)
    .await?;

    Ok(())
}

async fn upsert_pool_item_mining_total(
    transaction: &mut sqlx::Transaction<'_, sqlx::MySql>,
    pool_item_id: i64,
    ore_result: &CompletedPoolItemOreRecord,
) -> Result<(), sqlx::Error> {
    sqlx::query(
        "INSERT INTO PoolItemMiningTotals \
         (poolItemId, oreId, totalMined) \
         VALUES (?, ?, ?) \
         ON DUPLICATE KEY UPDATE \
         totalMined = totalMined + VALUES(totalMined)",
    )
    .bind(pool_item_id)
    .bind(ore_result.ore_id)
    .bind(ore_result.amount)
    .execute(&mut **transaction)
    .await?;

    Ok(())
}
