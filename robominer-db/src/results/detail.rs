use sqlx::MySqlPool;

use crate::assert_sql_safe;

use crate::{MiningResultActionStateRecord, MiningResultAreaOreRecord, MiningResultOreStateRecord};

#[derive(sqlx::FromRow)]
struct MiningResultOreStateRow {
    #[sqlx(rename = "id")]
    mining_queue_id: i64,
    #[sqlx(rename = "oreId")]
    ore_id: i64,
    #[sqlx(rename = "oreName")]
    ore_name: String,
    amount: i32,
    tax: i32,
    reward: i32,
}

impl From<MiningResultOreStateRow> for MiningResultOreStateRecord {
    fn from(row: MiningResultOreStateRow) -> Self {
        Self {
            mining_queue_id: row.mining_queue_id,
            ore_id: row.ore_id,
            ore_name: row.ore_name,
            amount: row.amount,
            tax: row.tax,
            reward: row.reward,
        }
    }
}

#[derive(sqlx::FromRow)]
struct MiningResultActionStateRow {
    #[sqlx(rename = "id")]
    mining_queue_id: i64,
    #[sqlx(rename = "actionType")]
    action_type: i32,
    amount: i32,
}

impl From<MiningResultActionStateRow> for MiningResultActionStateRecord {
    fn from(row: MiningResultActionStateRow) -> Self {
        Self {
            mining_queue_id: row.mining_queue_id,
            action_type: row.action_type,
            amount: row.amount,
        }
    }
}

#[derive(sqlx::FromRow)]
struct MiningResultAreaOreRow {
    #[sqlx(rename = "miningAreaId")]
    mining_area_id: i64,
    #[sqlx(rename = "oreId")]
    ore_id: i64,
    #[sqlx(rename = "oreName")]
    ore_name: String,
}

impl From<MiningResultAreaOreRow> for MiningResultAreaOreRecord {
    fn from(row: MiningResultAreaOreRow) -> Self {
        Self {
            mining_area_id: row.mining_area_id,
            ore_id: row.ore_id,
            ore_name: row.ore_name,
        }
    }
}

pub async fn list_mining_result_ore_states_for_user(
    pool: &MySqlPool,
    user_id: i64,
    maximum_results: i64,
) -> Result<Vec<MiningResultOreStateRecord>, sqlx::Error> {
    sqlx::query_as::<_, MiningResultOreStateRow>(assert_sql_safe(format!(
        "SELECT MiningQueue.id AS id, MiningOreResult.oreId AS oreId, Ore.oreName AS oreName, \
                MiningOreResult.amount AS amount, COALESCE(MiningOreResult.tax, 0) AS tax, \
                MiningOreResult.amount - COALESCE(MiningOreResult.tax, 0) AS reward \
         FROM MiningQueue \
         INNER JOIN MiningOreResult ON MiningOreResult.miningQueueId = MiningQueue.id \
         INNER JOIN Ore ON Ore.id = MiningOreResult.oreId \
         WHERE MiningQueue.id IN ({}) \
         ORDER BY MiningQueue.miningEndTime DESC, MiningQueue.id DESC, Ore.id",
        super::RECENT_CLAIMED_MINING_QUEUE_IDS_FOR_USER
    )))
    .bind(user_id)
    .bind(maximum_results)
    .fetch_all(pool)
    .await
    .map(|rows| {
        rows.into_iter()
            .map(MiningResultOreStateRecord::from)
            .collect()
    })
}

pub async fn list_mining_result_action_states_for_user(
    pool: &MySqlPool,
    user_id: i64,
    maximum_results: i64,
) -> Result<Vec<MiningResultActionStateRecord>, sqlx::Error> {
    sqlx::query_as::<_, MiningResultActionStateRow>(assert_sql_safe(format!(
        "SELECT MiningQueue.id AS id, RobotActionsDone.actionType AS actionType, \
                RobotActionsDone.amount AS amount \
         FROM MiningQueue \
         INNER JOIN RobotActionsDone ON RobotActionsDone.miningQueueId = MiningQueue.id \
         WHERE MiningQueue.id IN ({}) \
         ORDER BY MiningQueue.miningEndTime DESC, MiningQueue.id DESC, \
                  RobotActionsDone.actionType",
        super::RECENT_CLAIMED_MINING_QUEUE_IDS_FOR_USER
    )))
    .bind(user_id)
    .bind(maximum_results)
    .fetch_all(pool)
    .await
    .map(|rows| {
        rows.into_iter()
            .map(MiningResultActionStateRecord::from)
            .collect()
    })
}

pub async fn list_mining_result_area_ores_for_user(
    pool: &MySqlPool,
    user_id: i64,
    maximum_results: i64,
) -> Result<Vec<MiningResultAreaOreRecord>, sqlx::Error> {
    sqlx::query_as::<_, MiningResultAreaOreRow>(assert_sql_safe(format!(
        "SELECT DISTINCT MiningQueue.miningAreaId AS miningAreaId, Ore.id AS oreId, \
                Ore.oreName AS oreName \
         FROM MiningQueue \
         INNER JOIN MiningAreaOreSupply \
           ON MiningAreaOreSupply.miningAreaId = MiningQueue.miningAreaId \
         INNER JOIN Ore ON Ore.id = MiningAreaOreSupply.oreId \
         WHERE MiningQueue.id IN ({}) \
         ORDER BY MiningQueue.miningAreaId, Ore.id DESC",
        super::RECENT_CLAIMED_MINING_QUEUE_IDS_FOR_USER
    )))
    .bind(user_id)
    .bind(maximum_results)
    .fetch_all(pool)
    .await
    .map(|rows| {
        rows.into_iter()
            .map(MiningResultAreaOreRecord::from)
            .collect()
    })
}
