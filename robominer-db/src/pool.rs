//! Pool rally loadouts and completed-pool persist helpers.
//!
//! Primary entry points: [`get_pool`], [`list_pool_items`],
//! [`persist_completed_pool_rally`].

use sqlx::MySqlPool;

use crate::{
    CompletedPoolItemOreRecord, CompletedPoolItemRecord, CompletedPoolRallyRecord, PoolItemRecord,
    PoolRecord,
};

#[derive(sqlx::FromRow)]
struct PoolRow {
    id: i64,
    #[sqlx(rename = "miningAreaId")]
    mining_area_id: i64,
    #[sqlx(rename = "requiredRuns")]
    required_runs: i32,
}

impl From<PoolRow> for PoolRecord {
    fn from(row: PoolRow) -> Self {
        Self {
            id: row.id,
            mining_area_id: row.mining_area_id,
            required_runs: row.required_runs,
        }
    }
}

#[derive(Debug, Clone, sqlx::FromRow)]
pub(crate) struct PoolItemRow {
    pub(crate) id: i64,
    #[sqlx(rename = "poolId")]
    pub(crate) pool_id: i64,
    #[sqlx(rename = "robotId")]
    pub(crate) robot_id: i64,
    #[sqlx(rename = "sourceCode")]
    pub(crate) source_code: String,
    #[sqlx(rename = "totalScore")]
    pub(crate) total_score: f64,
    #[sqlx(rename = "runsDone")]
    pub(crate) runs_done: i32,
}

impl From<PoolItemRow> for PoolItemRecord {
    fn from(row: PoolItemRow) -> Self {
        Self {
            id: row.id,
            pool_id: row.pool_id,
            robot_id: row.robot_id,
            source_code: row.source_code,
            total_score: row.total_score,
            runs_done: row.runs_done,
        }
    }
}

pub(crate) fn pool_item_rows(rows: Vec<PoolItemRow>) -> Vec<PoolItemRecord> {
    rows.into_iter().map(PoolItemRecord::from).collect()
}

/// Keep only the cohort that shares the lowest `runs_done` value.
pub(crate) fn next_pool_rally_item_rows(rows: Vec<PoolItemRow>) -> Vec<PoolItemRecord> {
    let first_runs_done = rows.first().map(|row| row.runs_done);

    rows.into_iter()
        .filter(|row| Some(row.runs_done) == first_runs_done)
        .map(PoolItemRecord::from)
        .collect()
}

pub async fn get_pool(pool: &MySqlPool, pool_id: i64) -> Result<Option<PoolRecord>, sqlx::Error> {
    sqlx::query_as::<_, PoolRow>(
        "SELECT id, miningAreaId, requiredRuns \
         FROM Pool \
         WHERE id = ?",
    )
    .bind(pool_id)
    .fetch_optional(pool)
    .await
    .map(|row| row.map(PoolRecord::from))
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
    sqlx::query!(
        "UPDATE PoolItem \
         SET totalScore = totalScore + ?, runsDone = runsDone + 1 \
         WHERE id = ?",
        item.score,
        item.pool_item_id
    )
    .execute(&mut **transaction)
    .await?;

    Ok(())
}

async fn upsert_pool_item_mining_total(
    transaction: &mut sqlx::Transaction<'_, sqlx::MySql>,
    pool_item_id: i64,
    ore_result: &CompletedPoolItemOreRecord,
) -> Result<(), sqlx::Error> {
    sqlx::query!(
        "INSERT INTO PoolItemMiningTotals \
         (poolItemId, oreId, totalMined) \
         VALUES (?, ?, ?) \
         ON DUPLICATE KEY UPDATE \
         totalMined = totalMined + VALUES(totalMined)",
        pool_item_id,
        ore_result.ore_id,
        ore_result.amount
    )
    .execute(&mut **transaction)
    .await?;

    Ok(())
}
