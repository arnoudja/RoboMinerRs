use sqlx::MySqlPool;

use crate::{
    MiningAreaOverviewAreaRecord, MiningAreaOverviewOreAverageRecord, MiningAreaOverviewOreRecord,
};

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
                CAST(COALESCE(SUM(CASE WHEN MiningAreaLifetimeResult.totalRuns > 0 \
                                        THEN MiningAreaLifetimeResult.totalAmount / MiningAreaLifetimeResult.totalRuns \
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
                CAST(COALESCE(SUM(CASE WHEN MiningAreaLifetimeResult.totalRuns > 0 \
                                        THEN MiningAreaLifetimeResult.totalAmount / MiningAreaLifetimeResult.totalRuns \
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
        .map(|(mining_area_id, area_name, total_average_ore_per_run)| {
            MiningAreaOverviewAreaRecord {
                mining_area_id,
                area_name,
                total_average_ore_per_run,
            }
        })
        .collect()
}

pub async fn list_mining_area_overview_ore_averages(
    pool: &MySqlPool,
) -> Result<Vec<MiningAreaOverviewOreAverageRecord>, sqlx::Error> {
    sqlx::query_as::<_, (i64, i64, f64)>(
        "SELECT miningAreaId, oreId, \
                CAST(CASE WHEN totalRuns > 0 \
                          THEN totalAmount / totalRuns \
                          ELSE 0.0 END AS DOUBLE) \
         FROM MiningAreaLifetimeResult \
         ORDER BY miningAreaId, oreId",
    )
    .fetch_all(pool)
    .await
    .map(map_mining_area_overview_ore_average_rows)
}

pub async fn list_mining_area_overview_ore_averages_for_user(
    pool: &MySqlPool,
    user_id: i64,
) -> Result<Vec<MiningAreaOverviewOreAverageRecord>, sqlx::Error> {
    sqlx::query_as::<_, (i64, i64, f64)>(
        "SELECT MiningAreaLifetimeResult.miningAreaId, MiningAreaLifetimeResult.oreId, \
                CAST(CASE WHEN MiningAreaLifetimeResult.totalRuns > 0 \
                          THEN MiningAreaLifetimeResult.totalAmount / MiningAreaLifetimeResult.totalRuns \
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
    .map(map_mining_area_overview_ore_average_rows)
}

fn map_mining_area_overview_ore_average_rows(
    rows: Vec<(i64, i64, f64)>,
) -> Vec<MiningAreaOverviewOreAverageRecord> {
    rows.into_iter()
        .map(
            |(mining_area_id, ore_id, average_ore_per_run)| MiningAreaOverviewOreAverageRecord {
                mining_area_id,
                ore_id,
                average_ore_per_run,
            },
        )
        .collect()
}
