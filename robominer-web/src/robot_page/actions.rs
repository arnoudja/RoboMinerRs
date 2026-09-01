//! Robot configuration mutations for the robot page.

use crate::{Request, mutation_form_has, mutation_i64};
use robominer_domain::{DomainError, UpdateRobotConfigOutcome};

fn robot_mutation_error(error: DomainError) -> crate::page_context::PageLoadError {
    crate::page_context::PageLoadError::from_database(error).unwrap_or_else(|_| {
        crate::page_context::PageLoadError::from(sqlx::Error::Configuration(
            "unexpected domain error on robot config update".into(),
        ))
    })
}

pub(super) async fn apply_robot_config_mutation(
    pool: &robominer_db::MySqlPool,
    user_id: i64,
    request: &Request,
) -> Result<Option<String>, crate::page_context::PageLoadError> {
    let Some(robot_id) = mutation_i64(request, "robotId") else {
        return Ok(None);
    };
    if !mutation_form_has(request, &format!("robotName{robot_id}")) {
        return Ok(None);
    }

    let robot_name = request
        .form
        .get(&format!("robotName{robot_id}"))
        .cloned()
        .unwrap_or_default();

    match robominer_domain::update_robot_config(
        pool,
        robominer_db::UpdateRobotConfigRequest {
            user_id,
            robot_id,
            robot_name,
            program_source_id: mutation_i64(request, &format!("programSourceId{robot_id}"))
                .unwrap_or(0),
            ore_container_id: mutation_i64(request, &format!("oreContainerId{robot_id}"))
                .unwrap_or(0),
            mining_unit_id: mutation_i64(request, &format!("miningUnitId{robot_id}")).unwrap_or(0),
            battery_id: mutation_i64(request, &format!("batteryId{robot_id}")).unwrap_or(0),
            memory_module_id: mutation_i64(request, &format!("memoryModuleId{robot_id}"))
                .unwrap_or(0),
            cpu_id: mutation_i64(request, &format!("cpuId{robot_id}")).unwrap_or(0),
            engine_id: mutation_i64(request, &format!("engineId{robot_id}")).unwrap_or(0),
            ore_scanner_id: mutation_i64(request, &format!("oreScannerId{robot_id}")).unwrap_or(0),
        },
    )
    .await
    .map_err(robot_mutation_error)?
    {
        UpdateRobotConfigOutcome::Success(_) => Ok(Some("Robot changes queued".to_string())),
        UpdateRobotConfigOutcome::Rejected(rejection) => Ok(Some(format!(
            "Unable to apply robot changes: {}",
            robominer_domain::rejection_messages::update_robot_config_rejection_player_message(
                rejection
            )
        ))),
    }
}
