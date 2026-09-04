use super::super::{RobotParameters, RobotUpdateState};
use crate::UpdateRobotConfigRequest;

pub(super) async fn update_pending_robot_config(
    transaction: &mut sqlx::Transaction<'_, sqlx::MySql>,
    request: &UpdateRobotConfigRequest,
    source_code: &str,
    parameters: &RobotParameters,
) -> Result<(), sqlx::Error> {
    sqlx::query!(
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
        source_code,
        request.ore_container_id,
        request.mining_unit_id,
        request.battery_id,
        request.memory_module_id,
        request.cpu_id,
        request.engine_id,
        request.ore_scanner_id,
        parameters.recharge_time,
        parameters.max_ore,
        parameters.mining_speed,
        parameters.max_turns,
        parameters.memory_size,
        parameters.cpu_speed,
        parameters.forward_speed,
        parameters.backward_speed,
        parameters.rotate_speed,
        parameters.robot_size,
        parameters.scan_time,
        parameters.scan_distance,
        request.robot_id
    )
    .execute(&mut **transaction)
    .await?;

    Ok(())
}

pub(super) async fn delete_pending_robot_config(
    transaction: &mut sqlx::Transaction<'_, sqlx::MySql>,
    robot_id: i64,
) -> Result<(), sqlx::Error> {
    sqlx::query!(
        "DELETE FROM PendingRobotChanges WHERE robotId = ?",
        robot_id
    )
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
    sqlx::query!(
        "INSERT INTO PendingRobotChanges \
         (robotId, sourceCode, oreContainerId, miningUnitId, batteryId, memoryModuleId, \
          cpuId, engineId, oreScannerId, oldOreContainerId, oldMiningUnitId, oldBatteryId, \
          oldMemoryModuleId, oldCpuId, oldEngineId, oldOreScannerId, rechargeTime, maxOre, \
          miningSpeed, maxTurns, memorySize, cpuSpeed, forwardSpeed, backwardSpeed, rotateSpeed, \
          robotSize, scanTime, scanDistance, changesCommitTime) \
         VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, NULL)",
        request.robot_id,
        source_code,
        request.ore_container_id,
        request.mining_unit_id,
        request.battery_id,
        request.memory_module_id,
        request.cpu_id,
        request.engine_id,
        request.ore_scanner_id,
        robot.ore_container_id,
        robot.mining_unit_id,
        robot.battery_id,
        robot.memory_module_id,
        robot.cpu_id,
        robot.engine_id,
        robot.ore_scanner_id,
        parameters.recharge_time,
        parameters.max_ore,
        parameters.mining_speed,
        parameters.max_turns,
        parameters.memory_size,
        parameters.cpu_speed,
        parameters.forward_speed,
        parameters.backward_speed,
        parameters.rotate_speed,
        parameters.robot_size,
        parameters.scan_time,
        parameters.scan_distance
    )
    .execute(&mut **transaction)
    .await?;

    Ok(())
}

pub(super) async fn update_robot_header(
    transaction: &mut sqlx::Transaction<'_, sqlx::MySql>,
    request: &UpdateRobotConfigRequest,
) -> Result<(), sqlx::Error> {
    sqlx::query!(
        "UPDATE Robot \
         SET robotName = ?, programSourceId = ? \
         WHERE id = ? AND userId = ?",
        request.robot_name,
        request.program_source_id,
        request.robot_id,
        request.user_id
    )
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
    sqlx::query!(
        "UPDATE Robot \
         SET robotName = ?, programSourceId = ?, sourceCode = ?, oreContainerId = ?, \
             miningUnitId = ?, batteryId = ?, memoryModuleId = ?, cpuId = ?, engineId = ?, \
             oreScannerId = ?, rechargeTime = ?, maxOre = ?, miningSpeed = ?, maxTurns = ?, \
             memorySize = ?, cpuSpeed = ?, forwardSpeed = ?, backwardSpeed = ?, rotateSpeed = ?, \
             robotSize = ?, scanTime = ?, scanDistance = ? \
         WHERE id = ? AND userId = ?",
        request.robot_name,
        request.program_source_id,
        source_code,
        request.ore_container_id,
        request.mining_unit_id,
        request.battery_id,
        request.memory_module_id,
        request.cpu_id,
        request.engine_id,
        request.ore_scanner_id,
        parameters.recharge_time,
        parameters.max_ore,
        parameters.mining_speed,
        parameters.max_turns,
        parameters.memory_size,
        parameters.cpu_speed,
        parameters.forward_speed,
        parameters.backward_speed,
        parameters.rotate_speed,
        parameters.robot_size,
        parameters.scan_time,
        parameters.scan_distance,
        request.robot_id,
        request.user_id
    )
    .execute(&mut **transaction)
    .await?;

    Ok(())
}
