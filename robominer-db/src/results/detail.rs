use sqlx::MySqlPool;

use crate::{MiningResultActionStateRecord, MiningResultOreStateRecord};

pub async fn list_mining_result_ore_states_for_user(
    pool: &MySqlPool,
    user_id: i64,
    maximum_results: i64,
) -> Result<Vec<MiningResultOreStateRecord>, sqlx::Error> {
    sqlx::query_as::<_, (i64, i64, String, i32, i32, i32)>(
        "SELECT MiningQueue.id, MiningOreResult.oreId, Ore.oreName, \
                MiningOreResult.amount, COALESCE(MiningOreResult.tax, 0), \
                MiningOreResult.amount - COALESCE(MiningOreResult.tax, 0) \
         FROM MiningQueue \
         INNER JOIN Robot ON Robot.id = MiningQueue.robotId \
         INNER JOIN MiningOreResult ON MiningOreResult.miningQueueId = MiningQueue.id \
         INNER JOIN Ore ON Ore.id = MiningOreResult.oreId \
         WHERE Robot.userId = ? \
           AND MiningQueue.claimed = TRUE \
           AND (SELECT COUNT(*) \
                FROM MiningQueue RankedQueue \
                WHERE RankedQueue.robotId = MiningQueue.robotId \
                  AND RankedQueue.claimed = TRUE \
                  AND (RankedQueue.miningEndTime > MiningQueue.miningEndTime \
                       OR (RankedQueue.miningEndTime = MiningQueue.miningEndTime \
                           AND RankedQueue.id <= MiningQueue.id))) <= ? \
         ORDER BY MiningQueue.robotId, MiningQueue.miningEndTime DESC, MiningQueue.id, Ore.id",
    )
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
    sqlx::query_as::<_, (i64, i32, i32)>(
        "SELECT MiningQueue.id, RobotActionsDone.actionType, RobotActionsDone.amount \
         FROM MiningQueue \
         INNER JOIN Robot ON Robot.id = MiningQueue.robotId \
         INNER JOIN RobotActionsDone ON RobotActionsDone.miningQueueId = MiningQueue.id \
         WHERE Robot.userId = ? \
           AND MiningQueue.claimed = TRUE \
           AND (SELECT COUNT(*) \
                FROM MiningQueue RankedQueue \
                WHERE RankedQueue.robotId = MiningQueue.robotId \
                  AND RankedQueue.claimed = TRUE \
                  AND (RankedQueue.miningEndTime > MiningQueue.miningEndTime \
                       OR (RankedQueue.miningEndTime = MiningQueue.miningEndTime \
                           AND RankedQueue.id <= MiningQueue.id))) <= ? \
         ORDER BY MiningQueue.robotId, MiningQueue.miningEndTime DESC, MiningQueue.id, \
                  RobotActionsDone.actionType",
    )
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
