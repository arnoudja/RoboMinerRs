use sqlx::MySqlPool;

use crate::{
    MiningQueuePageAreaCostRecord, MiningQueuePageAreaRecord, MiningQueuePageAreaSupplyRecord,
    MiningQueuePageAreaYieldRecord, MiningQueuePageItemRecord, MiningQueuePageRobotRecord,
};

#[derive(sqlx::FromRow)]
struct MiningQueuePageRobotRow {
    #[sqlx(rename = "id")]
    robot_id: i64,
    #[sqlx(rename = "robotName")]
    robot_name: String,
    #[sqlx(rename = "rechargeTime")]
    recharge_time: i32,
}

impl From<MiningQueuePageRobotRow> for MiningQueuePageRobotRecord {
    fn from(row: MiningQueuePageRobotRow) -> Self {
        Self {
            robot_id: row.robot_id,
            robot_name: row.robot_name,
            recharge_time: row.recharge_time,
        }
    }
}

#[derive(sqlx::FromRow)]
struct MiningQueuePageAreaRow {
    #[sqlx(rename = "id")]
    mining_area_id: i64,
    #[sqlx(rename = "areaName")]
    area_name: String,
    #[sqlx(rename = "taxRate")]
    tax_rate: i32,
    #[sqlx(rename = "depotTaxRate")]
    depot_tax_rate: i32,
    #[sqlx(rename = "miningTime")]
    mining_time: i32,
    #[sqlx(rename = "maxMoves")]
    max_moves: i32,
    #[sqlx(rename = "sizeX")]
    size_x: i32,
    #[sqlx(rename = "sizeY")]
    size_y: i32,
    #[sqlx(rename = "scoreOreTarget")]
    score_ore_target: i32,
}

impl From<MiningQueuePageAreaRow> for MiningQueuePageAreaRecord {
    fn from(row: MiningQueuePageAreaRow) -> Self {
        Self {
            mining_area_id: row.mining_area_id,
            area_name: row.area_name,
            tax_rate: row.tax_rate,
            depot_tax_rate: row.depot_tax_rate,
            mining_time: row.mining_time,
            max_moves: row.max_moves,
            size_x: row.size_x,
            size_y: row.size_y,
            score_ore_target: row.score_ore_target,
        }
    }
}

#[derive(sqlx::FromRow)]
struct MiningQueuePageAreaCostRow {
    #[sqlx(rename = "miningAreaId")]
    mining_area_id: i64,
    #[sqlx(rename = "oreId")]
    ore_id: i64,
    #[sqlx(rename = "oreName")]
    ore_name: String,
    amount: i32,
}

impl From<MiningQueuePageAreaCostRow> for MiningQueuePageAreaCostRecord {
    fn from(row: MiningQueuePageAreaCostRow) -> Self {
        Self {
            mining_area_id: row.mining_area_id,
            ore_id: row.ore_id,
            ore_name: row.ore_name,
            amount: row.amount,
        }
    }
}

#[derive(sqlx::FromRow)]
struct MiningQueuePageAreaSupplyRow {
    #[sqlx(rename = "miningAreaId")]
    mining_area_id: i64,
    #[sqlx(rename = "oreId")]
    ore_id: i64,
    #[sqlx(rename = "oreName")]
    ore_name: String,
    supply: i32,
    radius: i32,
}

impl From<MiningQueuePageAreaSupplyRow> for MiningQueuePageAreaSupplyRecord {
    fn from(row: MiningQueuePageAreaSupplyRow) -> Self {
        Self {
            mining_area_id: row.mining_area_id,
            ore_id: row.ore_id,
            ore_name: row.ore_name,
            supply: row.supply,
            radius: row.radius,
        }
    }
}

#[derive(sqlx::FromRow)]
struct MiningQueuePageAreaYieldRow {
    #[sqlx(rename = "miningAreaId")]
    mining_area_id: i64,
    #[sqlx(rename = "oreId")]
    ore_id: i64,
    #[sqlx(rename = "oreName")]
    ore_name: String,
    percentage: f64,
}

impl From<MiningQueuePageAreaYieldRow> for MiningQueuePageAreaYieldRecord {
    fn from(row: MiningQueuePageAreaYieldRow) -> Self {
        Self {
            mining_area_id: row.mining_area_id,
            ore_id: row.ore_id,
            ore_name: row.ore_name,
            percentage: row.percentage,
        }
    }
}

#[derive(sqlx::FromRow)]
struct MiningQueuePageItemRow {
    #[sqlx(rename = "miningQueueId")]
    mining_queue_id: i64,
    #[sqlx(rename = "robotId")]
    robot_id: i64,
    #[sqlx(rename = "miningAreaId")]
    mining_area_id: i64,
    #[sqlx(rename = "areaName")]
    area_name: String,
    #[sqlx(rename = "rallyResultId")]
    rally_result_id: Option<i64>,
}

