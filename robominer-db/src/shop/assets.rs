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

pub(crate) async fn remove_user_robot_part_asset(
    transaction: &mut sqlx::Transaction<'_, sqlx::MySql>,
    user_id: i64,
    robot_part_id: i64,
) -> Result<(), sqlx::Error> {
    sqlx::query(
        "UPDATE UserRobotPartAsset \
         SET totalOwned = totalOwned - 1 \
         WHERE userId = ? AND robotPartId = ?",
    )
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
    use super::unassigned_robot_part_count;

    #[test]
    fn unassigned_robot_part_count_never_goes_negative() {
        assert_eq!(unassigned_robot_part_count(3, 1), 2);
        assert_eq!(unassigned_robot_part_count(1, 1), 0);
        assert_eq!(unassigned_robot_part_count(0, 5), 0);
    }
}
