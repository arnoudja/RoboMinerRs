use super::super::{RobotParameters, RobotUpdateState};
use crate::UpdateRobotConfigRequest;

pub(super) async fn update_pending_robot_config(
    transaction: &mut sqlx::Transaction<'_, sqlx::MySql>,
    request: &UpdateRobotConfigRequest,
    source_code: &str,
    parameters: &RobotParameters,
) -> Result<(), sqlx::Error> {
    sqlx::query(
        "UPDATE PendingRobotChanges \
         SET sourceCode = ?, \
             oreContainerId = ?, \
             miningUnitId = ?, \
             batteryId = ?, \
             memoryModuleId = ?, \
             cpuId = ?, \
             engineId = ?, \
             oreScannerId = ?, \
             rechargeTime = ?, \
             maxOre = ?, \
             miningSpeed = ?, \
             maxTurns = ?, \
             memorySize = ?, \
             cpuSpeed = ?, \
             forwardSpeed = ?, \
             backwardSpeed = ?, \
             rotateSpeed = ?, \
             robotSize = ?, \
             scanTime = ?, \
             scanDistance = ? \
         WHERE robotId = ?",
    )
    .bind(source_code)
    .bind(request.ore_container_id)
    .bind(request.mining_unit_id)
    .bind(request.battery_id)
    .bind(request.memory_module_id)
    .bind(request.cpu_id)
    .bind(request.engine_id)
    .bind(request.ore_scanner_id)
    .bind(parameters.recharge_time)
    .bind(parameters.max_ore)
    .bind(parameters.mining_speed)
    .bind(parameters.max_turns)
    .bind(parameters.memory_size)
    .bind(parameters.cpu_speed)
    .bind(parameters.forward_speed)
    .bind(parameters.backward_speed)
    .bind(parameters.rotate_speed)
    .bind(parameters.robot_size)
    .bind(parameters.scan_time)
    .bind(parameters.scan_distance)
    .bind(request.robot_id)
    .execute(&mut **transaction)
    .await?;

    Ok(())
}

pub(super) async fn delete_pending_robot_config(
    transaction: &mut sqlx::Transaction<'_, sqlx::MySql>,
    robot_id: i64,
) -> Result<(), sqlx::Error> {
    sqlx::query("DELETE FROM PendingRobotChanges WHERE robotId = ?")
        .bind(robot_id)
        .execute(&mut **transaction)
        .await?;

    Ok(())
}

pub(super) async fn insert_pending_robot_config(
    transaction: &mut sqlx::Transaction<'_, sqlx::MySql>,
    robot: &RobotUpdateState,
    request: &UpdateRobotConfigRequest,
    source_code: &str,
    parameters: &RobotParameters,
) -> Result<(), sqlx::Error> {
    sqlx::query(
        "INSERT INTO PendingRobotChanges \
         (robotId, sourceCode, oreContainerId, miningUnitId, batteryId, memoryModuleId, \
          cpuId, engineId, oreScannerId, oldOreContainerId, oldMiningUnitId, oldBatteryId, \
          oldMemoryModuleId, oldCpuId, oldEngineId, oldOreScannerId, rechargeTime, maxOre, \
          miningSpeed, maxTurns, memorySize, cpuSpeed, forwardSpeed, backwardSpeed, rotateSpeed, \
          robotSize, scanTime, scanDistance, changesCommitTime) \
         VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, NULL)",
    )
    .bind(request.robot_id)
    .bind(source_code)
    .bind(request.ore_container_id)
    .bind(request.mining_unit_id)
    .bind(request.battery_id)
    .bind(request.memory_module_id)
    .bind(request.cpu_id)
    .bind(request.engine_id)
    .bind(request.ore_scanner_id)
    .bind(robot.ore_container_id)
    .bind(robot.mining_unit_id)
    .bind(robot.battery_id)
    .bind(robot.memory_module_id)
    .bind(robot.cpu_id)
    .bind(robot.engine_id)
    .bind(robot.ore_scanner_id)
    .bind(parameters.recharge_time)
    .bind(parameters.max_ore)
    .bind(parameters.mining_speed)
    .bind(parameters.max_turns)
    .bind(parameters.memory_size)
    .bind(parameters.cpu_speed)
    .bind(parameters.forward_speed)
    .bind(parameters.backward_speed)
    .bind(parameters.rotate_speed)
    .bind(parameters.robot_size)
    .bind(parameters.scan_time)
    .bind(parameters.scan_distance)
    .execute(&mut **transaction)
    .await?;

    Ok(())
}

pub(super) async fn update_robot_header(
    transaction: &mut sqlx::Transaction<'_, sqlx::MySql>,
    request: &UpdateRobotConfigRequest,
) -> Result<(), sqlx::Error> {
    sqlx::query(
        "UPDATE Robot \
         SET robotName = ?, programSourceId = ? \
         WHERE id = ? AND userId = ?",
    )
    .bind(&request.robot_name)
    .bind(request.program_source_id)
    .bind(request.robot_id)
    .bind(request.user_id)
    .execute(&mut **transaction)
    .await?;

    Ok(())
}

pub(super) async fn update_robot_config_immediately(
    transaction: &mut sqlx::Transaction<'_, sqlx::MySql>,
    request: &UpdateRobotConfigRequest,
    source_code: &str,
    parameters: &RobotParameters,
) -> Result<(), sqlx::Error> {
    sqlx::query(
        "UPDATE Robot \
         SET robotName = ?, programSourceId = ?, sourceCode = ?, oreContainerId = ?, \
             miningUnitId = ?, batteryId = ?, memoryModuleId = ?, cpuId = ?, engineId = ?, \
             oreScannerId = ?, rechargeTime = ?, maxOre = ?, miningSpeed = ?, maxTurns = ?, \
             memorySize = ?, cpuSpeed = ?, forwardSpeed = ?, backwardSpeed = ?, rotateSpeed = ?, \
             robotSize = ?, scanTime = ?, scanDistance = ? \
         WHERE id = ? AND userId = ?",
    )
    .bind(&request.robot_name)
    .bind(request.program_source_id)
    .bind(source_code)
    .bind(request.ore_container_id)
    .bind(request.mining_unit_id)
    .bind(request.battery_id)
    .bind(request.memory_module_id)
    .bind(request.cpu_id)
    .bind(request.engine_id)
    .bind(request.ore_scanner_id)
    .bind(parameters.recharge_time)
    .bind(parameters.max_ore)
    .bind(parameters.mining_speed)
    .bind(parameters.max_turns)
    .bind(parameters.memory_size)
    .bind(parameters.cpu_speed)
    .bind(parameters.forward_speed)
    .bind(parameters.backward_speed)
    .bind(parameters.rotate_speed)
    .bind(parameters.robot_size)
    .bind(parameters.scan_time)
    .bind(parameters.scan_distance)
    .bind(request.robot_id)
    .bind(request.user_id)
    .execute(&mut **transaction)
    .await?;

    Ok(())
}
