use std::collections::HashMap;

use sqlx::MySqlPool;

use crate::{ClaimedOreRewardRecord, ClaimedUserResults, INITIAL_ORE_WALLET_MAX, in_placeholders};

pub async fn claim_user_results(
    pool: &MySqlPool,
    user_id: i64,
) -> Result<ClaimedUserResults, sqlx::Error> {
    let mut transaction = pool.begin().await?;
    let claimable_queues = list_claimable_mining_queues(&mut transaction, user_id).await?;
    let claimed_queues = claimable_queues.len() as u64;
    let mut ore_rewards: HashMap<i64, i32> = HashMap::new();

    if !claimable_queues.is_empty() {
        let queue_rewards = claim_mining_queues_batch(&mut transaction, &claimable_queues).await?;
        for (ore_id, reward) in queue_rewards {
            *ore_rewards.entry(ore_id).or_default() += reward;
        }
    }

    super::pending::reconcile_pending_robot_changes_in_transaction(&mut transaction, user_id)
        .await?;
    let ore_rewards = load_claimed_ore_rewards(&mut transaction, ore_rewards).await?;
    transaction.commit().await?;

    Ok(ClaimedUserResults {
        claimed_queues,
        ore_rewards,
    })
}

/// Distinct user ids with finished, unclaimed mining runs ready for the wallet.
pub async fn list_user_ids_with_claimable_mining_queues(
    pool: &MySqlPool,
) -> Result<Vec<i64>, sqlx::Error> {
    sqlx::query_scalar(
        "SELECT DISTINCT Robot.userId \
         FROM MiningQueue \
         INNER JOIN Robot ON Robot.id = MiningQueue.robotId \
         WHERE MiningQueue.miningEndTime IS NOT NULL \
           AND MiningQueue.miningEndTime <= NOW() \
           AND MiningQueue.claimed = false \
         ORDER BY Robot.userId",
    )
    .fetch_all(pool)
    .await
}

/// Seconds until the next unclaimed mining run finishes, capped at `max_sleep_seconds`.
///
/// When nothing is queued, returns `max_sleep_seconds`.
pub async fn next_wallet_claim_delay_seconds(
    pool: &MySqlPool,
    max_sleep_seconds: u64,
) -> Result<u64, sqlx::Error> {
    let delay: Option<i64> = sqlx::query_scalar(
        "SELECT TIMESTAMPDIFF(SECOND, NOW(), MIN(MiningQueue.miningEndTime)) \
         FROM MiningQueue \
         WHERE MiningQueue.miningEndTime IS NOT NULL \
           AND MiningQueue.miningEndTime > NOW() \
           AND MiningQueue.claimed = false",
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
    let count: i64 = sqlx::query_scalar(
        "SELECT COUNT(*) \
         FROM MiningQueue \
         INNER JOIN Robot ON Robot.id = MiningQueue.robotId \
         WHERE MiningQueue.miningEndTime IS NOT NULL \
           AND MiningQueue.miningEndTime <= NOW() \
           AND Robot.userId = ? \
           AND MiningQueue.claimed = false",
    )
    .bind(user_id)
    .fetch_one(pool)
    .await?;

    Ok(count.max(0) as u64)
}

#[derive(Debug, Clone, Copy)]
struct ClaimableMiningQueue {
    mining_queue_id: i64,
    mining_area_id: i64,
    robot_id: i64,
    robot_max_ore: i32,
}

#[derive(Debug, Clone, Copy)]
struct ClaimableMiningOreResult {
    mining_queue_id: i64,
    ore_id: i64,
    amount: i32,
    tax: i32,
}

async fn list_claimable_mining_queues(
    transaction: &mut sqlx::Transaction<'_, sqlx::MySql>,
    user_id: i64,
) -> Result<Vec<ClaimableMiningQueue>, sqlx::Error> {
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
            |(mining_queue_id, mining_area_id, robot_id, robot_max_ore)| ClaimableMiningQueue {
                mining_queue_id,
                mining_area_id,
                robot_id,
                robot_max_ore,
            },
        )
        .collect())
}

