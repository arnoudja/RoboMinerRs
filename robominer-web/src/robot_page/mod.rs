use std::collections::HashMap;

use crate::html::escape_html;
use crate::{
    Request, Response, ServerConfig, block_on_database, login_redirect, query_i64, session_username,
};

#[derive(Debug)]
pub(super) struct RobotPageState {
    pub(super) selected_robot_id: i64,
    pub(super) program_sources: Vec<robominer_db::ProgramSourceRecord>,
    pub(super) robots: Vec<robominer_db::RobotConfigStateRecord>,
    pub(super) part_assets: Vec<robominer_db::RobotConfigPartAssetStateRecord>,
    pub(super) message: Option<String>,
    pub(super) claimed_results: robominer_db::ClaimedUserResults,
}

pub(super) fn robot_page(request: &Request, config: &ServerConfig) -> Response {
    let Some(user_id) = crate::request_user_id(request) else {
        return login_redirect(request);
    };
    let Some(pool) = config.database_pool.as_ref() else {
        return Response::service_unavailable(
            "Robot page requires ROBOMINER_DATABASE_URL to be configured",
        );
    };
    let robot_id = query_i64(request, "robotId");

    let result = block_on_database(load_robot_page_state(pool, user_id, request, robot_id));

    match result {
        Ok(state) => Response::html(render::render_robot_page(
            session_username(request),
            crate::app_shell::hud_markup(request, config).as_deref(),
            &state,
        )),
        Err(error) => Response::service_unavailable(format!("Unable to load robot page: {error}")),
    }
}

async fn load_robot_page_state(
    pool: &robominer_db::MySqlPool,
    user_id: i64,
    request: &Request,
    requested_robot_id: Option<i64>,
) -> Result<RobotPageState, robominer_domain::DomainError> {
    let claim_result = robominer_domain::claim_user_results(pool, user_id).await?;

    let mut message = None;
    if let Some(robot_id) = requested_robot_id
        && request.form.contains_key(&format!("robotName{robot_id}"))
    {
        let robot_name = request
            .form
            .get(&format!("robotName{robot_id}"))
            .cloned()
            .unwrap_or_default();
        let result = robominer_domain::update_robot_config(
            pool,
            robominer_db::UpdateRobotConfigRequest {
                user_id,
                robot_id,
                robot_name,
                program_source_id: query_i64(request, &format!("programSourceId{robot_id}"))
                    .unwrap_or(0),
                ore_container_id: query_i64(request, &format!("oreContainerId{robot_id}"))
                    .unwrap_or(0),
                mining_unit_id: query_i64(request, &format!("miningUnitId{robot_id}")).unwrap_or(0),
                battery_id: query_i64(request, &format!("batteryId{robot_id}")).unwrap_or(0),
                memory_module_id: query_i64(request, &format!("memoryModuleId{robot_id}"))
                    .unwrap_or(0),
                cpu_id: query_i64(request, &format!("cpuId{robot_id}")).unwrap_or(0),
                engine_id: query_i64(request, &format!("engineId{robot_id}")).unwrap_or(0),
                ore_scanner_id: query_i64(request, &format!("oreScannerId{robot_id}"))
                    .unwrap_or(0),
            },
        )
        .await?;

        message = Some(if let Err(rejection) = result {
            format!(
                "Unable to apply robot changes: {}",
                update_robot_config_rejection_message(rejection)
            )
        } else {
            "Robot changes queued".to_string()
        });
    }

    let robots = robominer_domain::list_robot_config_states(pool, user_id).await?;
    let selected_robot_id = requested_robot_id
        .filter(|robot_id| robots.iter().any(|robot| robot.robot_id == *robot_id))
        .or_else(|| robots.first().map(|robot| robot.robot_id))
        .unwrap_or(0);

    Ok(RobotPageState {
        selected_robot_id,
        program_sources: robominer_domain::list_robot_config_program_sources(pool, user_id).await?,
        robots,
        part_assets: robominer_domain::list_robot_config_part_asset_states(pool, user_id).await?,
        message,
        claimed_results: claim_result,
    })
}


mod render;

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

#[allow(clippy::too_many_arguments)]
fn render_robot_part_select(
    body: &mut String,
    label: &str,
    field_prefix: &str,
    robot_id: i64,
    type_id: i64,
    current_part_id: i64,
    current_part_name: &str,
    part_asset_map: &HashMap<i64, Vec<&robominer_db::RobotConfigPartAssetStateRecord>>,
    memory_control: bool,
    disabled_attr: &str,
    current_memory_capacity: Option<i32>,
) {
    let id_attr = if memory_control {
        format!(r#" id="{field_prefix}{robot_id}""#)
    } else {
        String::new()
    };
    let current_capacity_attr = current_memory_capacity
        .map(|capacity| format!(r#" data-memory-capacity="{capacity}""#))
        .unwrap_or_default();
    body.push_str(&format!(
        r#"<label class="robot-field"><span class="robot-field-label">{}</span><select{id_attr} name="{}{}" class="tableitem robot-select"{disabled_attr}><option value="{}"{current_capacity_attr} selected="selected">{}</option>"#,
        label,
        field_prefix,
        robot_id,
        current_part_id,
        escape_html(current_part_name)
    ));
    for asset in part_asset_map.get(&type_id).into_iter().flatten() {
        if asset.unassigned > 0 && asset.robot_part_id != current_part_id {
            let capacity_attr = if asset.memory_capacity > 0 {
                format!(r#" data-memory-capacity="{}""#, asset.memory_capacity)
            } else {
                String::new()
            };
            body.push_str(&format!(
                r#"<option value="{}"{capacity_attr}>{}</option>"#,
                asset.robot_part_id,
                escape_html(&asset.part_name)
            ));
        }
    }
    body.push_str("</select></label>");
}

fn push_robot_highlight(body: &mut String, label: &str, value: i32, suffix: &str) {
    if value > 0 {
        body.push_str(&format!(
            r#"<span class="robot-stat-highlight"><span class="robot-stat-highlight-label">{label}</span><span class="robot-stat-highlight-value">{value}{suffix}</span></span>"#,
        ));
    }
}

fn add_robot_stat_entry(body: &mut String, label: &str, value: String) {
    body.push_str(&format!(
        r#"<div class="robot-stat"><dt>{label}</dt><dd>{value}</dd></div>"#,
    ));
}

fn robot_memory_percent(program_size: i32, memory_size: i32) -> f64 {
    if memory_size <= 0 {
        return 100.0;
    }
    ((program_size as f64 / memory_size as f64) * 100.0).clamp(0.0, 100.0)
}

pub(super) fn update_robot_config_rejection_message(
    rejection: robominer_db::UpdateRobotConfigRejection,
) -> &'static str {
    robominer_domain::update_robot_config_rejection_player_message(rejection)
}

