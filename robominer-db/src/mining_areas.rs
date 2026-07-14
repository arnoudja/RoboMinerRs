use sqlx::MySqlPool;

use crate::mappers::{
    MiningAreaRow, MiningRallyQueueRow, mining_area_record, mining_area_rows,
    mining_rally_queue_rows,
};
use crate::{
    MiningAreaOreSupplyRecord, MiningAreaOverviewAreaRecord, MiningAreaOverviewOreRecord,
    MiningAreaOverviewPercentageRecord, MiningAreaRecord, MiningRallyQueueRecord,
};

pub async fn list_mining_areas(pool: &MySqlPool) -> Result<Vec<MiningAreaRecord>, sqlx::Error> {
    sqlx::query_as::<_, MiningAreaRow>(
        "SELECT id, areaName, orePriceId, sizeX, sizeY, maxMoves, miningTime, taxRate, aiRobotId \
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
        "SELECT id, areaName, orePriceId, sizeX, sizeY, maxMoves, miningTime, taxRate, aiRobotId \
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
pub async fn list_mining_area_overview_ores(
    pool: &MySqlPool,
) -> Result<Vec<MiningAreaOverviewOreRecord>, sqlx::Error> {
    sqlx::query_as::<_, (i64, String)>(
        "SELECT id, oreName \
         FROM Ore \
         ORDER BY id",
    )
    .fetch_all(pool)
    .await
    .map(|rows| {
        rows.into_iter()
            .map(|(ore_id, ore_name)| MiningAreaOverviewOreRecord { ore_id, ore_name })
            .collect()
    })
}

pub async fn list_mining_area_overview_ores_for_user(
    pool: &MySqlPool,
    user_id: i64,
) -> Result<Vec<MiningAreaOverviewOreRecord>, sqlx::Error> {
    sqlx::query_as::<_, (i64, String)>(
        "SELECT DISTINCT Ore.id, Ore.oreName \
         FROM Ore \
         WHERE EXISTS ( \
             SELECT 1 \
             FROM UserOreAsset \
             WHERE UserOreAsset.userId = ? \
               AND UserOreAsset.oreId = Ore.id \
         ) \
            OR EXISTS ( \
             SELECT 1 \
             FROM MiningAreaOreSupply \
             INNER JOIN UserMiningArea \
               ON UserMiningArea.miningAreaId = MiningAreaOreSupply.miningAreaId \
             WHERE UserMiningArea.userId = ? \
               AND MiningAreaOreSupply.oreId = Ore.id \
         ) \
         ORDER BY Ore.id",
    )
    .bind(user_id)
    .bind(user_id)
    .fetch_all(pool)
    .await
    .map(|rows| {
        rows.into_iter()
            .map(|(ore_id, ore_name)| MiningAreaOverviewOreRecord { ore_id, ore_name })
            .collect()
    })
}

pub async fn list_mining_area_overview_areas(
    pool: &MySqlPool,
) -> Result<Vec<MiningAreaOverviewAreaRecord>, sqlx::Error> {
    sqlx::query_as::<_, (i64, String, f64)>(
        "SELECT MiningArea.id, MiningArea.areaName, \
                CAST(COALESCE(SUM(CASE WHEN MiningAreaLifetimeResult.totalContainerSize > 0 \
                                        THEN MiningAreaLifetimeResult.totalAmount * 100.0 / MiningAreaLifetimeResult.totalContainerSize \
                                        ELSE 0.0 END), 0.0) AS DOUBLE) \
         FROM MiningArea \
         INNER JOIN MiningAreaLifetimeResult \
           ON MiningAreaLifetimeResult.miningAreaId = MiningArea.id \
         GROUP BY MiningArea.id, MiningArea.areaName \
         ORDER BY MiningArea.id",
    )
    .fetch_all(pool)
    .await
    .map(map_mining_area_overview_area_rows)
}

pub async fn list_mining_area_overview_areas_for_user(
    pool: &MySqlPool,
    user_id: i64,
) -> Result<Vec<MiningAreaOverviewAreaRecord>, sqlx::Error> {
    sqlx::query_as::<_, (i64, String, f64)>(
        "SELECT MiningArea.id, MiningArea.areaName, \
                CAST(COALESCE(SUM(CASE WHEN MiningAreaLifetimeResult.totalContainerSize > 0 \
                                        THEN MiningAreaLifetimeResult.totalAmount * 100.0 / MiningAreaLifetimeResult.totalContainerSize \
                                        ELSE 0.0 END), 0.0) AS DOUBLE) \
         FROM MiningArea \
         INNER JOIN UserMiningArea \
           ON UserMiningArea.miningAreaId = MiningArea.id \
         INNER JOIN MiningAreaLifetimeResult \
           ON MiningAreaLifetimeResult.miningAreaId = MiningArea.id \
         WHERE UserMiningArea.userId = ? \
         GROUP BY MiningArea.id, MiningArea.areaName \
         ORDER BY MiningArea.id",
    )
    .bind(user_id)
    .fetch_all(pool)
    .await
    .map(map_mining_area_overview_area_rows)
}

fn map_mining_area_overview_area_rows(
    rows: Vec<(i64, String, f64)>,
) -> Vec<MiningAreaOverviewAreaRecord> {
    rows.into_iter()
        .map(
            |(mining_area_id, area_name, total_percentage)| MiningAreaOverviewAreaRecord {
                mining_area_id,
                area_name,
                total_percentage,
            },
        )
        .collect()
}

pub async fn list_mining_area_overview_percentages(
    pool: &MySqlPool,
) -> Result<Vec<MiningAreaOverviewPercentageRecord>, sqlx::Error> {
    sqlx::query_as::<_, (i64, i64, f64)>(
        "SELECT miningAreaId, oreId, \
                CAST(CASE WHEN totalContainerSize > 0 \
                          THEN totalAmount * 100.0 / totalContainerSize \
                          ELSE 0.0 END AS DOUBLE) \
         FROM MiningAreaLifetimeResult \
         ORDER BY miningAreaId, oreId",
    )
    .fetch_all(pool)
    .await
    .map(map_mining_area_overview_percentage_rows)
}

pub async fn list_mining_area_overview_percentages_for_user(
    pool: &MySqlPool,
    user_id: i64,
) -> Result<Vec<MiningAreaOverviewPercentageRecord>, sqlx::Error> {
    sqlx::query_as::<_, (i64, i64, f64)>(
        "SELECT MiningAreaLifetimeResult.miningAreaId, MiningAreaLifetimeResult.oreId, \
                CAST(CASE WHEN MiningAreaLifetimeResult.totalContainerSize > 0 \
                          THEN MiningAreaLifetimeResult.totalAmount * 100.0 / MiningAreaLifetimeResult.totalContainerSize \
                          ELSE 0.0 END AS DOUBLE) \
         FROM MiningAreaLifetimeResult \
         INNER JOIN UserMiningArea \
           ON UserMiningArea.miningAreaId = MiningAreaLifetimeResult.miningAreaId \
         WHERE UserMiningArea.userId = ? \
         ORDER BY MiningAreaLifetimeResult.miningAreaId, MiningAreaLifetimeResult.oreId",
    )
    .bind(user_id)
    .fetch_all(pool)
    .await
    .map(map_mining_area_overview_percentage_rows)
}

fn map_mining_area_overview_percentage_rows(
    rows: Vec<(i64, i64, f64)>,
) -> Vec<MiningAreaOverviewPercentageRecord> {
    rows.into_iter()
        .map(
            |(mining_area_id, ore_id, percentage)| MiningAreaOverviewPercentageRecord {
                mining_area_id,
                ore_id,
                percentage,
            },
        )
        .collect()
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
