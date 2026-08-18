use sqlx::MySqlPool;

use crate::MiningResultStateRecord;

type MiningResultStateRow = (i64, i64, String, Option<i64>, f64, i32, i32, i32, i64, i64);

const MINING_RESULT_STATE_COLUMNS: &str = "MiningQueue.robotId, MiningQueue.id, MiningArea.areaName, \
     MiningQueue.rallyResultId, COALESCE(MiningQueue.score, 0.0), \
     CAST(COALESCE(SUM(MiningOreResult.amount), 0) AS SIGNED), \
     CAST(COALESCE(SUM(COALESCE(MiningOreResult.tax, 0)), 0) AS SIGNED), \
     CAST(COALESCE(SUM(MiningOreResult.amount - COALESCE(MiningOreResult.tax, 0)), 0) AS SIGNED), \
     CAST(UNIX_TIMESTAMP(MiningQueue.creationTime) * 1000 AS SIGNED), \
     CAST(UNIX_TIMESTAMP(MiningQueue.miningEndTime) * 1000 AS SIGNED)";

fn mining_result_state_rows(rows: Vec<MiningResultStateRow>) -> Vec<MiningResultStateRecord> {
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
}

pub async fn list_mining_result_states_for_user(
    pool: &MySqlPool,
    user_id: i64,
    maximum_results: i64,
) -> Result<Vec<MiningResultStateRecord>, sqlx::Error> {
    sqlx::query_as::<_, MiningResultStateRow>(&format!(
        "SELECT {MINING_RESULT_STATE_COLUMNS} \
         FROM MiningQueue \
         INNER JOIN Robot ON Robot.id = MiningQueue.robotId \
         INNER JOIN MiningArea ON MiningArea.id = MiningQueue.miningAreaId \
         LEFT OUTER JOIN MiningOreResult ON MiningOreResult.miningQueueId = MiningQueue.id \
         WHERE Robot.userId = ? \
           AND MiningQueue.claimed = TRUE \
         GROUP BY MiningQueue.robotId, MiningQueue.id, MiningArea.areaName, \
                  MiningQueue.rallyResultId, MiningQueue.score, MiningQueue.creationTime, \
                  MiningQueue.miningEndTime \
         ORDER BY MiningQueue.miningEndTime DESC, MiningQueue.id DESC \
         LIMIT ?"
    ))
    .bind(user_id)
    .bind(maximum_results)
    .fetch_all(pool)
    .await
    .map(mining_result_state_rows)
}

pub async fn list_mining_result_states_for_robot(
    pool: &MySqlPool,
    robot_id: i64,
    maximum_results: i64,
) -> Result<Vec<MiningResultStateRecord>, sqlx::Error> {
    sqlx::query_as::<_, MiningResultStateRow>(&format!(
        "SELECT {MINING_RESULT_STATE_COLUMNS} \
         FROM MiningQueue \
         INNER JOIN MiningArea ON MiningArea.id = MiningQueue.miningAreaId \
         LEFT OUTER JOIN MiningOreResult ON MiningOreResult.miningQueueId = MiningQueue.id \
         WHERE MiningQueue.robotId = ? \
           AND MiningQueue.claimed = TRUE \
         GROUP BY MiningQueue.robotId, MiningQueue.id, MiningArea.areaName, \
                  MiningQueue.rallyResultId, MiningQueue.score, MiningQueue.creationTime, \
                  MiningQueue.miningEndTime \
         ORDER BY MiningQueue.miningEndTime DESC, MiningQueue.id DESC \
         LIMIT ?"
    ))
    .bind(robot_id)
    .bind(maximum_results)
    .fetch_all(pool)
    .await
    .map(mining_result_state_rows)
}
