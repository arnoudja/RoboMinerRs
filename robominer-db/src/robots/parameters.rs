use super::{RequestedRobotParts, RobotParameters};

pub fn robot_parameters_for_parts(parts: &RequestedRobotParts) -> Option<RobotParameters> {
    let robot_parts = [
        &parts.ore_container,
        &parts.mining_unit,
        &parts.battery,
        &parts.memory_module,
        &parts.cpu,
        &parts.engine,
        &parts.ore_scanner,
    ];
    let battery_capacity: i32 = robot_parts.iter().map(|part| part.battery_capacity).sum();
    let power_usage: i32 = robot_parts.iter().map(|part| part.power_usage).sum();
    let weight: i32 = robot_parts.iter().map(|part| part.weight).sum();
    let volume: i32 = robot_parts.iter().map(|part| part.volume).sum();
    let forward_capacity: i32 = robot_parts.iter().map(|part| part.forward_capacity).sum();
    let backward_capacity: i32 = robot_parts.iter().map(|part| part.backward_capacity).sum();
    let rotate_capacity: i32 = robot_parts.iter().map(|part| part.rotate_capacity).sum();

    if power_usage == 0 || weight == 0 || volume < 0 {
        return None;
    }

    Some(RobotParameters {
        recharge_time: robot_parts.iter().map(|part| part.recharge_time).sum(),
        max_ore: robot_parts.iter().map(|part| part.ore_capacity).sum(),
        mining_speed: robot_parts.iter().map(|part| part.mining_capacity).sum(),
        max_turns: battery_capacity / power_usage,
        memory_size: robot_parts.iter().map(|part| part.memory_capacity).sum(),
        cpu_speed: robot_parts.iter().map(|part| part.cpu_capacity).sum(),
        forward_speed: 3.0 * f64::from(forward_capacity) / f64::from(weight),
        backward_speed: 3.0 * f64::from(backward_capacity) / f64::from(weight),
        rotate_speed: (20.0 * f64::from(rotate_capacity) / f64::from(weight)) as i32,
        robot_size: f64::from(volume).powf(0.33) / 2.0,
        scan_time: robot_parts.iter().map(|part| part.scan_time).sum(),
        scan_distance: robot_parts.iter().map(|part| part.scan_distance).sum(),
    })
}

pub(crate) async fn robot_is_recharging(
    transaction: &mut sqlx::Transaction<'_, sqlx::MySql>,
    robot_id: i64,
) -> Result<bool, sqlx::Error> {
    sqlx::query_scalar(
        "SELECT rechargeEndTime > NOW() \
                AND (miningEndTime IS NULL \
                     OR miningEndTime <= NOW() \
                     OR miningEndTime > rechargeEndTime) \
         FROM Robot \
         WHERE id = ?",
    )
    .bind(robot_id)
    .fetch_one(&mut **transaction)
    .await
}