async fn load_claimed_ore_rewards(
    transaction: &mut sqlx::Transaction<'_, sqlx::MySql>,
    ore_rewards: HashMap<i64, i32>,
) -> Result<Vec<ClaimedOreRewardRecord>, sqlx::Error> {
    let positive: Vec<(i64, i32)> = ore_rewards
        .into_iter()
        .filter(|(_, reward)| *reward > 0)
        .collect();
    if positive.is_empty() {
        return Ok(Vec::new());
    }

    let ore_ids: Vec<i64> = positive.iter().map(|(ore_id, _)| *ore_id).collect();
    let placeholders = in_placeholders(ore_ids.len());
    let query = format!("SELECT id, oreName FROM Ore WHERE id IN ({placeholders})");
    let mut query_builder = sqlx::query_as::<_, (i64, String)>(&query);
    for ore_id in &ore_ids {
        query_builder = query_builder.bind(ore_id);
    }
    let names: HashMap<i64, String> = query_builder
        .fetch_all(&mut **transaction)
        .await?
        .into_iter()
        .collect();

    let mut rewards: Vec<ClaimedOreRewardRecord> = Vec::with_capacity(positive.len());
    for (ore_id, reward) in positive {
        let Some(ore_name) = names.get(&ore_id).cloned() else {
            return Err(sqlx::Error::RowNotFound);
        };
        rewards.push(ClaimedOreRewardRecord {
            ore_id,
            ore_name,
            reward,
        });
    }

    rewards.sort_by_key(|reward| std::cmp::Reverse(reward.ore_id));
    Ok(rewards)
}

async fn claim_mining_queues_batch(
    transaction: &mut sqlx::Transaction<'_, sqlx::MySql>,
    claimable_queues: &[ClaimableMiningQueue],
) -> Result<Vec<(i64, i32)>, sqlx::Error> {
    let queue_ids: Vec<i64> = claimable_queues
        .iter()
        .map(|queue| queue.mining_queue_id)
        .collect();

    increment_robot_mining_runs_batch(transaction, claimable_queues).await?;
    mark_mining_queues_claimed_batch(transaction, &queue_ids).await?;
    calculate_mining_ore_result_tax_batch(transaction, &queue_ids).await?;
    let ore_results = list_claimable_mining_ore_results_batch(transaction, &queue_ids).await?;

    let mut ore_results_by_queue: HashMap<i64, Vec<ClaimableMiningOreResult>> = HashMap::new();
    for ore_result in ore_results {
        ore_results_by_queue
            .entry(ore_result.mining_queue_id)
            .or_default()
            .push(ore_result);
    }

    let mut rewards = Vec::new();
    let mut lifetime_rows = Vec::new();
    let mut wallet_rows = Vec::new();
    let mut area_updates: Vec<(ClaimableMiningQueue, Vec<ClaimableMiningOreResult>)> = Vec::new();

    for queue in claimable_queues {
        let queue_ore_results = ore_results_by_queue
            .remove(&queue.mining_queue_id)
            .unwrap_or_default();

        for ore_result in &queue_ore_results {
            lifetime_rows.push((
                queue.robot_id,
                ore_result.ore_id,
                ore_result.amount,
                ore_result.tax,
            ));
            let reward = ore_result.amount - ore_result.tax;
            if reward > 0 {
                wallet_rows.push((queue.robot_id, ore_result.ore_id, reward));
                rewards.push((ore_result.ore_id, reward));
            }
        }

        area_updates.push((*queue, queue_ore_results));
    }

    batch_update_mining_area_lifetime_results(transaction, &area_updates).await?;
    batch_upsert_robot_lifetime_results(transaction, &lifetime_rows).await?;
    batch_upsert_user_ore_assets_from_rewards(transaction, &wallet_rows).await?;

    Ok(rewards)
}

async fn increment_robot_mining_runs_batch(
    transaction: &mut sqlx::Transaction<'_, sqlx::MySql>,
    claimable_queues: &[ClaimableMiningQueue],
) -> Result<(), sqlx::Error> {
    let mut run_counts: HashMap<i64, u32> = HashMap::new();
    for queue in claimable_queues {
        *run_counts.entry(queue.robot_id).or_default() += 1;
    }

    for (robot_id, count) in run_counts {
        sqlx::query("UPDATE Robot SET totalMiningRuns = totalMiningRuns + ? WHERE id = ?")
            .bind(count)
            .bind(robot_id)
            .execute(&mut **transaction)
            .await?;
    }

    Ok(())
}

