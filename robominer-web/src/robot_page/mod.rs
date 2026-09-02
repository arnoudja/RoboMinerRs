use crate::{Request, Response, ServerConfig, query_i64};

mod actions;
mod config;
mod config_parts;
mod config_stats;
mod fleet;
mod render;
mod scripts;

#[cfg(test)]
mod tests;

use actions::apply_robot_config_mutation;

#[derive(Debug)]
pub(super) struct RobotPageState {
    pub(super) selected_robot_id: i64,
    pub(super) program_sources: Vec<robominer_db::ProgramSourceRecord>,
    pub(super) robots: Vec<robominer_db::RobotConfigStateRecord>,
    pub(super) part_assets: Vec<robominer_db::RobotConfigPartAssetStateRecord>,
    pub(super) message: Option<String>,
}

pub(super) async fn robot_page(
    request: &Request,
    config: &ServerConfig,
    session: crate::page_context::PageSession<'_>,
) -> Response {
    let robot_id = query_i64(request, "robotId");

    let result = load_robot_page_state(session.pool, session.user_id, request, robot_id).await;

    match result {
        Ok(state) => {
            session
                .html_with_hud(request, config, |username, hud| {
                    render::render_robot_page(username, hud, &state)
                })
                .await
        }
        Err(error) => crate::page_context::page_load_error("robot page", error),
    }
}

async fn load_robot_page_state(
    pool: &robominer_db::MySqlPool,
    user_id: i64,
    request: &Request,
    requested_robot_id: Option<i64>,
) -> Result<RobotPageState, crate::page_context::PageLoadError> {
    let message = apply_robot_config_mutation(pool, user_id, request).await?;

    let robots = robominer_db::list_robot_config_states(pool, user_id).await?;
    let selected_robot_id = requested_robot_id
        .filter(|robot_id| robots.iter().any(|robot| robot.robot_id == *robot_id))
        .or_else(|| robots.first().map(|robot| robot.robot_id))
        .unwrap_or(0);

    Ok(RobotPageState {
        selected_robot_id,
        program_sources: robominer_db::list_program_sources_for_user(pool, user_id).await?,
        robots,
        part_assets: robominer_db::list_robot_config_part_asset_states(pool, user_id).await?,
        message,
    })
}

pub(super) fn robot_apply_block_reason(
    robot: &robominer_db::RobotConfigStateRecord,
    program_sources: &[robominer_db::ProgramSourceRecord],
) -> Option<&'static str> {
    let program_source = program_sources
        .iter()
        .find(|program_source| program_source.id == robot.program_source_id)?;
    if program_source.compiled_size > robot.memory_size {
        return Some("Not enough memory available.");
    }
    None
}
