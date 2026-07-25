use crate::UpdateRobotConfigRequest;
use crate::shop::user_robot_part_total_owned;

pub(crate) async fn user_has_unassigned_parts_for_update(
    transaction: &mut sqlx::Transaction<'_, sqlx::MySql>,
    user_id: i64,
    robot_id: i64,
    baseline_parts: &[Option<i64>; 7],
    request: &UpdateRobotConfigRequest,
) -> Result<bool, sqlx::Error> {
    let mut simulated_parts = *baseline_parts;
    let requested_parts = [
        request.ore_container_id,
        request.mining_unit_id,
        request.battery_id,
        request.memory_module_id,
        request.cpu_id,
        request.engine_id,
        request.ore_scanner_id,
    ];

    for slot_index in 0..requested_parts.len() {
        let requested_part_id = requested_parts[slot_index];

        if simulated_parts[slot_index] == Some(requested_part_id) {
            continue;
        }

        if !user_has_unassigned_part_under_simulated_robot(
            transaction,
            user_id,
            robot_id,
            requested_part_id,
            &simulated_parts,
        )
        .await?
        {
            return Ok(false);
        }

        simulated_parts[slot_index] = Some(requested_part_id);
    }

    Ok(true)
}

async fn user_has_unassigned_part_under_simulated_robot(
    transaction: &mut sqlx::Transaction<'_, sqlx::MySql>,
    user_id: i64,
    robot_id: i64,
    robot_part_id: i64,
    simulated_parts: &[Option<i64>; 7],
) -> Result<bool, sqlx::Error> {
    let total_owned = user_robot_part_total_owned(transaction, user_id, robot_part_id).await?;
    let other_robot_usage =
        user_robot_part_usage_count_excluding_robot(transaction, user_id, robot_id, robot_part_id)
            .await?;
    let simulated_robot_usage = if simulated_parts.contains(&Some(robot_part_id)) {
        1
    } else {
        0
    };

    Ok(i64::from(total_owned) - other_robot_usage - simulated_robot_usage > 0)
}
pub(crate) async fn user_robot_part_usage_count_excluding_robot(
    transaction: &mut sqlx::Transaction<'_, sqlx::MySql>,
    user_id: i64,
    excluded_robot_id: i64,
    robot_part_id: i64,
) -> Result<i64, sqlx::Error> {
    sqlx::query_scalar(
        "SELECT COUNT(*) \
         FROM Robot \
         WHERE userId = ? \
           AND id <> ? \
           AND (Robot.oreContainerId = ? \
                OR Robot.miningUnitId = ? \
                OR Robot.batteryId = ? \
                OR Robot.memoryModuleId = ? \
                OR Robot.cpuId = ? \
                OR Robot.engineId = ? \
                OR Robot.oreScannerId = ? \
                OR EXISTS ( \
                    SELECT 1 \
                    FROM PendingRobotChanges \
                    WHERE PendingRobotChanges.robotId = Robot.id \
                      AND (PendingRobotChanges.oreContainerId = ? \
                           OR PendingRobotChanges.miningUnitId = ? \
                           OR PendingRobotChanges.batteryId = ? \
                           OR PendingRobotChanges.memoryModuleId = ? \
                           OR PendingRobotChanges.cpuId = ? \
                           OR PendingRobotChanges.engineId = ? \
                           OR PendingRobotChanges.oreScannerId = ?)))",
    )
    .bind(user_id)
    .bind(excluded_robot_id)
    .bind(robot_part_id)
    .bind(robot_part_id)
    .bind(robot_part_id)
    .bind(robot_part_id)
    .bind(robot_part_id)
    .bind(robot_part_id)
    .bind(robot_part_id)
    .bind(robot_part_id)
    .bind(robot_part_id)
    .bind(robot_part_id)
    .bind(robot_part_id)
    .bind(robot_part_id)
    .bind(robot_part_id)
    .bind(robot_part_id)
    .fetch_one(&mut **transaction)
    .await
}