async fn batch_upsert_robot_lifetime_results(
    transaction: &mut sqlx::Transaction<'_, sqlx::MySql>,
    rows: &[(i64, i64, i32, i32)],
) -> Result<(), sqlx::Error> {
    const CHUNK: usize = 64;
    for chunk in rows.chunks(CHUNK) {
        if chunk.is_empty() {
            continue;
        }
        let value_placeholders = chunk
            .iter()
            .map(|_| "(?, ?, ?, ?)")
            .collect::<Vec<_>>()
            .join(", ");
        let query = format!(
            "INSERT INTO RobotLifetimeResult (robotId, oreId, amount, tax) VALUES {value_placeholders} \
             ON DUPLICATE KEY UPDATE \
             amount = amount + VALUES(amount), \
             tax = tax + VALUES(tax)"
        );
        let mut query_builder = sqlx::query(&query);
        for (robot_id, ore_id, amount, tax) in chunk {
            query_builder = query_builder
                .bind(robot_id)
                .bind(ore_id)
                .bind(amount)
                .bind(tax);
        }
        query_builder.execute(&mut **transaction).await?;
    }
    Ok(())
}

async fn batch_upsert_user_ore_assets_from_rewards(
    transaction: &mut sqlx::Transaction<'_, sqlx::MySql>,
    rows: &[(i64, i64, i32)],
) -> Result<(), sqlx::Error> {
    if rows.is_empty() {
        return Ok(());
    }

    let mut reward_by_robot_ore: HashMap<(i64, i64), i32> = HashMap::new();
    for &(robot_id, ore_id, reward) in rows {
        if reward > 0 {
            *reward_by_robot_ore.entry((robot_id, ore_id)).or_default() += reward;
        }
    }
    if reward_by_robot_ore.is_empty() {
        return Ok(());
    }

    let robot_ids: Vec<i64> = reward_by_robot_ore
        .keys()
        .map(|(robot_id, _)| *robot_id)
        .collect::<std::collections::HashSet<_>>()
        .into_iter()
        .collect();
    let placeholders = in_placeholders(robot_ids.len());
    let query = format!("SELECT id, userId FROM Robot WHERE id IN ({placeholders})");
    let mut query_builder = sqlx::query_as::<_, (i64, i64)>(&query);
    for robot_id in &robot_ids {
        query_builder = query_builder.bind(robot_id);
    }
    let robot_users: HashMap<i64, i64> = query_builder
        .fetch_all(&mut **transaction)
        .await?
        .into_iter()
        .collect();

    let mut reward_by_user_ore: HashMap<(i64, i64), i32> = HashMap::new();
    for ((robot_id, ore_id), reward) in reward_by_robot_ore {
        let Some(user_id) = robot_users.get(&robot_id).copied() else {
            return Err(sqlx::Error::RowNotFound);
        };
        *reward_by_user_ore.entry((user_id, ore_id)).or_default() += reward;
    }

    let upsert_rows: Vec<(i64, i64, i32)> = reward_by_user_ore
        .into_iter()
        .map(|((user_id, ore_id), reward)| (user_id, ore_id, reward))
        .collect();

    const CHUNK: usize = 64;
    for chunk in upsert_rows.chunks(CHUNK) {
        if chunk.is_empty() {
            continue;
        }

        // Existing wallets need the raw reward on UPDATE; new rows need LEAST(reward, INITIAL).
        // Multi-VALUES cannot bind an extra per-row UPDATE param, so amount is raw for
        // duplicates (read via row alias) and pre-capped only for true inserts.
        let pair_placeholders = chunk
            .iter()
            .map(|_| "(?, ?)")
            .collect::<Vec<_>>()
            .join(", ");
        let existing_query = format!(
            "SELECT userId, oreId FROM UserOreAsset WHERE (userId, oreId) IN ({pair_placeholders})"
        );
        let mut existing_builder = sqlx::query_as::<_, (i64, i64)>(&existing_query);
        for (user_id, ore_id, _) in chunk {
            existing_builder = existing_builder.bind(user_id).bind(ore_id);
        }
        let existing: std::collections::HashSet<(i64, i64)> = existing_builder
            .fetch_all(&mut **transaction)
            .await?
            .into_iter()
            .collect();

        // INSERT amount is LEAST(reward, INITIAL) for new rows; raw reward for existing
        // rows so ON DUPLICATE KEY UPDATE can add the full delta via new_row.amount.
        let value_placeholders = chunk
            .iter()
            .map(|_| "(?, ?, ?, ?)")
            .collect::<Vec<_>>()
            .join(", ");
        let query = format!(
            "INSERT INTO UserOreAsset (userId, oreId, amount, maxAllowed) VALUES {value_placeholders} AS new_row \
             ON DUPLICATE KEY UPDATE \
             amount = LEAST(UserOreAsset.maxAllowed, UserOreAsset.amount + new_row.amount)"
        );
        let mut query_builder = sqlx::query(&query);
        for (user_id, ore_id, reward) in chunk {
            let amount_value = if existing.contains(&(*user_id, *ore_id)) {
                *reward
            } else {
                (*reward).min(INITIAL_ORE_WALLET_MAX)
            };
            query_builder = query_builder
                .bind(user_id)
                .bind(ore_id)
                .bind(amount_value)
                .bind(INITIAL_ORE_WALLET_MAX);
        }
        query_builder.execute(&mut **transaction).await?;
    }

    Ok(())
}

