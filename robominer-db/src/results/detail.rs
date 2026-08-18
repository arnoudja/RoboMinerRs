use sqlx::MySqlPool;

use crate::{MiningResultActionStateRecord, MiningResultOreStateRecord};

pub async fn list_mining_result_ore_states_for_user(
    pool: &MySqlPool,
    user_id: i64,
    maximum_results: i64,
) -> Result<Vec<MiningResultOreStateRecord>, sqlx::Error> {
    sqlx::query_as::<_, (i64, i64, String, i32, i32, i32)>(&format!(
        "SELECT MiningQueue.id, MiningOreResult.oreId, Ore.oreName, \
                MiningOreResult.amount, COALESCE(MiningOreResult.tax, 0), \
                MiningOreResult.amount - COALESCE(MiningOreResult.tax, 0) \
         FROM MiningQueue \
         INNER JOIN MiningOreResult ON MiningOreResult.miningQueueId = MiningQueue.id \
         INNER JOIN Ore ON Ore.id = MiningOreResult.oreId \
         WHERE MiningQueue.id IN ({}) \
         ORDER BY MiningQueue.miningEndTime DESC, MiningQueue.id DESC, Ore.id",
        super::RECENT_CLAIMED_MINING_QUEUE_IDS_FOR_USER
    ))
    .bind(user_id)
    .bind(maximum_results)
    .fetch_all(pool)
    .await
    .map(|rows| {
        rows.into_iter()
            .map(|(mining_queue_id, ore_id, ore_name, amount, tax, reward)| {
                MiningResultOreStateRecord {
                    mining_queue_id,
                    ore_id,
                    ore_name,
                    amount,
                    tax,
                    reward,
                }
            })
            .collect()
    })
}

pub async fn list_mining_result_action_states_for_user(
    pool: &MySqlPool,
    user_id: i64,
    maximum_results: i64,
) -> Result<Vec<MiningResultActionStateRecord>, sqlx::Error> {
    sqlx::query_as::<_, (i64, i32, i32)>(&format!(
        "SELECT MiningQueue.id, RobotActionsDone.actionType, RobotActionsDone.amount \
         FROM MiningQueue \
         INNER JOIN RobotActionsDone ON RobotActionsDone.miningQueueId = MiningQueue.id \
         WHERE MiningQueue.id IN ({}) \
         ORDER BY MiningQueue.miningEndTime DESC, MiningQueue.id DESC, \
                  RobotActionsDone.actionType",
        super::RECENT_CLAIMED_MINING_QUEUE_IDS_FOR_USER
    ))
    .bind(user_id)
    .bind(maximum_results)
    .fetch_all(pool)
    .await
    .map(|rows| {
        rows.into_iter()
            .map(
                |(mining_queue_id, action_type, amount)| MiningResultActionStateRecord {
                    mining_queue_id,
                    action_type,
                    amount,
                },
            )
            .collect()
    })
}
