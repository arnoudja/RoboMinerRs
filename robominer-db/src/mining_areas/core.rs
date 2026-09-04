use sqlx::MySqlPool;

use crate::{
    MiningAreaOreSupplyRecord, MiningAreaRecord, MiningQueueRecord, MiningRallyQueueRecord,
};

#[derive(sqlx::FromRow)]
struct MiningAreaRow {
    id: i64,
    #[sqlx(rename = "areaName")]
    area_name: String,
    #[sqlx(rename = "orePriceId")]
    ore_price_id: i64,
    #[sqlx(rename = "sizeX")]
    size_x: i32,
    #[sqlx(rename = "sizeY")]
    size_y: i32,
    #[sqlx(rename = "maxMoves")]
    max_moves: i32,
    #[sqlx(rename = "miningTime")]
    mining_time: i32,
    #[sqlx(rename = "taxRate")]
    tax_rate: i32,
    #[sqlx(rename = "depotTaxRate")]
    depot_tax_rate: i32,
    #[sqlx(rename = "scoreOreTarget")]
    score_ore_target: i32,
    #[sqlx(rename = "aiRobotId")]
    ai_robot_id: i64,
}

impl From<MiningAreaRow> for MiningAreaRecord {
    fn from(row: MiningAreaRow) -> Self {
        Self {
            id: row.id,
            area_name: row.area_name,
            ore_price_id: row.ore_price_id,
            size_x: row.size_x,
            size_y: row.size_y,
            max_moves: row.max_moves,
            mining_time: row.mining_time,
            tax_rate: row.tax_rate,
            depot_tax_rate: row.depot_tax_rate,
            score_ore_target: row.score_ore_target,
            ai_robot_id: row.ai_robot_id,
        }
    }
}

#[derive(Debug, Clone, sqlx::FromRow)]
pub(crate) struct MiningRallyQueueRow {
    pub(crate) id: i64,
    #[sqlx(rename = "miningAreaId")]
    pub(crate) mining_area_id: i64,
    #[sqlx(rename = "robotId")]
    pub(crate) robot_id: i64,
    #[sqlx(rename = "userId")]
    pub(crate) user_id: i64,
    #[sqlx(rename = "rallyResultId")]
    pub(crate) rally_result_id: Option<i64>,
    #[sqlx(rename = "playerNumber")]
    pub(crate) player_number: Option<i32>,
    pub(crate) score: Option<f64>,
    pub(crate) claimed: bool,
    #[sqlx(rename = "secondsLeft")]
    pub(crate) seconds_left: i32,
}

impl From<MiningRallyQueueRow> for MiningRallyQueueRecord {
    fn from(row: MiningRallyQueueRow) -> Self {
        Self {
            queue: MiningQueueRecord {
                id: row.id,
                mining_area_id: row.mining_area_id,
                robot_id: row.robot_id,
                rally_result_id: row.rally_result_id,
                player_number: row.player_number,
                score: row.score,
                claimed: row.claimed,
            },
            user_id: row.user_id,
            seconds_left: row.seconds_left,
        }
    }
}

#[derive(sqlx::FromRow)]
struct MiningAreaOreSupplyRow {
    id: i64,
    #[sqlx(rename = "miningAreaId")]
    mining_area_id: i64,
    #[sqlx(rename = "oreId")]
    ore_id: i64,
    supply: i32,
    radius: i32,
}

impl From<MiningAreaOreSupplyRow> for MiningAreaOreSupplyRecord {
    fn from(row: MiningAreaOreSupplyRow) -> Self {
        Self {
            id: row.id,
            mining_area_id: row.mining_area_id,
            ore_id: row.ore_id,
            supply: row.supply,
            radius: row.radius,
        }
    }
}

/// Keep the first queue row per user, then cap at four participants.
pub(crate) fn mining_rally_queue_rows(
    rows: Vec<MiningRallyQueueRow>,
) -> Vec<MiningRallyQueueRecord> {
    let mut seen_users = Vec::new();

    rows.into_iter()
        .filter(|row| {
            if seen_users.contains(&row.user_id) {
                false
            } else {
                seen_users.push(row.user_id);
                true
            }
        })
        .take(4)
        .map(MiningRallyQueueRecord::from)
        .collect()
}

