use sqlx::MySqlPool;

use super::assets::{
    add_user_robot_part_asset, delete_zero_owned_robot_part_assets, remove_user_robot_part_asset,
    unassigned_robot_part_count, user_robot_part_total_owned, user_robot_part_usage_count,
};
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