impl From<MiningQueuePageItemRow> for MiningQueuePageItemRecord {
    fn from(row: MiningQueuePageItemRow) -> Self {
        Self {
            mining_queue_id: row.mining_queue_id,
            robot_id: row.robot_id,
            mining_area_id: row.mining_area_id,
            area_name: row.area_name,
            rally_result_id: row.rally_result_id,
        }
    }
}

pub async fn list_mining_queue_page_robots(
    pool: &MySqlPool,
    user_id: i64,
) -> Result<Vec<MiningQueuePageRobotRecord>, sqlx::Error> {
    sqlx::query_as::<_, MiningQueuePageRobotRow>(
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
            .map(MiningQueuePageRobotRecord::from)
            .collect()
    })
}

pub async fn list_mining_queue_page_areas(
    pool: &MySqlPool,
    user_id: i64,
) -> Result<Vec<MiningQueuePageAreaRecord>, sqlx::Error> {
    sqlx::query_as::<_, MiningQueuePageAreaRow>(
        "SELECT MiningArea.id, MiningArea.areaName, MiningArea.taxRate, MiningArea.depotTaxRate, \
                MiningArea.miningTime, MiningArea.maxMoves, MiningArea.sizeX, MiningArea.sizeY, \
                MiningArea.scoreOreTarget \
         FROM MiningArea \
         INNER JOIN UserMiningArea ON UserMiningArea.miningAreaId = MiningArea.id \
         WHERE UserMiningArea.userId = ? \
         ORDER BY MiningArea.id DESC",
    )
    .bind(user_id)
    .fetch_all(pool)
    .await
    .map(|rows| {
        rows.into_iter()
            .map(MiningQueuePageAreaRecord::from)
            .collect()
    })
}

pub async fn list_mining_queue_page_area_costs(
    pool: &MySqlPool,
    user_id: i64,
) -> Result<Vec<MiningQueuePageAreaCostRecord>, sqlx::Error> {
    sqlx::query_as::<_, MiningQueuePageAreaCostRow>(
        "SELECT MiningArea.id AS miningAreaId, OrePriceAmount.oreId, Ore.oreName, OrePriceAmount.amount \
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
            .map(MiningQueuePageAreaCostRecord::from)
            .collect()
    })
}

pub async fn list_mining_queue_page_area_supplies(
    pool: &MySqlPool,
    user_id: i64,
) -> Result<Vec<MiningQueuePageAreaSupplyRecord>, sqlx::Error> {
    sqlx::query_as::<_, MiningQueuePageAreaSupplyRow>(
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
            .map(MiningQueuePageAreaSupplyRecord::from)
            .collect()
    })
}

pub async fn list_mining_queue_page_area_yields(
    pool: &MySqlPool,
    user_id: i64,
) -> Result<Vec<MiningQueuePageAreaYieldRecord>, sqlx::Error> {
    sqlx::query_as::<_, MiningQueuePageAreaYieldRow>(
        "SELECT MiningAreaAverageResult.miningAreaId, MiningAreaAverageResult.oreId, \
                Ore.oreName, \
                CAST(CASE WHEN MiningAreaAverageResult.totalContainerSize > 0 \
                          THEN MiningAreaAverageResult.totalAmount * 100.0 / MiningAreaAverageResult.totalContainerSize \
                          ELSE 0.0 END AS DOUBLE) AS percentage \
         FROM MiningAreaAverageResult \
         INNER JOIN UserMiningArea ON UserMiningArea.miningAreaId = MiningAreaAverageResult.miningAreaId \
         INNER JOIN Ore ON Ore.id = MiningAreaAverageResult.oreId \
         WHERE UserMiningArea.userId = ? \
         ORDER BY MiningAreaAverageResult.miningAreaId, MiningAreaAverageResult.oreId DESC",
    )
    .bind(user_id)
    .fetch_all(pool)
    .await
    .map(|rows| {
        rows.into_iter()
            .map(MiningQueuePageAreaYieldRecord::from)
            .collect()
    })
}

pub async fn list_mining_queue_page_items(
    pool: &MySqlPool,
    user_id: i64,
) -> Result<Vec<MiningQueuePageItemRecord>, sqlx::Error> {
    sqlx::query_as::<_, MiningQueuePageItemRow>(
        "SELECT MiningQueue.id AS miningQueueId, MiningQueue.robotId, MiningQueue.miningAreaId, \
                MiningArea.areaName AS areaName, MiningQueue.rallyResultId \
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
            .map(MiningQueuePageItemRecord::from)
            .collect()
    })
}