async fn mark_mining_queues_claimed_batch(
    transaction: &mut sqlx::Transaction<'_, sqlx::MySql>,
    queue_ids: &[i64],
) -> Result<(), sqlx::Error> {
    if queue_ids.is_empty() {
        return Ok(());
    }

    let placeholders = in_placeholders(queue_ids.len());
    let query = format!("UPDATE MiningQueue SET claimed = true WHERE id IN ({placeholders})");
    let mut query_builder = sqlx::query(&query);
    for queue_id in queue_ids {
        query_builder = query_builder.bind(queue_id);
    }
    query_builder.execute(&mut **transaction).await?;

    Ok(())
}

async fn calculate_mining_ore_result_tax_batch(
    transaction: &mut sqlx::Transaction<'_, sqlx::MySql>,
    queue_ids: &[i64],
) -> Result<(), sqlx::Error> {
    if queue_ids.is_empty() {
        return Ok(());
    }

    let placeholders = in_placeholders(queue_ids.len());
    let query = format!(
        "UPDATE MiningOreResult \
         INNER JOIN MiningQueue ON MiningQueue.id = MiningOreResult.miningQueueId \
         INNER JOIN MiningArea ON MiningArea.id = MiningQueue.miningAreaId \
         SET MiningOreResult.tax = \
             FLOOR(GREATEST(MiningOreResult.amount - MiningOreResult.depotAmount, 0) \
                   * MiningArea.taxRate / 100) \
           + FLOOR(LEAST(MiningOreResult.depotAmount, MiningOreResult.amount) \
                   * MiningArea.depotTaxRate / 100) \
         WHERE MiningOreResult.miningQueueId IN ({placeholders})"
    );
    let mut query_builder = sqlx::query(&query);
    for queue_id in queue_ids {
        query_builder = query_builder.bind(queue_id);
    }
    query_builder.execute(&mut **transaction).await?;

    Ok(())
}

async fn list_claimable_mining_ore_results_batch(
    transaction: &mut sqlx::Transaction<'_, sqlx::MySql>,
    queue_ids: &[i64],
) -> Result<Vec<ClaimableMiningOreResult>, sqlx::Error> {
    if queue_ids.is_empty() {
        return Ok(Vec::new());
    }

    let placeholders = in_placeholders(queue_ids.len());
    let query = format!(
        "SELECT miningQueueId, oreId, amount, COALESCE(tax, 0) \
         FROM MiningOreResult \
         WHERE miningQueueId IN ({placeholders}) \
         ORDER BY miningQueueId, oreId"
    );
    let mut query_builder = sqlx::query_as::<_, (i64, i64, i32, i32)>(&query);
    for queue_id in queue_ids {
        query_builder = query_builder.bind(queue_id);
    }
    let rows = query_builder.fetch_all(&mut **transaction).await?;

    Ok(rows
        .into_iter()
        .map(
            |(mining_queue_id, ore_id, amount, tax)| ClaimableMiningOreResult {
                mining_queue_id,
                ore_id,
                amount,
                tax,
            },
        )
        .collect())
}

