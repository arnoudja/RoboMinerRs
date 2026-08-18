use sqlx::MySqlPool;

use crate::{
    MiningResultActionStateRecord, MiningResultAreaOreSlotRecord, MiningResultOreStateRecord,
};

use super::RECENT_CLAIMED_QUEUE_RANK_FILTER;

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
         INNER JOIN Robot ON Robot.id = MiningQueue.robotId \
         INNER JOIN MiningOreResult ON MiningOreResult.miningQueueId = MiningQueue.id \
         INNER JOIN Ore ON Ore.id = MiningOreResult.oreId \
         WHERE {RECENT_CLAIMED_QUEUE_RANK_FILTER} \
         ORDER BY MiningQueue.robotId, MiningQueue.miningEndTime DESC, MiningQueue.id, Ore.id"
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
         INNER JOIN Robot ON Robot.id = MiningQueue.robotId \
         INNER JOIN RobotActionsDone ON RobotActionsDone.miningQueueId = MiningQueue.id \
         WHERE {RECENT_CLAIMED_QUEUE_RANK_FILTER} \
         ORDER BY MiningQueue.robotId, MiningQueue.miningEndTime DESC, MiningQueue.id, \
                  RobotActionsDone.actionType"
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

pub async fn list_mining_result_area_ore_slots_for_user(
    pool: &MySqlPool,
    user_id: i64,
    maximum_results: i64,
) -> Result<Vec<MiningResultAreaOreSlotRecord>, sqlx::Error> {
    sqlx::query_as::<_, (i64, i64, String)>(&format!(
        "SELECT DISTINCT MiningArea.id, MiningAreaOreSupply.oreId, Ore.oreName \
         FROM MiningQueue \
         INNER JOIN Robot ON Robot.id = MiningQueue.robotId \
         INNER JOIN MiningArea ON MiningArea.id = MiningQueue.miningAreaId \
         INNER JOIN MiningAreaOreSupply ON MiningAreaOreSupply.miningAreaId = MiningArea.id \
         INNER JOIN Ore ON Ore.id = MiningAreaOreSupply.oreId \
         WHERE {RECENT_CLAIMED_QUEUE_RANK_FILTER} \
         ORDER BY MiningArea.id, MiningAreaOreSupply.oreId DESC, Ore.oreName"
    ))
    .bind(user_id)
    .bind(maximum_results)
    .fetch_all(pool)
    .await
    .map(|rows| {
        rows.into_iter()
            .map(
                |(mining_area_id, ore_id, ore_name)| MiningResultAreaOreSlotRecord {
                    mining_area_id,
                    ore_id,
                    ore_name,
                },
            )
            .collect()
    })
}
