#[derive(sqlx::FromRow)]
struct RobotPartSlotsRow {
    robot_id: i64,
    ore_container_id: Option<i64>,
    mining_unit_id: Option<i64>,
    battery_id: Option<i64>,
    memory_module_id: Option<i64>,
    cpu_id: Option<i64>,
    engine_id: Option<i64>,
    ore_scanner_id: Option<i64>,
}

fn part_slot_values(row: &RobotPartSlotsRow) -> impl Iterator<Item = i64> + '_ {
    [
        row.ore_container_id,
        row.mining_unit_id,
        row.battery_id,
        row.memory_module_id,
        row.cpu_id,
        row.engine_id,
        row.ore_scanner_id,
    ]
    .into_iter()
    .flatten()
}

pub(crate) fn unassigned_robot_part_count(total_owned: i32, usage_count: i64) -> i32 {
    i64::from(total_owned)
        .saturating_sub(usage_count)
        .clamp(0, i64::from(i32::MAX)) as i32
}

pub(crate) async fn add_user_robot_part_asset(
    transaction: &mut sqlx::Transaction<'_, sqlx::MySql>,
    user_id: i64,
    robot_part_id: i64,
) -> Result<(), sqlx::Error> {
    sqlx::query(
        "INSERT INTO UserRobotPartAsset (userId, robotPartId, totalOwned) \
         VALUES (?, ?, 1) \
         ON DUPLICATE KEY UPDATE totalOwned = totalOwned + 1",
    )
    .bind(user_id)
    .bind(robot_part_id)
    .execute(&mut **transaction)
    .await?;

    Ok(())
}

pub(crate) async fn user_robot_part_total_owned(
    transaction: &mut sqlx::Transaction<'_, sqlx::MySql>,
    user_id: i64,
    robot_part_id: i64,
) -> Result<i32, sqlx::Error> {
    let total_owned: Option<i32> = sqlx::query_scalar(
        "SELECT totalOwned \
         FROM UserRobotPartAsset \
         WHERE userId = ? AND robotPartId = ? \
         FOR UPDATE",
    )
    .bind(user_id)
    .bind(robot_part_id)
    .fetch_optional(&mut **transaction)
    .await?;

    Ok(total_owned.unwrap_or_default())
}

fn robot_part_usage_counts_from_slot_rows(
    robot_rows: &[RobotPartSlotsRow],
    pending_rows: &[RobotPartSlotsRow],
) -> std::collections::HashMap<i64, i64> {
    use std::collections::HashMap;

    let pending_by_robot: HashMap<i64, &RobotPartSlotsRow> =
        pending_rows.iter().map(|row| (row.robot_id, row)).collect();

    let mut usage_by_part = HashMap::new();
    for row in robot_rows {
        let pending = pending_by_robot.get(&row.robot_id);
        let mut seen_for_robot = std::collections::HashSet::new();
        for part_id in part_slot_values(row) {
            seen_for_robot.insert(part_id);
        }
        if let Some(pending_row) = pending {
            for part_id in part_slot_values(pending_row) {
                seen_for_robot.insert(part_id);
            }
        }
        for part_id in seen_for_robot {
            *usage_by_part.entry(part_id).or_default() += 1;
        }
    }

    usage_by_part
}

