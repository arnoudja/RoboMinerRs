use sqlx::MySqlPool;

use crate::ShopRobotPartStateRecord;

pub async fn list_shop_robot_part_states(
    pool: &MySqlPool,
    user_id: i64,
) -> Result<Vec<ShopRobotPartStateRecord>, sqlx::Error> {
    let rows = sqlx::query_as::<_, (i64, i32, i64, i64, i32)>(
        "SELECT RobotPart.id, \
                COALESCE(UserRobotPartAsset.totalOwned, 0), \
                (SELECT COUNT(*) \
                 FROM Robot \
                 LEFT JOIN PendingRobotChanges ON PendingRobotChanges.robotId = Robot.id \
                 WHERE Robot.userId = ? \
                   AND (Robot.oreContainerId = RobotPart.id \
                        OR Robot.miningUnitId = RobotPart.id \
                        OR Robot.batteryId = RobotPart.id \
                        OR Robot.memoryModuleId = RobotPart.id \
                        OR Robot.cpuId = RobotPart.id \
                        OR Robot.engineId = RobotPart.id \
                        OR Robot.oreScannerId = RobotPart.id \
                        OR PendingRobotChanges.oreContainerId = RobotPart.id \
                        OR PendingRobotChanges.miningUnitId = RobotPart.id \
                        OR PendingRobotChanges.batteryId = RobotPart.id \
                        OR PendingRobotChanges.memoryModuleId = RobotPart.id \
                        OR PendingRobotChanges.cpuId = RobotPart.id \
                        OR PendingRobotChanges.engineId = RobotPart.id \
                        OR PendingRobotChanges.oreScannerId = RobotPart.id)), \
                (SELECT COUNT(*) FROM Robot WHERE Robot.userId = ?), \
                CASE WHEN NOT EXISTS \
                    (SELECT 1 \
                     FROM OrePriceAmount \
                     LEFT JOIN UserOreAsset \
                       ON UserOreAsset.userId = ? \
                      AND UserOreAsset.oreId = OrePriceAmount.oreId \
                     WHERE OrePriceAmount.orePriceId = RobotPart.orePriceId \
                       AND COALESCE(UserOreAsset.amount, 0) < OrePriceAmount.amount) \
                    THEN 1 ELSE 0 END \
         FROM RobotPart \
         LEFT JOIN UserRobotPartAsset \
           ON UserRobotPartAsset.robotPartId = RobotPart.id \
          AND UserRobotPartAsset.userId = ? \
         ORDER BY RobotPart.typeId, RobotPart.id",
    )
    .bind(user_id)
    .bind(user_id)
    .bind(user_id)
    .bind(user_id)
    .fetch_all(pool)
    .await?;

    Ok(rows
        .into_iter()
        .map(
            |(robot_part_id, total_owned, assigned_count, robot_count, can_afford)| {
                let assigned = assigned_count as i32;
                let unassigned = total_owned.saturating_sub(assigned);
                let can_sell = unassigned > 0;
                let can_buy = can_afford != 0 && robot_count > i64::from(total_owned);

                ShopRobotPartStateRecord {
                    robot_part_id,
                    total_owned,
                    assigned,
                    unassigned,
                    can_buy,
                    can_sell,
                }
            },
        )
        .collect())
}
