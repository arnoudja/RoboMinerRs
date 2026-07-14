use robominer_db::MySqlPool;

use crate::error::DomainError;

pub async fn list_mining_area_overview_ores(
    pool: &MySqlPool,
) -> Result<Vec<robominer_db::MiningAreaOverviewOreRecord>, DomainError> {
    robominer_db::list_mining_area_overview_ores(pool)
        .await
        .map_err(DomainError::Database)
}

pub async fn list_mining_area_overview_ores_for_user(
    pool: &MySqlPool,
    user_id: i64,
) -> Result<Vec<robominer_db::MiningAreaOverviewOreRecord>, DomainError> {
    robominer_db::list_mining_area_overview_ores_for_user(pool, user_id)
        .await
        .map_err(DomainError::Database)
}

pub async fn list_mining_area_overview_areas(
    pool: &MySqlPool,
) -> Result<Vec<robominer_db::MiningAreaOverviewAreaRecord>, DomainError> {
    robominer_db::list_mining_area_overview_areas(pool)
        .await
        .map_err(DomainError::Database)
}

pub async fn list_mining_area_overview_areas_for_user(
    pool: &MySqlPool,
    user_id: i64,
) -> Result<Vec<robominer_db::MiningAreaOverviewAreaRecord>, DomainError> {
    robominer_db::list_mining_area_overview_areas_for_user(pool, user_id)
        .await
        .map_err(DomainError::Database)
}

pub async fn list_mining_area_overview_percentages(
    pool: &MySqlPool,
) -> Result<Vec<robominer_db::MiningAreaOverviewPercentageRecord>, DomainError> {
    robominer_db::list_mining_area_overview_percentages(pool)
        .await
        .map_err(DomainError::Database)
}

pub async fn list_mining_area_overview_percentages_for_user(
    pool: &MySqlPool,
    user_id: i64,
) -> Result<Vec<robominer_db::MiningAreaOverviewPercentageRecord>, DomainError> {
    robominer_db::list_mining_area_overview_percentages_for_user(pool, user_id)
        .await
        .map_err(DomainError::Database)
}