pub(crate) async fn user_robot_part_usage_counts_for_user(
    transaction: &mut sqlx::Transaction<'_, sqlx::MySql>,
    user_id: i64,
) -> Result<std::collections::HashMap<i64, i64>, sqlx::Error> {
    let robot_rows = sqlx::query_as::<_, RobotPartSlotsRow>(
        "SELECT id AS robot_id, oreContainerId AS ore_container_id, miningUnitId AS mining_unit_id, \
                batteryId AS battery_id, memoryModuleId AS memory_module_id, cpuId AS cpu_id, \
                engineId AS engine_id, oreScannerId AS ore_scanner_id \
         FROM Robot \
         WHERE userId = ?",
    )
    .bind(user_id)
    .fetch_all(&mut **transaction)
    .await?;

    let pending_rows = sqlx::query_as::<_, RobotPartSlotsRow>(
        "SELECT PendingRobotChanges.robotId AS robot_id, \
                PendingRobotChanges.oreContainerId AS ore_container_id, \
                PendingRobotChanges.miningUnitId AS mining_unit_id, \
                PendingRobotChanges.batteryId AS battery_id, \
                PendingRobotChanges.memoryModuleId AS memory_module_id, \
                PendingRobotChanges.cpuId AS cpu_id, \
                PendingRobotChanges.engineId AS engine_id, \
                PendingRobotChanges.oreScannerId AS ore_scanner_id \
         FROM PendingRobotChanges \
         INNER JOIN Robot ON Robot.id = PendingRobotChanges.robotId \
         WHERE Robot.userId = ?",
    )
    .bind(user_id)
    .fetch_all(&mut **transaction)
    .await?;

    Ok(robot_part_usage_counts_from_slot_rows(
        &robot_rows,
        &pending_rows,
    ))
}

pub(crate) async fn user_robot_part_usage_count(
    transaction: &mut sqlx::Transaction<'_, sqlx::MySql>,
    user_id: i64,
    robot_part_id: i64,
) -> Result<i64, sqlx::Error> {
    sqlx::query_scalar(
        "SELECT COUNT(*) \
         FROM Robot \
         WHERE userId = ? \
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

pub(crate) async fn remove_user_robot_part_assets(
    transaction: &mut sqlx::Transaction<'_, sqlx::MySql>,
    user_id: i64,
    robot_part_id: i64,
    count: i32,
) -> Result<(), sqlx::Error> {
    sqlx::query(
        "UPDATE UserRobotPartAsset \
         SET totalOwned = totalOwned - ? \
         WHERE userId = ? AND robotPartId = ?",
    )
    .bind(count)
    .bind(user_id)
    .bind(robot_part_id)
    .execute(&mut **transaction)
    .await?;

    Ok(())
}

pub(crate) async fn delete_zero_owned_robot_part_assets(
    transaction: &mut sqlx::Transaction<'_, sqlx::MySql>,
    user_id: i64,
) -> Result<(), sqlx::Error> {
    sqlx::query("DELETE FROM UserRobotPartAsset WHERE userId = ? AND totalOwned = 0")
        .bind(user_id)
        .execute(&mut **transaction)
        .await?;

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::{
        RobotPartSlotsRow, robot_part_usage_counts_from_slot_rows, unassigned_robot_part_count,
    };

    #[test]
    fn unassigned_robot_part_count_never_goes_negative() {
        assert_eq!(unassigned_robot_part_count(3, 1), 2);
        assert_eq!(unassigned_robot_part_count(1, 1), 0);
        assert_eq!(unassigned_robot_part_count(0, 5), 0);
    }

    #[test]
    fn robot_part_usage_counts_installed_and_pending_slots() {
        let robot_rows = vec![
            RobotPartSlotsRow {
                robot_id: 1,
                ore_container_id: Some(10),
                mining_unit_id: None,
                battery_id: Some(20),
                memory_module_id: None,
                cpu_id: None,
                engine_id: None,
                ore_scanner_id: None,
            },
            RobotPartSlotsRow {
                robot_id: 2,
                ore_container_id: Some(10),
                mining_unit_id: None,
                battery_id: None,
                memory_module_id: None,
                cpu_id: None,
                engine_id: None,
                ore_scanner_id: None,
            },
        ];
        let pending_rows = vec![RobotPartSlotsRow {
            robot_id: 1,
            ore_container_id: None,
            mining_unit_id: Some(30),
            battery_id: None,
            memory_module_id: None,
            cpu_id: None,
            engine_id: None,
            ore_scanner_id: None,
        }];

        let usage = robot_part_usage_counts_from_slot_rows(&robot_rows, &pending_rows);
        assert_eq!(usage.get(&10), Some(&2));
        assert_eq!(usage.get(&20), Some(&1));
        assert_eq!(usage.get(&30), Some(&1));
        assert_eq!(usage.get(&99), None);
    }
}
