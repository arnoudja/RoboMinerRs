use sqlx::MySqlPool;

use crate::{MiningResultActionStateRecord, MiningResultOreStateRecord, MiningResultStateRecord};

pub async fn list_mining_result_states_for_user(
    pool: &MySqlPool,
    user_id: i64,
    maximum_results: i64,
) -> Result<Vec<MiningResultStateRecord>, sqlx::Error> {
    sqlx::query_as::<_, (i64, i64, String, Option<i64>, f64, i32, i32, i32, i64, i64)>(
        "SELECT MiningQueue.robotId, MiningQueue.id, MiningArea.areaName, \
                MiningQueue.rallyResultId, COALESCE(MiningQueue.score, 0.0), \
                CAST(COALESCE(SUM(MiningOreResult.amount), 0) AS SIGNED), \
                CAST(COALESCE(SUM(COALESCE(MiningOreResult.tax, 0)), 0) AS SIGNED), \
                CAST(COALESCE(SUM(MiningOreResult.amount - COALESCE(MiningOreResult.tax, 0)), 0) AS SIGNED), \
                CAST(UNIX_TIMESTAMP(MiningQueue.creationTime) * 1000 AS SIGNED), \
                CAST(UNIX_TIMESTAMP(MiningQueue.miningEndTime) * 1000 AS SIGNED) \
         FROM MiningQueue \
         INNER JOIN Robot ON Robot.id = MiningQueue.robotId \
         INNER JOIN MiningArea ON MiningArea.id = MiningQueue.miningAreaId \
         LEFT OUTER JOIN MiningOreResult ON MiningOreResult.miningQueueId = MiningQueue.id \
         WHERE Robot.userId = ? \
           AND MiningQueue.claimed = TRUE \
           AND (SELECT COUNT(*) \
                FROM MiningQueue RankedQueue \
                WHERE RankedQueue.robotId = MiningQueue.robotId \
                  AND RankedQueue.claimed = TRUE \
                  AND (RankedQueue.miningEndTime > MiningQueue.miningEndTime \
                       OR (RankedQueue.miningEndTime = MiningQueue.miningEndTime \
                           AND RankedQueue.id <= MiningQueue.id))) <= ? \
         GROUP BY MiningQueue.robotId, MiningQueue.id, MiningArea.areaName, \
                  MiningQueue.rallyResultId, MiningQueue.score, MiningQueue.creationTime, \
                  MiningQueue.miningEndTime \
         ORDER BY MiningQueue.robotId, MiningQueue.miningEndTime DESC, MiningQueue.id",
    )
    .bind(user_id)
    .bind(maximum_results)
    .fetch_all(pool)
    .await
    .map(|rows| {
        rows.into_iter()
            .map(
                |(
                    robot_id,
                    mining_queue_id,
                    mining_area_name,
                    rally_result_id,
                    score,
                    total_ore_mined,
                    total_tax,
                    total_reward,
                    creation_time_millis,
                    mining_end_time_millis,
                )| MiningResultStateRecord {
                    robot_id,
                    mining_queue_id,
                    mining_area_name,
                    rally_result_id,
                    score,
                    total_ore_mined,
                    total_tax,
                    total_reward,
                    creation_time_millis,
                    mining_end_time_millis,
                },
            )
            .collect()
    })
}

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
