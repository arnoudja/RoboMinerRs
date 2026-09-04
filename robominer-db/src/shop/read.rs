use sqlx::MySqlPool;

use crate::ShopRobotPartStateRecord;

#[derive(sqlx::FromRow)]
struct ShopRobotPartStateRow {
    #[sqlx(rename = "id")]
    robot_part_id: i64,
    #[sqlx(rename = "totalOwned")]
    total_owned: i32,
    #[sqlx(rename = "assignedCount")]
    assigned_count: i64,
    #[sqlx(rename = "robotCount")]
    robot_count: i64,
    #[sqlx(rename = "canAfford")]
    can_afford: i32,
}

impl From<ShopRobotPartStateRow> for ShopRobotPartStateRecord {
    fn from(row: ShopRobotPartStateRow) -> Self {
        let assigned = row.assigned_count as i32;
        let unassigned = row.total_owned.saturating_sub(assigned);
        let can_sell = unassigned > 0;
        let can_buy = row.can_afford != 0 && row.robot_count > i64::from(row.total_owned);

        Self {
            robot_part_id: row.robot_part_id,
            total_owned: row.total_owned,
            assigned,
            unassigned,
            can_buy,
            can_sell,
        }
    }
}

pub async fn list_shop_robot_part_states(
    pool: &MySqlPool,
    user_id: i64,
) -> Result<Vec<ShopRobotPartStateRecord>, sqlx::Error> {
    sqlx::query_as::<_, ShopRobotPartStateRow>(
        "SELECT RobotPart.id AS id, \
                COALESCE(UserRobotPartAsset.totalOwned, 0) AS totalOwned, \
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
                        OR PendingRobotChanges.oreScannerId = RobotPart.id)) AS assignedCount, \
                (SELECT COUNT(*) FROM Robot WHERE Robot.userId = ?) AS robotCount, \
                CASE WHEN NOT EXISTS \
                    (SELECT 1 \
                     FROM OrePriceAmount \
                     LEFT JOIN UserOreAsset \
                       ON UserOreAsset.userId = ? \
                      AND UserOreAsset.oreId = OrePriceAmount.oreId \
                     WHERE OrePriceAmount.orePriceId = RobotPart.orePriceId \
                       AND COALESCE(UserOreAsset.amount, 0) < OrePriceAmount.amount) \
                    THEN 1 ELSE 0 END AS canAfford \
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
    .await
    .map(|rows| {
        rows.into_iter()
            .map(ShopRobotPartStateRecord::from)
            .collect()
    })
}
