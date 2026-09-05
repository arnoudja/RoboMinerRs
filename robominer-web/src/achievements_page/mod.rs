use crate::{Request, Response, ServerConfig, mutation_i64};

mod actions;
mod card;
mod overview;
mod render;

#[cfg(test)]
mod tests;

use actions::claim_achievement;

#[derive(Debug)]
pub(super) struct AchievementsPageState {
    /// When set, the page is a read-only overview of this player (or not-found).
    pub(super) viewed_username: Option<String>,
    pub(super) player_not_found: bool,
    pub(super) overview_tracks: Vec<robominer_db::AchievementOverviewTrackRecord>,
    pub(super) robot_count: i64,
    pub(super) achievements: Vec<robominer_db::AchievementPageStateRecord>,
    pub(super) total_requirements: Vec<robominer_db::AchievementPageTotalRequirementRecord>,
    pub(super) score_requirements: Vec<robominer_db::AchievementPageScoreRequirementRecord>,
    pub(super) depot_total_requirements:
        Vec<robominer_db::AchievementPageDepotTotalRequirementRecord>,
    pub(super) points_summary: robominer_db::AchievementPagePointsSummaryRecord,
    pub(super) claim_message: Option<String>,
}

pub(super) async fn achievements_page(
    request: &Request,
    config: &ServerConfig,
    session: crate::page_context::PageSession<'_>,
) -> Response {
    // Prefer the DB username for self vs overview. The unsigned
    // `robominer_username` cookie is display-only and must not drive authz.
    let session_name = match robominer_db::users::get_user_by_id(session.pool, session.user_id).await {
        Ok(Some(user)) => user.username,
        Ok(None) => {
            return crate::page_context::page_load_error(
                "achievements",
                sqlx::Error::RowNotFound.into(),
            );
        }
        Err(error) => {
            return crate::page_context::page_load_error("achievements", error.into());
        }
    };
    let requested_user = request
        .query
        .get("user")
        .map(|value| value.trim())
        .filter(|value| !value.is_empty())
        .map(str::to_string);

    let result = match requested_user {
        Some(username) if username != session_name => {
            load_achievements_overview(session.pool, username).await
        }
        _ => {
            let achievement_id = mutation_i64(request, "achievementId");
            load_achievements_state(session.pool, session.user_id, achievement_id).await
        }
    };

    match result {
        Ok(state) => {
            session
                .html_with_hud(request, config, |username, hud| {
                    render::render_achievements_page(username, hud, &state)
                })
                .await
        }
        Err(error) => crate::page_context::page_load_error("achievements", error),
    }
}

async fn load_achievements_state(
    pool: &robominer_db::MySqlPool,
    user_id: i64,
    achievement_id: Option<i64>,
) -> Result<AchievementsPageState, crate::page_context::PageLoadError> {
    let claim_message = if let Some(achievement_id) = achievement_id {
        claim_achievement(pool, user_id, achievement_id).await?
    } else {
        None
    };

    Ok(AchievementsPageState {
        viewed_username: None,
        player_not_found: false,
        overview_tracks: Vec::new(),
        robot_count: robominer_db::count_user_robots(pool, user_id).await?,
        achievements: robominer_db::list_achievement_page_states_for_user(pool, user_id).await?,
        total_requirements: robominer_db::list_achievement_page_total_requirements_for_user(
            pool, user_id,
        )
        .await?,
        score_requirements: robominer_db::list_achievement_page_score_requirements_for_user(
            pool, user_id,
        )
        .await?,
        depot_total_requirements:
            robominer_db::list_achievement_page_depot_total_requirements_for_user(pool, user_id)
                .await?,
        points_summary: robominer_db::load_achievement_page_points_summary_for_user(pool, user_id)
            .await?,
        claim_message,
    })
}

async fn load_achievements_overview(
    pool: &robominer_db::MySqlPool,
    username: String,
) -> Result<AchievementsPageState, crate::page_context::PageLoadError> {
    let Some(target_user_id) = robominer_db::get_user_id_by_username(pool, &username).await? else {
        return Ok(AchievementsPageState {
            viewed_username: Some(username),
            player_not_found: true,
            overview_tracks: Vec::new(),
            robot_count: 0,
            achievements: Vec::new(),
            total_requirements: Vec::new(),
            score_requirements: Vec::new(),
            depot_total_requirements: Vec::new(),
            points_summary: robominer_db::AchievementPagePointsSummaryRecord {
                points_earned: 0,
                points_achievable: 0,
            },
            claim_message: None,
        });
    };

    Ok(AchievementsPageState {
        viewed_username: Some(username),
        player_not_found: false,
        overview_tracks: robominer_db::list_achievement_overview_tracks_for_user(
            pool,
            target_user_id,
        )
        .await?,
        robot_count: 0,
        achievements: Vec::new(),
        total_requirements: Vec::new(),
        score_requirements: Vec::new(),
        depot_total_requirements: Vec::new(),
        points_summary: robominer_db::load_achievement_page_points_summary_for_user(
            pool,
            target_user_id,
        )
        .await?,
        claim_message: None,
    })
}
