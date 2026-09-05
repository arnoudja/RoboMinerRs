//! Achievement claim mutations for the achievements page.

pub(super) async fn claim_achievement(
    pool: &robominer_db::MySqlPool,
    user_id: i64,
    achievement_id: i64,
) -> Result<Option<String>, crate::page_context::PageLoadError> {
    match robominer_db::achievements::claim_achievement_step(
        pool,
        robominer_db::ClaimAchievementStepRequest {
            user_id,
            achievement_id,
        },
    )
    .await?
    {
        robominer_db::DbOutcome::Success(_) => Ok(Some("Achievement claimed".to_string())),
        robominer_db::DbOutcome::Rejected(rejection) => Ok(Some(format!(
            "Unable to claim achievement: {}",
            robominer_domain::rejection_messages::claim_achievement_step_rejection_message(
                rejection
            )
        ))),
    }
}
