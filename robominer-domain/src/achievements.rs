use robominer_db::MySqlPool;

use crate::error::DomainError;

pub async fn list_achievement_claim_states(
    pool: &MySqlPool,
    user_id: i64,
) -> Result<Vec<robominer_db::AchievementClaimStateRecord>, DomainError> {
    robominer_db::list_achievement_claim_states_for_user(pool, user_id)
        .await
        .map_err(DomainError::Database)
}
pub async fn list_achievement_page_states(
    pool: &MySqlPool,
    user_id: i64,
) -> Result<Vec<robominer_db::AchievementPageStateRecord>, DomainError> {
    robominer_db::list_achievement_page_states_for_user(pool, user_id)
        .await
        .map_err(DomainError::Database)
}

pub async fn list_achievement_page_total_requirements(
    pool: &MySqlPool,
    user_id: i64,
) -> Result<Vec<robominer_db::AchievementPageTotalRequirementRecord>, DomainError> {
    robominer_db::list_achievement_page_total_requirements_for_user(pool, user_id)
        .await
        .map_err(DomainError::Database)
}

pub async fn list_achievement_page_score_requirements(
    pool: &MySqlPool,
    user_id: i64,
) -> Result<Vec<robominer_db::AchievementPageScoreRequirementRecord>, DomainError> {
    robominer_db::list_achievement_page_score_requirements_for_user(pool, user_id)
        .await
        .map_err(DomainError::Database)
}

pub async fn load_achievement_page_points_summary(
    pool: &MySqlPool,
    user_id: i64,
) -> Result<robominer_db::AchievementPagePointsSummaryRecord, DomainError> {
    robominer_db::load_achievement_page_points_summary_for_user(pool, user_id)
        .await
        .map_err(DomainError::Database)
}

pub async fn claim_achievement_step(
    pool: &MySqlPool,
    request: robominer_db::ClaimAchievementStepRequest,
) -> Result<
    Result<robominer_db::ClaimedAchievementStep, robominer_db::ClaimAchievementStepRejection>,
    DomainError,
> {
    robominer_db::claim_achievement_step(pool, request)
        .await
        .map_err(DomainError::Database)
}
