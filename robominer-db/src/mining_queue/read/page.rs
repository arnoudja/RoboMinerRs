use sqlx::MySqlPool;

use crate::{
    MiningQueuePageAreaCostRecord, MiningQueuePageAreaRecord, MiningQueuePageAreaSupplyRecord,
    MiningQueuePageAreaYieldRecord, MiningQueuePageItemRecord, MiningQueuePageRobotRecord,
};

pub async fn list_mining_queue_page_robots(
    pool: &MySqlPool,
    user_id: i64,
) -> Result<Vec<MiningQueuePageRobotRecord>, sqlx::Error> {
    sqlx::query_as::<_, (i64, String, i32)>(
        "SELECT id, robotName, rechargeTime \
         FROM Robot \
         WHERE userId = ? \
         ORDER BY id",
    )
    .bind(user_id)
    .fetch_all(pool)
    .await
    .map(|rows| {
        rows.into_iter()
            .map(
                |(robot_id, robot_name, recharge_time)| MiningQueuePageRobotRecord {
                    robot_id,
                    robot_name,
                    recharge_time,
                },
            )
            .collect()
    })
}

pub async fn list_mining_queue_page_areas(
    pool: &MySqlPool,
    user_id: i64,
) -> Result<Vec<MiningQueuePageAreaRecord>, sqlx::Error> {
    sqlx::query_as::<_, (i64, String, i32, i32, i32, i32, i32, i32, i32)>(
        "SELECT MiningArea.id, MiningArea.areaName, MiningArea.taxRate, MiningArea.depotTaxRate, \
                MiningArea.miningTime, MiningArea.maxMoves, MiningArea.sizeX, MiningArea.sizeY, \
                MiningArea.scoreOreTarget \
         FROM MiningArea \
         INNER JOIN UserMiningArea ON UserMiningArea.miningAreaId = MiningArea.id \
         WHERE UserMiningArea.userId = ? \
         ORDER BY MiningArea.id",
    )
    .bind(user_id)
    .fetch_all(pool)
    .await
    .map(|rows| {
        rows.into_iter()
            .map(
                |(
                    mining_area_id,
                    area_name,
                    tax_rate,
                    depot_tax_rate,
                    mining_time,
                    max_moves,
                    size_x,
                    size_y,
                    score_ore_target,
                )| {
                    MiningQueuePageAreaRecord {
                        mining_area_id,
                        area_name,
                        tax_rate,
                        depot_tax_rate,
                        mining_time,
                        max_moves,
                        size_x,
                        size_y,
                        score_ore_target,
                    }
                },
            )
            .collect()
    })
}

pub async fn list_mining_queue_page_area_costs(
    pool: &MySqlPool,
    user_id: i64,
) -> Result<Vec<MiningQueuePageAreaCostRecord>, sqlx::Error> {
    sqlx::query_as::<_, (i64, i64, String, i32)>(
        "SELECT MiningArea.id, OrePriceAmount.oreId, Ore.oreName, OrePriceAmount.amount \
         FROM MiningArea \
         INNER JOIN UserMiningArea ON UserMiningArea.miningAreaId = MiningArea.id \
         INNER JOIN OrePriceAmount ON OrePriceAmount.orePriceId = MiningArea.orePriceId \
         INNER JOIN Ore ON Ore.id = OrePriceAmount.oreId \
         WHERE UserMiningArea.userId = ? \
         ORDER BY MiningArea.id, OrePriceAmount.oreId DESC",
    )
    .bind(user_id)
    .fetch_all(pool)
    .await
    .map(|rows| {
        rows.into_iter()
            .map(
                |(mining_area_id, ore_id, ore_name, amount)| MiningQueuePageAreaCostRecord {
                    mining_area_id,
                    ore_id,
                    ore_name,
                    amount,
                },
            )
            .collect()
    })
}

