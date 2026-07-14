use robominer_db::MySqlPool;

use crate::error::DomainError;

pub async fn list_user_ore_asset_states(
    pool: &MySqlPool,
    user_id: i64,
) -> Result<Vec<robominer_db::UserOreAssetStateRecord>, DomainError> {
    robominer_db::list_user_ore_asset_states(pool, user_id)
        .await
        .map_err(DomainError::Database)
}

pub async fn load_user_asset_summary(
    pool: &MySqlPool,
    user_id: i64,
) -> Result<robominer_db::UserAssetSummaryRecord, DomainError> {
    robominer_db::load_user_asset_summary(pool, user_id)
        .await
        .map_err(DomainError::Database)
}

pub async fn get_user_by_id(
    pool: &MySqlPool,
    user_id: i64,
) -> Result<Option<robominer_db::UserRecord>, DomainError> {
    robominer_db::get_user_by_id(pool, user_id)
        .await
        .map_err(DomainError::Database)
}

pub async fn list_robot_mining_area_scores(
    pool: &MySqlPool,
    user_id: i64,
) -> Result<Vec<robominer_db::RobotMiningAreaScoreRecord>, DomainError> {
    robominer_db::list_robot_mining_area_scores_for_user(pool, user_id)
        .await
        .map_err(DomainError::Database)
}

pub async fn list_user_ore_mined_totals(
    pool: &MySqlPool,
    user_id: i64,
) -> Result<Vec<robominer_db::UserOreMinedRecord>, DomainError> {
    robominer_db::list_user_ore_mined_totals(pool, user_id)
        .await
        .map_err(DomainError::Database)
}

pub async fn list_user_best_mining_area_scores(
    pool: &MySqlPool,
    user_id: i64,
) -> Result<Vec<robominer_db::UserMiningAreaScoreRecord>, DomainError> {
    robominer_db::list_user_best_mining_area_scores(pool, user_id)
        .await
        .map_err(DomainError::Database)
}

pub async fn count_user_robots(pool: &MySqlPool, user_id: i64) -> Result<i64, DomainError> {
    robominer_db::count_user_robots(pool, user_id)
        .await
        .map_err(DomainError::Database)
}
