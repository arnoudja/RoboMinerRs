use sqlx::MySqlPool;

use crate::assets::{
    can_pay_ore_costs, deduct_ore_costs, list_ore_price_amounts, refund_half_ore_costs,
    robot_part_ore_price_id,
};
use crate::users::{touch_user_last_login_time, user_exists};
use crate::{
    RobotPartTransaction, RobotPartTransactionRejection, RobotPartTransactionRequest,
    SellAllUnassignedRobotPartsResult,
};

pub async fn buy_robot_part(
    pool: &MySqlPool,
    request: RobotPartTransactionRequest,
) -> Result<Result<RobotPartTransaction, RobotPartTransactionRejection>, sqlx::Error> {
    let mut transaction = pool.begin().await?;

    if !user_exists(&mut transaction, request.user_id).await? {
        transaction.rollback().await?;
        return Ok(Err(RobotPartTransactionRejection::UnknownUser));
    }

    let Some(ore_price_id) =
        robot_part_ore_price_id(&mut transaction, request.robot_part_id).await?
    else {
        transaction.rollback().await?;
        return Ok(Err(RobotPartTransactionRejection::UnknownRobotPart));
    };

    let costs = list_ore_price_amounts(&mut transaction, ore_price_id).await?;
    if !can_pay_ore_costs(&mut transaction, request.user_id, &costs).await? {
        transaction.rollback().await?;
        return Ok(Err(RobotPartTransactionRejection::InsufficientFunds));
    }

    deduct_ore_costs(&mut transaction, request.user_id, &costs).await?;
    add_user_robot_part_asset(&mut transaction, request.user_id, request.robot_part_id).await?;

    touch_user_last_login_time(&mut transaction, request.user_id).await?;

    transaction.commit().await?;

    Ok(Ok(RobotPartTransaction {
        robot_part_id: request.robot_part_id,
    }))
}

pub async fn sell_robot_part(
    pool: &MySqlPool,
    request: RobotPartTransactionRequest,
) -> Result<Result<RobotPartTransaction, RobotPartTransactionRejection>, sqlx::Error> {
    let mut transaction = pool.begin().await?;

    if !user_exists(&mut transaction, request.user_id).await? {
        transaction.rollback().await?;
        return Ok(Err(RobotPartTransactionRejection::UnknownUser));
    }

    match sell_one_unassigned_robot_part_in_transaction(
        &mut transaction,
        request.user_id,
        request.robot_part_id,
    )
    .await?
    {
        Ok(()) => {}
        Err(rejection) => {
            transaction.rollback().await?;
            return Ok(Err(rejection));
        }
    }

    delete_zero_owned_robot_part_assets(&mut transaction, request.user_id).await?;

    touch_user_last_login_time(&mut transaction, request.user_id).await?;

    transaction.commit().await?;

    Ok(Ok(RobotPartTransaction {
        robot_part_id: request.robot_part_id,
    }))
}

pub async fn sell_all_unassigned_robot_parts(
    pool: &MySqlPool,
    user_id: i64,
) -> Result<Result<SellAllUnassignedRobotPartsResult, RobotPartTransactionRejection>, sqlx::Error> {
    let mut transaction = pool.begin().await?;

    if !user_exists(&mut transaction, user_id).await? {
        transaction.rollback().await?;
        return Ok(Err(RobotPartTransactionRejection::UnknownUser));
    }

    let sellable_parts = list_user_sellable_robot_part_counts(&mut transaction, user_id).await?;
    let mut sold_count = 0;

    for (robot_part_id, unassigned) in sellable_parts {
        for _ in 0..unassigned {
            match sell_one_unassigned_robot_part_in_transaction(
                &mut transaction,
                user_id,
                robot_part_id,
            )
            .await?
            {
                Ok(()) => sold_count += 1,
                Err(rejection) => {
                    transaction.rollback().await?;
                    return Ok(Err(rejection));
                }
            }
        }
    }

    if sold_count == 0 {
        transaction.rollback().await?;
        return Ok(Err(RobotPartTransactionRejection::NoUnassignedRobotPart));
    }

    delete_zero_owned_robot_part_assets(&mut transaction, user_id).await?;

    touch_user_last_login_time(&mut transaction, user_id).await?;

    transaction.commit().await?;

    Ok(Ok(SellAllUnassignedRobotPartsResult { sold_count }))
}

async fn sell_one_unassigned_robot_part_in_transaction(
    transaction: &mut sqlx::Transaction<'_, sqlx::MySql>,
    user_id: i64,
    robot_part_id: i64,
) -> Result<Result<(), RobotPartTransactionRejection>, sqlx::Error> {
    let Some(ore_price_id) = robot_part_ore_price_id(transaction, robot_part_id).await? else {
        return Ok(Err(RobotPartTransactionRejection::UnknownRobotPart));
    };

    let total_owned = user_robot_part_total_owned(transaction, user_id, robot_part_id).await?;
    let usage_count = user_robot_part_usage_count(transaction, user_id, robot_part_id).await?;

    if i64::from(total_owned) - usage_count < 1 {
        return Ok(Err(RobotPartTransactionRejection::NoUnassignedRobotPart));
    }

    remove_user_robot_part_asset(transaction, user_id, robot_part_id).await?;

    let costs = list_ore_price_amounts(transaction, ore_price_id).await?;
    refund_half_ore_costs(transaction, user_id, &costs).await?;

    Ok(Ok(()))
}

async fn list_user_sellable_robot_part_counts(
    transaction: &mut sqlx::Transaction<'_, sqlx::MySql>,
    user_id: i64,
) -> Result<Vec<(i64, i32)>, sqlx::Error> {
    let rows = sqlx::query_as::<_, (i64, i32)>(
        "SELECT robotPartId, totalOwned \
         FROM UserRobotPartAsset \
         WHERE userId = ? AND totalOwned > 0 \
         FOR UPDATE",
    )
    .bind(user_id)
    .fetch_all(&mut **transaction)
    .await?;

    let mut sellable_parts = Vec::new();
    for (robot_part_id, total_owned) in rows {
        let usage_count = user_robot_part_usage_count(transaction, user_id, robot_part_id).await?;
        let unassigned = unassigned_robot_part_count(total_owned, usage_count);
        if unassigned > 0 {
            sellable_parts.push((robot_part_id, unassigned));
        }
    }

    Ok(sellable_parts)
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