pub async fn list_mining_areas(pool: &MySqlPool) -> Result<Vec<MiningAreaRecord>, sqlx::Error> {
    sqlx::query_as::<_, MiningAreaRow>(
        "SELECT id, areaName, orePriceId, sizeX, sizeY, maxMoves, miningTime, taxRate, depotTaxRate, scoreOreTarget, aiRobotId \
         FROM MiningArea \
         ORDER BY id",
    )
    .fetch_all(pool)
    .await
    .map(|rows| rows.into_iter().map(MiningAreaRecord::from).collect())
}

pub async fn get_mining_area(
    pool: &MySqlPool,
    mining_area_id: i64,
) -> Result<Option<MiningAreaRecord>, sqlx::Error> {
    sqlx::query_as::<_, MiningAreaRow>(
        "SELECT id, areaName, orePriceId, sizeX, sizeY, maxMoves, miningTime, taxRate, depotTaxRate, scoreOreTarget, aiRobotId \
         FROM MiningArea \
         WHERE id = ?",
    )
    .bind(mining_area_id)
    .fetch_optional(pool)
    .await
    .map(|row| row.map(MiningAreaRecord::from))
}

pub async fn list_mining_area_ore_supplies(
    pool: &MySqlPool,
    mining_area_id: i64,
) -> Result<Vec<MiningAreaOreSupplyRecord>, sqlx::Error> {
    sqlx::query_as::<_, MiningAreaOreSupplyRow>(
        "SELECT id, miningAreaId, oreId, supply, radius \
         FROM MiningAreaOreSupply \
         WHERE miningAreaId = ? \
         ORDER BY oreId",
    )
    .bind(mining_area_id)
    .fetch_all(pool)
    .await
    .map(|rows| {
        rows.into_iter()
            .map(MiningAreaOreSupplyRecord::from)
            .collect()
    })
}

pub async fn list_next_mining_rally_queue_for_area(
    pool: &MySqlPool,
    mining_area_id: i64,
) -> Result<Vec<MiningRallyQueueRecord>, sqlx::Error> {
    sqlx::query_as::<_, MiningRallyQueueRow>(
        "SELECT MiningQueue.id, MiningQueue.miningAreaId, MiningQueue.robotId, \
                Robot.userId, \
                MiningQueue.rallyResultId, MiningQueue.playerNumber, MiningQueue.score, \
                MiningQueue.claimed, \
                TIMESTAMPDIFF(SECOND, NOW(), \
                    TIMESTAMPADD(SECOND, MiningArea.miningTime, \
                        IF(Robot.rechargeEndTime < MiningQueue.creationTime, \
                           MiningQueue.creationTime, Robot.rechargeEndTime))) AS secondsLeft \
         FROM MiningQueue, Robot, MiningArea \
         WHERE MiningQueue.miningAreaId = ? \
           AND MiningQueue.miningEndTime IS NULL \
           AND (MiningQueue.processingLeaseUntil IS NULL \
                OR MiningQueue.processingLeaseUntil < NOW()) \
           AND Robot.id = MiningQueue.robotId \
           AND (Robot.rechargeEndTime IS NULL OR Robot.rechargeEndTime <= NOW()) \
           AND (Robot.miningEndTime IS NULL OR Robot.miningEndTime <= NOW()) \
           AND MiningArea.id = MiningQueue.miningAreaId \
           AND NOT EXISTS ( \
               SELECT prev.id \
               FROM MiningQueue prev \
               WHERE prev.id < MiningQueue.id \
                 AND prev.robotId = MiningQueue.robotId \
                 AND prev.miningEndTime IS NULL \
           ) \
         ORDER BY secondsLeft, MiningQueue.id",
    )
    .bind(mining_area_id)
    .fetch_all(pool)
    .await
    .map(mining_rally_queue_rows)
}
