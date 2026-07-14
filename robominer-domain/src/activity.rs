use robominer_db::MySqlPool;

use crate::error::DomainError;

pub async fn list_activity_recent_users(
    pool: &MySqlPool,
    maximum_users: i64,
) -> Result<Vec<robominer_db::ActivityRecentUserRecord>, DomainError> {
    robominer_db::list_activity_recent_users(pool, maximum_users)
        .await
        .map_err(DomainError::Database)
}

pub async fn list_activity_recent_rallies(
    pool: &MySqlPool,
    maximum_rallies: i64,
) -> Result<Vec<robominer_db::ActivityRecentRallyRecord>, DomainError> {
    robominer_db::list_activity_recent_rallies(pool, maximum_rallies)
        .await
        .map_err(DomainError::Database)
}

pub async fn list_activity_recent_rally_feed(
    pool: &MySqlPool,
    user_id: Option<i64>,
    mining_area_id: Option<i64>,
    limit: i64,
) -> Result<(Vec<robominer_db::ActivityRecentRallyRecord>, bool), DomainError> {
    robominer_db::list_activity_recent_rally_feed(pool, user_id, mining_area_id, limit)
        .await
        .map_err(DomainError::Database)
}

pub async fn list_activity_rally_area_options(
    pool: &MySqlPool,
    maximum_areas: i64,
) -> Result<Vec<robominer_db::ActivityRallyAreaOption>, DomainError> {
    robominer_db::list_activity_rally_area_options(pool, maximum_areas)
        .await
        .map_err(DomainError::Database)
}

pub async fn list_activity_recent_rally_participants(
    pool: &MySqlPool,
    maximum_rallies: i64,
) -> Result<Vec<robominer_db::ActivityRecentRallyParticipantRecord>, DomainError> {
    robominer_db::list_activity_recent_rally_participants(pool, maximum_rallies)
        .await
        .map_err(DomainError::Database)
}

pub async fn list_activity_rally_participants_for_queues(
    pool: &MySqlPool,
    mining_queue_ids: &[i64],
) -> Result<Vec<robominer_db::ActivityRecentRallyParticipantRecord>, DomainError> {
    robominer_db::list_activity_rally_participants_for_queues(pool, mining_queue_ids)
        .await
        .map_err(DomainError::Database)
}

pub async fn rally_view_state(
    pool: &MySqlPool,
    user_id: i64,
    rally_result_id: i64,
    require_user_result: bool,
) -> Result<Option<robominer_db::RallyViewStateRecord>, DomainError> {
    robominer_db::rally_view_state(pool, user_id, rally_result_id, require_user_result)
        .await
        .map_err(DomainError::Database)
}

pub async fn list_rally_view_participants(
    pool: &MySqlPool,
    rally_result_id: i64,
) -> Result<Vec<robominer_db::RallyViewParticipantRecord>, DomainError> {
    robominer_db::list_rally_view_participants(pool, rally_result_id)
        .await
        .map_err(DomainError::Database)
}

pub async fn rally_view_metadata(
    pool: &MySqlPool,
    user_id: i64,
    rally_result_id: i64,
    require_claimed_viewer_result: bool,
) -> Result<Option<robominer_db::RallyViewMetadataRecord>, DomainError> {
    robominer_db::rally_view_metadata(
        pool,
        user_id,
        rally_result_id,
        require_claimed_viewer_result,
    )
    .await
    .map_err(DomainError::Database)
}