async fn batch_update_mining_area_lifetime_results(
    transaction: &mut sqlx::Transaction<'_, sqlx::MySql>,
    area_updates: &[(ClaimableMiningQueue, Vec<ClaimableMiningOreResult>)],
) -> Result<(), sqlx::Error> {
    if area_updates.is_empty() {
        return Ok(());
    }

    let mut runs_by_area: HashMap<i64, u32> = HashMap::new();
    let mut amount_by_area_ore: HashMap<(i64, i64), i32> = HashMap::new();
    let mut area_ids: Vec<i64> = Vec::new();

    for (queue, ore_results) in area_updates {
        *runs_by_area.entry(queue.mining_area_id).or_default() += 1;
        if !area_ids.contains(&queue.mining_area_id) {
            area_ids.push(queue.mining_area_id);
        }
        for ore_result in ore_results {
            *amount_by_area_ore
                .entry((queue.mining_area_id, ore_result.ore_id))
                .or_default() += ore_result.amount;
        }
    }

    let mut supply_ores_by_area: HashMap<i64, Vec<i64>> = HashMap::new();
    for area_id in &area_ids {
        let ore_ids = sqlx::query_scalar::<_, i64>(
            "SELECT DISTINCT oreId \
             FROM MiningAreaOreSupply \
             WHERE miningAreaId = ? \
             ORDER BY oreId",
        )
        .bind(area_id)
        .fetch_all(&mut **transaction)
        .await?;
        supply_ores_by_area.insert(*area_id, ore_ids);
    }

    let mut container_by_area_ore: HashMap<(i64, i64), i32> = HashMap::new();
    for (queue, _) in area_updates {
        let Some(supply_ores) = supply_ores_by_area.get(&queue.mining_area_id) else {
            continue;
        };
        for ore_id in supply_ores {
            *container_by_area_ore
                .entry((queue.mining_area_id, *ore_id))
                .or_default() += queue.robot_max_ore;
        }
    }

    for (area_id, run_count) in &runs_by_area {
        sqlx::query(
            "UPDATE MiningAreaLifetimeResult \
             SET totalRuns = totalRuns + ? \
             WHERE miningAreaId = ?",
        )
        .bind(run_count)
        .bind(area_id)
        .execute(&mut **transaction)
        .await?;
    }

    for area_id in &area_ids {
        let run_count = runs_by_area.get(area_id).copied().unwrap_or(0);
        let supply_ores = supply_ores_by_area
            .get(area_id)
            .cloned()
            .unwrap_or_default();
        for ore_id in supply_ores {
            let amount = amount_by_area_ore
                .get(&(*area_id, ore_id))
                .copied()
                .unwrap_or(0);
            let container = container_by_area_ore
                .get(&(*area_id, ore_id))
                .copied()
                .unwrap_or(0);
            sqlx::query(
                "INSERT INTO MiningAreaLifetimeResult \
                 (miningAreaId, oreId, totalAmount, totalContainerSize, totalRuns) \
                 VALUES (?, ?, ?, ?, \
                         COALESCE((SELECT totalRuns \
                                   FROM MiningAreaLifetimeResult AS existing \
                                   WHERE existing.miningAreaId = ? \
                                   LIMIT 1), ?)) \
                 ON DUPLICATE KEY UPDATE \
                 totalAmount = totalAmount + VALUES(totalAmount), \
                 totalContainerSize = totalContainerSize + VALUES(totalContainerSize)",
            )
            .bind(area_id)
            .bind(ore_id)
            .bind(amount)
            .bind(container)
            .bind(area_id)
            .bind(run_count)
            .execute(&mut **transaction)
            .await?;
        }
    }

    Ok(())
}
