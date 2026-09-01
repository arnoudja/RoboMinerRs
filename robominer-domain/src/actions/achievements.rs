use crate::DomainError;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ClaimAchievementStepOutcome {
    Success(robominer_db::ClaimedAchievementStep),
    Rejected(robominer_db::ClaimAchievementStepRejection),
}

pub async fn claim_achievement_step(
    pool: &robominer_db::MySqlPool,
    request: robominer_db::ClaimAchievementStepRequest,
) -> Result<ClaimAchievementStepOutcome, DomainError> {
    match robominer_db::claim_achievement_step(pool, request).await? {
        robominer_db::DbOutcome::Success(value) => Ok(ClaimAchievementStepOutcome::Success(value)),
        robominer_db::DbOutcome::Rejected(rejection) => {
            Ok(ClaimAchievementStepOutcome::Rejected(rejection))
        }
    }
}
