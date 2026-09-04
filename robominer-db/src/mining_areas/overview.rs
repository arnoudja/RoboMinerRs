use sqlx::MySqlPool;

use crate::{
    MiningAreaOverviewAreaRecord, MiningAreaOverviewOreAverageRecord, MiningAreaOverviewOreRecord,
};

#[derive(sqlx::FromRow)]
struct MiningAreaOverviewOreRow {
    #[sqlx(rename = "oreId")]
    ore_id: i64,
    #[sqlx(rename = "oreName")]
    ore_name: String,
}

impl From<MiningAreaOverviewOreRow> for MiningAreaOverviewOreRecord {
    fn from(row: MiningAreaOverviewOreRow) -> Self {
        Self {
            ore_id: row.ore_id,
            ore_name: row.ore_name,
        }
    }
}

#[derive(sqlx::FromRow)]
struct MiningAreaOverviewAreaRow {
    #[sqlx(rename = "miningAreaId")]
    mining_area_id: i64,
    #[sqlx(rename = "areaName")]
    area_name: String,
    #[sqlx(rename = "totalAverageOrePerRun")]
    total_average_ore_per_run: f64,
}

impl From<MiningAreaOverviewAreaRow> for MiningAreaOverviewAreaRecord {
    fn from(row: MiningAreaOverviewAreaRow) -> Self {
        Self {
            mining_area_id: row.mining_area_id,
            area_name: row.area_name,
            total_average_ore_per_run: row.total_average_ore_per_run,
        }
    }
}

#[derive(sqlx::FromRow)]
struct MiningAreaOverviewOreAverageRow {
    #[sqlx(rename = "miningAreaId")]
    mining_area_id: i64,
    #[sqlx(rename = "oreId")]
    ore_id: i64,
    #[sqlx(rename = "averageOrePerRun")]
    average_ore_per_run: f64,
}

impl From<MiningAreaOverviewOreAverageRow> for MiningAreaOverviewOreAverageRecord {
    fn from(row: MiningAreaOverviewOreAverageRow) -> Self {
        Self {
            mining_area_id: row.mining_area_id,
            ore_id: row.ore_id,
            average_ore_per_run: row.average_ore_per_run,
        }
    }
}

pub async fn list_mining_area_overview_ores(
    pool: &MySqlPool,
) -> Result<Vec<MiningAreaOverviewOreRecord>, sqlx::Error> {
    sqlx::query_as::<_, MiningAreaOverviewOreRow>(
        "SELECT id AS oreId, oreName AS oreName \
         FROM Ore \
         ORDER BY id",
    )
    .fetch_all(pool)
    .await
    .map(|rows| {
        rows.into_iter()
            .map(MiningAreaOverviewOreRecord::from)
            .collect()
    })
}

pub async fn list_mining_area_overview_ores_for_user(
    pool: &MySqlPool,
    user_id: i64,
) -> Result<Vec<MiningAreaOverviewOreRecord>, sqlx::Error> {
    sqlx::query_as::<_, MiningAreaOverviewOreRow>(
        "SELECT DISTINCT Ore.id AS oreId, Ore.oreName AS oreName \
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
            .map(MiningAreaOverviewOreRecord::from)
            .collect()
    })
}

pub async fn list_mining_area_overview_areas(
    pool: &MySqlPool,
) -> Result<Vec<MiningAreaOverviewAreaRecord>, sqlx::Error> {
    sqlx::query_as::<_, MiningAreaOverviewAreaRow>(
        "SELECT MiningArea.id AS miningAreaId, MiningArea.areaName AS areaName, \
                CAST(COALESCE(SUM(CASE WHEN MiningAreaLifetimeResult.totalRuns > 0 \
                                        THEN MiningAreaLifetimeResult.totalAmount / MiningAreaLifetimeResult.totalRuns \
                                        ELSE 0.0 END), 0.0) AS DOUBLE) AS totalAverageOrePerRun \
         FROM MiningArea \
         INNER JOIN MiningAreaLifetimeResult \
           ON MiningAreaLifetimeResult.miningAreaId = MiningArea.id \
         GROUP BY MiningArea.id, MiningArea.areaName \
         ORDER BY MiningArea.id",
    )
    .fetch_all(pool)
    .await
    .map(|rows| {
        rows.into_iter()
            .map(MiningAreaOverviewAreaRecord::from)
            .collect()
    })
}

pub async fn list_mining_area_overview_areas_for_user(
    pool: &MySqlPool,
    user_id: i64,
) -> Result<Vec<MiningAreaOverviewAreaRecord>, sqlx::Error> {
    sqlx::query_as::<_, MiningAreaOverviewAreaRow>(
        "SELECT MiningArea.id AS miningAreaId, MiningArea.areaName AS areaName, \
                CAST(COALESCE(SUM(CASE WHEN MiningAreaLifetimeResult.totalRuns > 0 \
                                        THEN MiningAreaLifetimeResult.totalAmount / MiningAreaLifetimeResult.totalRuns \
                                        ELSE 0.0 END), 0.0) AS DOUBLE) AS totalAverageOrePerRun \
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
    .map(|rows| {
        rows.into_iter()
            .map(MiningAreaOverviewAreaRecord::from)
            .collect()
    })
}

pub async fn list_mining_area_overview_ore_averages(
    pool: &MySqlPool,
) -> Result<Vec<MiningAreaOverviewOreAverageRecord>, sqlx::Error> {
    sqlx::query_as::<_, MiningAreaOverviewOreAverageRow>(
        "SELECT miningAreaId AS miningAreaId, oreId AS oreId, \
                CAST(CASE WHEN totalRuns > 0 \
                          THEN totalAmount / totalRuns \
                          ELSE 0.0 END AS DOUBLE) AS averageOrePerRun \
         FROM MiningAreaLifetimeResult \
         ORDER BY miningAreaId, oreId",
    )
    .fetch_all(pool)
    .await
    .map(|rows| {
        rows.into_iter()
            .map(MiningAreaOverviewOreAverageRecord::from)
            .collect()
    })
}

pub async fn list_mining_area_overview_ore_averages_for_user(
    pool: &MySqlPool,
    user_id: i64,
) -> Result<Vec<MiningAreaOverviewOreAverageRecord>, sqlx::Error> {
    sqlx::query_as::<_, MiningAreaOverviewOreAverageRow>(
        "SELECT MiningAreaLifetimeResult.miningAreaId AS miningAreaId, \
                MiningAreaLifetimeResult.oreId AS oreId, \
                CAST(CASE WHEN MiningAreaLifetimeResult.totalRuns > 0 \
                          THEN MiningAreaLifetimeResult.totalAmount / MiningAreaLifetimeResult.totalRuns \
                          ELSE 0.0 END AS DOUBLE) AS averageOrePerRun \
         FROM MiningAreaLifetimeResult \
         INNER JOIN UserMiningArea \
           ON UserMiningArea.miningAreaId = MiningAreaLifetimeResult.miningAreaId \
         WHERE UserMiningArea.userId = ? \
         ORDER BY MiningAreaLifetimeResult.miningAreaId, MiningAreaLifetimeResult.oreId",
    )
    .bind(user_id)
    .fetch_all(pool)
    .await
    .map(|rows| {
        rows.into_iter()
            .map(MiningAreaOverviewOreAverageRecord::from)
            .collect()
    })
}
