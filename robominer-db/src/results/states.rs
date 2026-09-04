use crate::assert_sql_safe;
use sqlx::MySqlPool;

use crate::MiningResultStateRecord;

#[derive(sqlx::FromRow)]
struct MiningResultStateRow {
    #[sqlx(rename = "robotId")]
    robot_id: i64,
    #[sqlx(rename = "miningQueueId")]
    mining_queue_id: i64,
    #[sqlx(rename = "miningAreaId")]
    mining_area_id: i64,
    #[sqlx(rename = "areaName")]
    mining_area_name: String,
    #[sqlx(rename = "rallyResultId")]
    rally_result_id: Option<i64>,
    score: f64,
    #[sqlx(rename = "scoreOreTarget")]
    score_ore_target: i32,
    #[sqlx(rename = "totalOreMined")]
    total_ore_mined: i32,
    #[sqlx(rename = "totalTax")]
    total_tax: i32,
    #[sqlx(rename = "totalReward")]
    total_reward: i32,
    #[sqlx(rename = "creationTimeMillis")]
    creation_time_millis: i64,
    #[sqlx(rename = "miningEndTimeMillis")]
    mining_end_time_millis: i64,
}

impl From<MiningResultStateRow> for MiningResultStateRecord {
    fn from(row: MiningResultStateRow) -> Self {
        Self {
            robot_id: row.robot_id,
            mining_queue_id: row.mining_queue_id,
            mining_area_id: row.mining_area_id,
            mining_area_name: row.mining_area_name,
            rally_result_id: row.rally_result_id,
            score: row.score,
            score_ore_target: row.score_ore_target,
            total_ore_mined: row.total_ore_mined,
            total_tax: row.total_tax,
            total_reward: row.total_reward,
            creation_time_millis: row.creation_time_millis,
            mining_end_time_millis: row.mining_end_time_millis,
        }
    }
}

const MINING_RESULT_STATE_COLUMNS: &str = "MiningQueue.robotId AS robotId, \
     MiningQueue.id AS miningQueueId, \
     MiningQueue.miningAreaId AS miningAreaId, \
     MiningArea.areaName AS areaName, \
     MiningQueue.rallyResultId AS rallyResultId, \
     COALESCE(MiningQueue.score, 0.0) AS score, \
     MiningArea.scoreOreTarget AS scoreOreTarget, \
     CAST(COALESCE(SUM(MiningOreResult.amount), 0) AS SIGNED) AS totalOreMined, \
     CAST(COALESCE(SUM(COALESCE(MiningOreResult.tax, 0)), 0) AS SIGNED) AS totalTax, \
     CAST(COALESCE(SUM(MiningOreResult.amount - COALESCE(MiningOreResult.tax, 0)), 0) AS SIGNED) AS totalReward, \
     CAST(UNIX_TIMESTAMP(MiningQueue.creationTime) * 1000 AS SIGNED) AS creationTimeMillis, \
     CAST(UNIX_TIMESTAMP(MiningQueue.miningEndTime) * 1000 AS SIGNED) AS miningEndTimeMillis";

const MINING_RESULT_STATE_GROUP_BY: &str = "MiningQueue.robotId, MiningQueue.id, MiningQueue.miningAreaId, \
     MiningArea.areaName, MiningArea.scoreOreTarget, MiningQueue.rallyResultId, MiningQueue.score, \
     MiningQueue.creationTime, MiningQueue.miningEndTime";

pub async fn list_mining_result_states_for_user(
    pool: &MySqlPool,
    user_id: i64,
    maximum_results: i64,
) -> Result<Vec<MiningResultStateRecord>, sqlx::Error> {
    sqlx::query_as::<_, MiningResultStateRow>(assert_sql_safe(format!(
        "SELECT {MINING_RESULT_STATE_COLUMNS} \
         FROM MiningQueue \
         INNER JOIN Robot ON Robot.id = MiningQueue.robotId \
         INNER JOIN MiningArea ON MiningArea.id = MiningQueue.miningAreaId \
         LEFT OUTER JOIN MiningOreResult ON MiningOreResult.miningQueueId = MiningQueue.id \
         WHERE Robot.userId = ? \
           AND MiningQueue.claimed = TRUE \
         GROUP BY {MINING_RESULT_STATE_GROUP_BY} \
         ORDER BY MiningQueue.miningEndTime DESC, MiningQueue.id DESC \
         LIMIT ?"
    )))
    .bind(user_id)
    .bind(maximum_results)
    .fetch_all(pool)
    .await
    .map(|rows| {
        rows.into_iter()
            .map(MiningResultStateRecord::from)
            .collect()
    })
}

pub async fn list_mining_result_states_for_robot(
    pool: &MySqlPool,
    robot_id: i64,
    maximum_results: i64,
) -> Result<Vec<MiningResultStateRecord>, sqlx::Error> {
    sqlx::query_as::<_, MiningResultStateRow>(assert_sql_safe(format!(
        "SELECT {MINING_RESULT_STATE_COLUMNS} \
         FROM MiningQueue \
         INNER JOIN MiningArea ON MiningArea.id = MiningQueue.miningAreaId \
         LEFT OUTER JOIN MiningOreResult ON MiningOreResult.miningQueueId = MiningQueue.id \
         WHERE MiningQueue.robotId = ? \
           AND MiningQueue.claimed = TRUE \
         GROUP BY {MINING_RESULT_STATE_GROUP_BY} \
         ORDER BY MiningQueue.miningEndTime DESC, MiningQueue.id DESC \
         LIMIT ?"
    )))
    .bind(robot_id)
    .bind(maximum_results)
    .fetch_all(pool)
    .await
    .map(|rows| {
        rows.into_iter()
            .map(MiningResultStateRecord::from)
            .collect()
    })
}
