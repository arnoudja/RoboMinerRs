use crate::{
    Request, Response, ServerConfig, block_on_database, login_redirect, query_i64, session_username,
};

#[derive(Debug)]
pub(super) struct AchievementsPageState {
    pub(super) robot_count: i64,
    pub(super) achievements: Vec<robominer_db::AchievementPageStateRecord>,
    pub(super) total_requirements: Vec<robominer_db::AchievementPageTotalRequirementRecord>,
    pub(super) score_requirements: Vec<robominer_db::AchievementPageScoreRequirementRecord>,
    pub(super) points_summary: robominer_db::AchievementPagePointsSummaryRecord,
    pub(super) claim_message: Option<String>,
}

pub(super) fn achievements_page(request: &Request, config: &ServerConfig) -> Response {
    let Some(user_id) = crate::request_user_id(request) else {
        return login_redirect(request);
    };
    let Some(pool) = config.database_pool.as_ref() else {
        return Response::service_unavailable(
            "Achievements require ROBOMINER_DATABASE_URL to be configured",
        );
    };
    let achievement_id = query_i64(request, "achievementId");

    let result = block_on_database(load_achievements_state(pool, user_id, achievement_id));

    match result {
        Ok(state) => Response::html(render::render_achievements_page(
            session_username(request),
            crate::app_shell::hud_markup(request, config).as_deref(),
            &state,
        )),
        Err(error) => {
            Response::service_unavailable(format!("Unable to load achievements: {error}"))
        }
    }
}

async fn load_achievements_state(
    pool: &robominer_db::MySqlPool,
    user_id: i64,
    achievement_id: Option<i64>,
) -> Result<AchievementsPageState, robominer_domain::DomainError> {
    robominer_domain::claim_user_results(pool, user_id).await?;

    let claim_message = if let Some(achievement_id) = achievement_id {
        match robominer_domain::claim_achievement_step(
            pool,
            robominer_db::ClaimAchievementStepRequest {
                user_id,
                achievement_id,
            },
        )
        .await?
        {
            Ok(_) => Some("Achievement claimed".to_string()),
            Err(rejection) => Some(format!(
                "Unable to claim achievement: {}",
                claim_achievement_step_rejection_message(rejection)
            )),
        }
    } else {
        None
    };

    Ok(AchievementsPageState {
        robot_count: robominer_domain::count_user_robots(pool, user_id).await?,
        achievements: robominer_domain::list_achievement_page_states(pool, user_id).await?,
        total_requirements: robominer_domain::list_achievement_page_total_requirements(
            pool, user_id,
        )
        .await?,
        score_requirements: robominer_domain::list_achievement_page_score_requirements(
            pool, user_id,
        )
        .await?,
        points_summary: robominer_domain::load_achievement_page_points_summary(pool, user_id)
            .await?,
        claim_message,
    })
}

pub(super) fn claim_achievement_step_rejection_message(
    rejection: robominer_db::ClaimAchievementStepRejection,
) -> &'static str {
    robominer_domain::claim_achievement_step_rejection_message(rejection)
}

mod render;

#[cfg(test)]
mod tests;
