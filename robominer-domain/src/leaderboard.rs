use robominer_db::MySqlPool;

use crate::error::DomainError;

pub async fn list_leaderboard_mining_areas(
    pool: &MySqlPool,
) -> Result<Vec<robominer_db::LeaderboardMiningAreaRecord>, DomainError> {
    robominer_db::list_leaderboard_mining_areas(pool)
        .await
        .map_err(DomainError::Database)
}

pub async fn list_leaderboard_mining_area_scores(
    pool: &MySqlPool,
    maximum_results: i64,
) -> Result<Vec<robominer_db::LeaderboardMiningAreaScoreRecord>, DomainError> {
    robominer_db::list_leaderboard_mining_area_scores(pool, maximum_results)
        .await
        .map_err(DomainError::Database)
}

pub async fn list_leaderboard_top_robots(
    pool: &MySqlPool,
    maximum_results: i64,
) -> Result<Vec<robominer_db::LeaderboardTopRobotRecord>, DomainError> {
    robominer_db::list_leaderboard_top_robots(pool, maximum_results)
        .await
        .map_err(DomainError::Database)
}

pub async fn list_leaderboard_top_users(
    pool: &MySqlPool,
    maximum_results: i64,
) -> Result<Vec<robominer_db::LeaderboardTopUserRecord>, DomainError> {
    robominer_db::list_leaderboard_top_users(pool, maximum_results)
        .await
        .map_err(DomainError::Database)
}

pub async fn load_leaderboard_viewer_standing(
    pool: &MySqlPool,
    user_id: i64,
) -> Result<robominer_db::LeaderboardViewerStandingRecord, DomainError> {
    robominer_db::load_leaderboard_viewer_standing(pool, user_id)
        .await
        .map_err(DomainError::Database)
}
