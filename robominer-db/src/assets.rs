use sqlx::MySqlPool;

use crate::{
    INITIAL_ORE_WALLET_MAX, ShopRobotPartStateRecord, UserAssetSummaryRecord,
    UserOreAssetStateRecord,
};

#[derive(Debug, Clone, Copy)]
pub(crate) struct OrePriceCost {
    ore_id: i64,
    amount: i32,
}

pub async fn list_user_ore_asset_states(
    pool: &MySqlPool,
    user_id: i64,
) -> Result<Vec<UserOreAssetStateRecord>, sqlx::Error> {
    sqlx::query_as::<_, (i64, String, i32, i32, i32)>(
        "SELECT UserOreAsset.oreId, Ore.oreName, UserOreAsset.amount, UserOreAsset.maxAllowed, \
                UserOreAsset.depotMaxAllowed \
         FROM UserOreAsset \
         INNER JOIN Ore ON Ore.id = UserOreAsset.oreId \
         WHERE UserOreAsset.userId = ? \
         ORDER BY UserOreAsset.oreId DESC",
    )
    .bind(user_id)
    .fetch_all(pool)
    .await
    .map(|rows| {
        rows.into_iter()
            .map(
                |(ore_id, ore_name, amount, max_allowed, depot_max_allowed)| {
                    UserOreAssetStateRecord {
                        ore_id,
                        ore_name,
                        amount,
                        max_allowed,
                        depot_max_allowed,
                    }
                },
            )
            .collect()
    })
}

/// Per-ore depot capacity for a user (defaults to empty when no asset rows exist).
pub async fn list_user_depot_max_allowed(
    pool: &MySqlPool,
    user_id: i64,
) -> Result<Vec<(i64, i32)>, sqlx::Error> {
    sqlx::query_as::<_, (i64, i32)>(
        "SELECT oreId, depotMaxAllowed \
         FROM UserOreAsset \
         WHERE userId = ? \
         ORDER BY oreId",
    )
    .bind(user_id)
    .fetch_all(pool)
    .await
}

pub async fn load_user_asset_summary(
    pool: &MySqlPool,
    user_id: i64,
) -> Result<UserAssetSummaryRecord, sqlx::Error> {
    sqlx::query_as::<_, (String, i32, i32, i64)>(
        "SELECT User.username, User.achievementPoints, GREATEST(User.miningQueueSize, 1), \
                (SELECT COUNT(*) FROM Robot WHERE Robot.userId = User.id) \
         FROM User \
         WHERE User.id = ?",
    )
    .bind(user_id)
    .fetch_one(pool)
    .await
    .map(
        |(username, achievement_points, mining_queue_size, robot_count)| UserAssetSummaryRecord {
            username,
            achievement_points,
            mining_queue_size,
            robot_count,
        },
    )
}

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
pub(crate) async fn list_ore_price_amounts(
    transaction: &mut sqlx::Transaction<'_, sqlx::MySql>,
    ore_price_id: i64,
) -> Result<Vec<OrePriceCost>, sqlx::Error> {
    let rows = sqlx::query_as::<_, (i64, i32)>(
        "SELECT oreId, amount FROM OrePriceAmount WHERE orePriceId = ? ORDER BY oreId",
    )
    .bind(ore_price_id)
    .fetch_all(&mut **transaction)
    .await?;

    Ok(rows
        .into_iter()
        .map(|(ore_id, amount)| OrePriceCost { ore_id, amount })
        .collect())
}

pub(crate) async fn robot_part_ore_price_id(
    transaction: &mut sqlx::Transaction<'_, sqlx::MySql>,
    robot_part_id: i64,
) -> Result<Option<i64>, sqlx::Error> {
    sqlx::query_scalar("SELECT orePriceId FROM RobotPart WHERE id = ?")
        .bind(robot_part_id)
        .fetch_optional(&mut **transaction)
        .await
}

pub(crate) async fn can_pay_ore_costs(
    transaction: &mut sqlx::Transaction<'_, sqlx::MySql>,
    user_id: i64,
    costs: &[OrePriceCost],
) -> Result<bool, sqlx::Error> {
    for cost in costs {
        let amount: Option<i32> = sqlx::query_scalar(
            "SELECT amount \
             FROM UserOreAsset \
             WHERE userId = ? AND oreId = ? \
             FOR UPDATE",
        )
        .bind(user_id)
        .bind(cost.ore_id)
        .fetch_optional(&mut **transaction)
        .await?;

        if amount.unwrap_or_default() < cost.amount {
            return Ok(false);
        }
    }

    Ok(true)
}

pub(crate) async fn deduct_ore_costs(
    transaction: &mut sqlx::Transaction<'_, sqlx::MySql>,
    user_id: i64,
    costs: &[OrePriceCost],
) -> Result<(), sqlx::Error> {
    for cost in costs {
        sqlx::query(
            "UPDATE UserOreAsset \
             SET amount = amount - ? \
             WHERE userId = ? AND oreId = ?",
        )
        .bind(cost.amount)
        .bind(user_id)
        .bind(cost.ore_id)
        .execute(&mut **transaction)
        .await?;
    }

    Ok(())
}

pub(crate) async fn refund_half_ore_costs(
    transaction: &mut sqlx::Transaction<'_, sqlx::MySql>,
    user_id: i64,
    costs: &[OrePriceCost],
) -> Result<(), sqlx::Error> {
    for cost in costs {
        let refund = cost.amount / 2;
        let asset: Option<(i32, i32)> = sqlx::query_as(
            "SELECT amount, maxAllowed \
             FROM UserOreAsset \
             WHERE userId = ? AND oreId = ? \
             FOR UPDATE",
        )
        .bind(user_id)
        .bind(cost.ore_id)
        .fetch_optional(&mut **transaction)
        .await?;

        if let Some((amount, max_allowed)) = asset {
            let new_amount = (amount + refund).min(max_allowed);
            sqlx::query(
                "UPDATE UserOreAsset \
                 SET amount = ? \
                 WHERE userId = ? AND oreId = ?",
            )
            .bind(new_amount)
            .bind(user_id)
            .bind(cost.ore_id)
            .execute(&mut **transaction)
            .await?;
        } else {
            sqlx::query(
                "INSERT INTO UserOreAsset (userId, oreId, amount, maxAllowed) \
                 VALUES (?, ?, LEAST(?, ?), ?)",
            )
            .bind(user_id)
            .bind(cost.ore_id)
            .bind(refund)
            .bind(INITIAL_ORE_WALLET_MAX)
            .bind(INITIAL_ORE_WALLET_MAX)
            .execute(&mut **transaction)
            .await?;
        }
    }

    Ok(())
}
