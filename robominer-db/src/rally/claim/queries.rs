use sqlx::MySqlPool;

/// Distinct user ids with finished, unclaimed mining runs ready for the wallet.
pub async fn list_user_ids_with_claimable_mining_queues(
    pool: &MySqlPool,
) -> Result<Vec<i64>, sqlx::Error> {
    let rows = sqlx::query_scalar!(
        r#"
SELECT DISTINCT Robot.userId AS "user_id!: i64"
FROM MiningQueue
INNER JOIN Robot ON Robot.id = MiningQueue.robotId
WHERE MiningQueue.miningEndTime IS NOT NULL
  AND MiningQueue.miningEndTime <= NOW()
  AND MiningQueue.claimed = false
ORDER BY Robot.userId
        "#
    )
    .fetch_all(pool)
    .await?;

    Ok(rows)
}

/// Seconds until the next unclaimed mining run finishes, capped at `max_sleep_seconds`.
///
/// When finished unclaimed runs are already ready, returns `1` so poll loops retry promptly
/// without a busy-spin. When nothing is queued, returns `max_sleep_seconds`.
pub async fn next_wallet_claim_delay_seconds(
    pool: &MySqlPool,
    max_sleep_seconds: u64,
) -> Result<u64, sqlx::Error> {
    let ready_now = sqlx::query_scalar!(
        r#"
SELECT COUNT(*) AS "count!: i64"
FROM MiningQueue
WHERE MiningQueue.miningEndTime IS NOT NULL
  AND MiningQueue.miningEndTime <= NOW()
  AND MiningQueue.claimed = false
        "#
    )
    .fetch_one(pool)
    .await?;
    if ready_now > 0 {
        return Ok(1.min(max_sleep_seconds));
    }

    let delay = sqlx::query_scalar!(
        r#"
SELECT TIMESTAMPDIFF(SECOND, NOW(), MIN(MiningQueue.miningEndTime)) AS "seconds: i64"
FROM MiningQueue
WHERE MiningQueue.miningEndTime IS NOT NULL
  AND MiningQueue.miningEndTime > NOW()
  AND MiningQueue.claimed = false
        "#
    )
    .fetch_one(pool)
    .await?;

    Ok(delay
        .map(|seconds| seconds.max(1) as u64)
        .unwrap_or(max_sleep_seconds)
        .min(max_sleep_seconds))
}

/// Read-only count of finished mining runs waiting to be claimed into the wallet.
pub async fn count_claimable_mining_queues(
    pool: &MySqlPool,
    user_id: i64,
) -> Result<u64, sqlx::Error> {
    let count = sqlx::query_scalar!(
        r#"
SELECT COUNT(*) AS "count!: i64"
FROM MiningQueue
INNER JOIN Robot ON Robot.id = MiningQueue.robotId
WHERE MiningQueue.miningEndTime IS NOT NULL
  AND MiningQueue.miningEndTime <= NOW()
  AND Robot.userId = ?
  AND MiningQueue.claimed = false
        "#,
        user_id
    )
    .fetch_one(pool)
    .await?;

    Ok(count.max(0) as u64)
}

pub(super) async fn list_claimable_mining_queues(
    transaction: &mut sqlx::Transaction<'_, sqlx::MySql>,
    user_id: i64,
) -> Result<Vec<super::types::ClaimableMiningQueue>, sqlx::Error> {
    // FOR UPDATE + dynamic claim batches stay on runtime queries; schema is still
    // covered by the compile-checked pool queries above and DB integration tests.
    let rows = sqlx::query_as::<_, (i64, i64, i64, i32)>(
        "SELECT MiningQueue.id, MiningQueue.miningAreaId, MiningQueue.robotId, Robot.maxOre \
         FROM MiningQueue \
         INNER JOIN Robot ON Robot.id = MiningQueue.robotId \
         WHERE MiningQueue.miningEndTime IS NOT NULL \
           AND MiningQueue.miningEndTime <= NOW() \
           AND Robot.userId = ? \
           AND MiningQueue.claimed = false \
         ORDER BY MiningQueue.miningEndTime, MiningQueue.id \
         FOR UPDATE",
    )
    .bind(user_id)
    .fetch_all(&mut **transaction)
    .await?;

    Ok(rows
        .into_iter()
        .map(
            |(mining_queue_id, mining_area_id, robot_id, robot_max_ore)| {
                super::types::ClaimableMiningQueue {
                    mining_queue_id,
                    mining_area_id,
                    robot_id,
                    robot_max_ore,
                }
            },
        )
        .collect())
}
