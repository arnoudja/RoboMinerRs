//! Achievement claim mutations for the achievements page.

use robominer_domain::{ClaimAchievementStepOutcome, DomainError};

fn achievement_mutation_error(error: DomainError) -> crate::page_context::PageLoadError {
    crate::page_context::PageLoadError::from_database(error).unwrap_or_else(|_| {
        crate::page_context::PageLoadError::from(sqlx::Error::Configuration(
            "unexpected domain error on achievement claim".into(),
        ))
    })
}

pub(super) async fn claim_achievement(
    pool: &robominer_db::MySqlPool,
    user_id: i64,
    achievement_id: i64,
) -> Result<Option<String>, crate::page_context::PageLoadError> {
    match robominer_domain::claim_achievement_step(
        pool,
        robominer_db::ClaimAchievementStepRequest {
            user_id,
            achievement_id,
        },
    )
    .await
    .map_err(achievement_mutation_error)?
    {
        ClaimAchievementStepOutcome::Success(_) => Ok(Some("Achievement claimed".to_string())),
        ClaimAchievementStepOutcome::Rejected(rejection) => Ok(Some(format!(
            "Unable to claim achievement: {}",
            robominer_domain::rejection_messages::claim_achievement_step_rejection_message(
                rejection
            )
        ))),
    }
}