pub async fn list_mining_queue_page_area_supplies(
    pool: &MySqlPool,
    user_id: i64,
) -> Result<Vec<MiningQueuePageAreaSupplyRecord>, sqlx::Error> {
    sqlx::query_as::<_, (i64, i64, String, i32, i32)>(
        "SELECT MiningAreaOreSupply.miningAreaId, MiningAreaOreSupply.oreId, Ore.oreName, \
                MiningAreaOreSupply.supply, MiningAreaOreSupply.radius \
         FROM MiningAreaOreSupply \
         INNER JOIN UserMiningArea ON UserMiningArea.miningAreaId = MiningAreaOreSupply.miningAreaId \
         INNER JOIN Ore ON Ore.id = MiningAreaOreSupply.oreId \
         WHERE UserMiningArea.userId = ? \
         ORDER BY MiningAreaOreSupply.miningAreaId, MiningAreaOreSupply.oreId DESC",
    )
    .bind(user_id)
    .fetch_all(pool)
    .await
    .map(|rows| {
        rows.into_iter()
            .map(
                |(mining_area_id, ore_id, ore_name, supply, radius)| {
                    MiningQueuePageAreaSupplyRecord {
                        mining_area_id,
                        ore_id,
                        ore_name,
                        supply,
                        radius,
                    }
                },
            )
            .collect()
    })
}

pub async fn list_mining_queue_page_area_yields(
    pool: &MySqlPool,
    user_id: i64,
) -> Result<Vec<MiningQueuePageAreaYieldRecord>, sqlx::Error> {
    sqlx::query_as::<_, (i64, i64, String, f64)>(
        "SELECT MiningAreaLifetimeResult.miningAreaId, MiningAreaLifetimeResult.oreId, \
                Ore.oreName, \
                CAST(CASE WHEN MiningAreaLifetimeResult.totalContainerSize > 0 \
                          THEN MiningAreaLifetimeResult.totalAmount * 100.0 / MiningAreaLifetimeResult.totalContainerSize \
                          ELSE 0.0 END AS DOUBLE) \
         FROM MiningAreaLifetimeResult \
         INNER JOIN UserMiningArea ON UserMiningArea.miningAreaId = MiningAreaLifetimeResult.miningAreaId \
         INNER JOIN Ore ON Ore.id = MiningAreaLifetimeResult.oreId \
         WHERE UserMiningArea.userId = ? \
         ORDER BY MiningAreaLifetimeResult.miningAreaId, MiningAreaLifetimeResult.oreId DESC",
    )
    .bind(user_id)
    .fetch_all(pool)
    .await
    .map(|rows| {
        rows.into_iter()
            .map(
                |(mining_area_id, ore_id, ore_name, percentage)| {
                    MiningQueuePageAreaYieldRecord {
                        mining_area_id,
                        ore_id,
                        ore_name,
                        percentage,
                    }
                },
            )
            .collect()
    })
}

pub async fn list_mining_queue_page_items(
    pool: &MySqlPool,
    user_id: i64,
) -> Result<Vec<MiningQueuePageItemRecord>, sqlx::Error> {
    sqlx::query_as::<_, (i64, i64, i64, String, Option<i64>)>(
        "SELECT MiningQueue.id, MiningQueue.robotId, MiningQueue.miningAreaId, MiningArea.areaName, \
                MiningQueue.rallyResultId \
         FROM MiningQueue \
         INNER JOIN Robot ON Robot.id = MiningQueue.robotId \
         INNER JOIN MiningArea ON MiningArea.id = MiningQueue.miningAreaId \
         WHERE Robot.userId = ? \
           AND (MiningQueue.miningEndTime IS NULL OR MiningQueue.miningEndTime > NOW()) \
         ORDER BY MiningQueue.robotId, MiningQueue.id",
    )
    .bind(user_id)
    .fetch_all(pool)
    .await
    .map(|rows| {
        rows.into_iter()
            .map(
                |(mining_queue_id, robot_id, mining_area_id, area_name, rally_result_id)| {
                    MiningQueuePageItemRecord {
                        mining_queue_id,
                        robot_id,
                        mining_area_id,
                        area_name,
                        rally_result_id,
                    }
                },
            )
            .collect()
    })
}
