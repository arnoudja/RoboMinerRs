use crate::{Request, Response, ServerConfig, is_post, mutation_i64, query_i64};

#[derive(Debug)]
pub(super) struct RobotPageState {
    pub(super) selected_robot_id: i64,
    pub(super) program_sources: Vec<robominer_db::ProgramSourceRecord>,
    pub(super) robots: Vec<robominer_db::RobotConfigStateRecord>,
    pub(super) part_assets: Vec<robominer_db::RobotConfigPartAssetStateRecord>,
    pub(super) message: Option<String>,
    pub(super) pending_claim_count: u64,
    pub(super) claimed_results: robominer_db::ClaimedUserResults,
}

pub(super) async fn robot_page(request: &Request, config: &ServerConfig) -> Response {
    let session = match crate::page_context::PageSession::require(
        request,
        config,
        "Robot page requires ROBOMINER_DATABASE_URL to be configured",
    ) {
        Ok(session) => session,
        Err(response) => return response,
    };
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
    let claim_state =
        crate::page_context::resolve_mining_claim_state(pool, user_id, request).await?;

    let mut message = None;
    if is_post(request)
        && let Some(robot_id) = mutation_i64(request, "robotId")
        && crate::mutation_form_has(request, &format!("robotName{robot_id}"))
    {
        let robot_name = request
            .form
            .get(&format!("robotName{robot_id}"))
            .cloned()
            .unwrap_or_default();
        let result = robominer_db::update_robot_config(
            pool,
            robominer_db::UpdateRobotConfigRequest {
                user_id,
                robot_id,
                robot_name,
                program_source_id: mutation_i64(request, &format!("programSourceId{robot_id}"))
                    .unwrap_or(0),
                ore_container_id: mutation_i64(request, &format!("oreContainerId{robot_id}"))
                    .unwrap_or(0),
                mining_unit_id: mutation_i64(request, &format!("miningUnitId{robot_id}"))
                    .unwrap_or(0),
                battery_id: mutation_i64(request, &format!("batteryId{robot_id}")).unwrap_or(0),
                memory_module_id: mutation_i64(request, &format!("memoryModuleId{robot_id}"))
                    .unwrap_or(0),
                cpu_id: mutation_i64(request, &format!("cpuId{robot_id}")).unwrap_or(0),
                engine_id: mutation_i64(request, &format!("engineId{robot_id}")).unwrap_or(0),
                ore_scanner_id: mutation_i64(request, &format!("oreScannerId{robot_id}"))
                    .unwrap_or(0),
            },
        )
        .await?;

        message = Some(if let Err(rejection) = result.into_result() {
            format!(
                "Unable to apply robot changes: {}",
                robominer_domain::rejection_messages::update_robot_config_rejection_player_message(
                    rejection
                )
            )
        } else {
            "Robot changes queued".to_string()
        });
    }

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
        pending_claim_count: claim_state.pending_count,
        claimed_results: claim_state.claimed_results,
    })
}

mod config;
mod config_parts;
mod config_stats;
mod fleet;
mod render;
mod scripts;

#[cfg(test)]
mod tests;
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
