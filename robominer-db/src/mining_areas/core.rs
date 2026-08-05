use sqlx::MySqlPool;

use crate::mappers::{
    MiningAreaRow, MiningRallyQueueRow, mining_area_record, mining_area_rows,
    mining_rally_queue_rows,
};
use crate::{MiningAreaOreSupplyRecord, MiningAreaRecord, MiningRallyQueueRecord};

pub async fn list_mining_areas(pool: &MySqlPool) -> Result<Vec<MiningAreaRecord>, sqlx::Error> {
    sqlx::query_as::<_, MiningAreaRow>(
        "SELECT id, areaName, orePriceId, sizeX, sizeY, maxMoves, miningTime, taxRate, scoreOreTarget, aiRobotId \
         FROM MiningArea \
         ORDER BY id",
    )
    .fetch_all(pool)
    .await
    .map(mining_area_rows)
}

pub async fn get_mining_area(
    pool: &MySqlPool,
    mining_area_id: i64,
) -> Result<Option<MiningAreaRecord>, sqlx::Error> {
    sqlx::query_as::<_, MiningAreaRow>(
        "SELECT id, areaName, orePriceId, sizeX, sizeY, maxMoves, miningTime, taxRate, scoreOreTarget, aiRobotId \
         FROM MiningArea \
         WHERE id = ?",
    )
    .bind(mining_area_id)
    .fetch_optional(pool)
    .await
    .map(|row| row.map(mining_area_record))
}

pub async fn list_mining_area_ore_supplies(
    pool: &MySqlPool,
    mining_area_id: i64,
) -> Result<Vec<MiningAreaOreSupplyRecord>, sqlx::Error> {
    sqlx::query_as::<_, (i64, i64, i64, i32, i32)>(
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
            .map(
                |(id, mining_area_id, ore_id, supply, radius)| MiningAreaOreSupplyRecord {
                    id,
                    mining_area_id,
                    ore_id,
                    supply,
                    radius,
                },
            )
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
