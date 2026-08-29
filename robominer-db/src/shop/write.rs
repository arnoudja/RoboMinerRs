use sqlx::MySqlPool;

use super::assets::{
    add_user_robot_part_asset, delete_zero_owned_robot_part_assets, remove_user_robot_part_assets,
    unassigned_robot_part_count, user_robot_part_total_owned, user_robot_part_usage_count,
    user_robot_part_usage_counts_for_user,
};
use crate::assets::{
    can_pay_ore_costs, deduct_ore_costs, list_ore_price_amounts, refund_half_ore_costs_scaled,
    robot_part_ore_price_id,
};
use crate::users::{touch_user_last_login_time, user_exists};
use crate::{
    DbOutcome, RobotPartTransaction, RobotPartTransactionRejection, RobotPartTransactionRequest,
    SellAllUnassignedRobotPartsResult, db_ok, db_reject,
};

pub async fn buy_robot_part(
    pool: &MySqlPool,
    request: RobotPartTransactionRequest,
) -> Result<DbOutcome<RobotPartTransaction, RobotPartTransactionRejection>, sqlx::Error> {
    let mut transaction = pool.begin().await?;

    if !user_exists(&mut transaction, request.user_id).await? {
        transaction.rollback().await?;
        return db_reject(RobotPartTransactionRejection::UnknownUser);
    }

    let Some(ore_price_id) =
        robot_part_ore_price_id(&mut transaction, request.robot_part_id).await?
    else {
        transaction.rollback().await?;
        return db_reject(RobotPartTransactionRejection::UnknownRobotPart);
    };

    let costs = list_ore_price_amounts(&mut transaction, ore_price_id).await?;
    if !can_pay_ore_costs(&mut transaction, request.user_id, &costs).await? {
        transaction.rollback().await?;
        return db_reject(RobotPartTransactionRejection::InsufficientFunds);
    }

    deduct_ore_costs(&mut transaction, request.user_id, &costs).await?;
    add_user_robot_part_asset(&mut transaction, request.user_id, request.robot_part_id).await?;

    touch_user_last_login_time(&mut transaction, request.user_id).await?;

    transaction.commit().await?;

    db_ok(RobotPartTransaction {
        robot_part_id: request.robot_part_id,
    })
}

pub async fn sell_robot_part(
    pool: &MySqlPool,
    request: RobotPartTransactionRequest,
) -> Result<DbOutcome<RobotPartTransaction, RobotPartTransactionRejection>, sqlx::Error> {
    let mut transaction = pool.begin().await?;

    if !user_exists(&mut transaction, request.user_id).await? {
        transaction.rollback().await?;
        return db_reject(RobotPartTransactionRejection::UnknownUser);
    }

    match sell_one_unassigned_robot_part_in_transaction(
        &mut transaction,
        request.user_id,
        request.robot_part_id,
    )
    .await?
    {
        DbOutcome::Success(()) => {}
        DbOutcome::Rejected(rejection) => {
            transaction.rollback().await?;
            return db_reject(rejection);
        }
    }

    delete_zero_owned_robot_part_assets(&mut transaction, request.user_id).await?;

    touch_user_last_login_time(&mut transaction, request.user_id).await?;

    transaction.commit().await?;

    db_ok(RobotPartTransaction {
        robot_part_id: request.robot_part_id,
    })
}

pub async fn sell_all_unassigned_robot_parts(
    pool: &MySqlPool,
    user_id: i64,
) -> Result<DbOutcome<SellAllUnassignedRobotPartsResult, RobotPartTransactionRejection>, sqlx::Error>
{
    let mut transaction = pool.begin().await?;

    if !user_exists(&mut transaction, user_id).await? {
        transaction.rollback().await?;
        return db_reject(RobotPartTransactionRejection::UnknownUser);
    }

    let sellable_parts = list_user_sellable_robot_part_counts(&mut transaction, user_id).await?;
    let mut sold_count = 0;

    for (robot_part_id, unassigned) in sellable_parts {
        match sell_unassigned_robot_parts_counted(
            &mut transaction,
            user_id,
            robot_part_id,
            unassigned,
        )
        .await?
        {
            DbOutcome::Success(sold) => sold_count += sold,
            DbOutcome::Rejected(rejection) => {
                transaction.rollback().await?;
                return db_reject(rejection);
            }
        }
    }

    if sold_count == 0 {
        transaction.rollback().await?;
        return db_reject(RobotPartTransactionRejection::NoUnassignedRobotPart);
    }

    delete_zero_owned_robot_part_assets(&mut transaction, user_id).await?;

    touch_user_last_login_time(&mut transaction, user_id).await?;

    transaction.commit().await?;

    db_ok(SellAllUnassignedRobotPartsResult { sold_count })
}

async fn sell_unassigned_robot_parts_counted(
    transaction: &mut sqlx::Transaction<'_, sqlx::MySql>,
    user_id: i64,
    robot_part_id: i64,
    count: i32,
) -> Result<DbOutcome<i32, RobotPartTransactionRejection>, sqlx::Error> {
    if count <= 0 {
        return db_ok(0);
    }

    let Some(ore_price_id) = robot_part_ore_price_id(transaction, robot_part_id).await? else {
        return db_reject(RobotPartTransactionRejection::UnknownRobotPart);
    };

    // Caller already validated unassigned stock under the same transaction
    // (assets locked with FOR UPDATE in list_user_sellable_robot_part_counts).
    remove_user_robot_part_assets(transaction, user_id, robot_part_id, count).await?;

    let costs = list_ore_price_amounts(transaction, ore_price_id).await?;
    refund_half_ore_costs_scaled(transaction, user_id, &costs, count).await?;

    db_ok(count)
}

async fn sell_unassigned_robot_parts_in_transaction(
    transaction: &mut sqlx::Transaction<'_, sqlx::MySql>,
    user_id: i64,
    robot_part_id: i64,
    count: i32,
) -> Result<DbOutcome<i32, RobotPartTransactionRejection>, sqlx::Error> {
    if count <= 0 {
        return db_ok(0);
    }

    let total_owned = user_robot_part_total_owned(transaction, user_id, robot_part_id).await?;
    let usage_count = user_robot_part_usage_count(transaction, user_id, robot_part_id).await?;
    let unassigned = unassigned_robot_part_count(total_owned, usage_count);

    if unassigned < count {
        return db_reject(RobotPartTransactionRejection::NoUnassignedRobotPart);
    }

    sell_unassigned_robot_parts_counted(transaction, user_id, robot_part_id, count).await
}

async fn sell_one_unassigned_robot_part_in_transaction(
    transaction: &mut sqlx::Transaction<'_, sqlx::MySql>,
    user_id: i64,
    robot_part_id: i64,
) -> Result<DbOutcome<(), RobotPartTransactionRejection>, sqlx::Error> {
    match sell_unassigned_robot_parts_in_transaction(transaction, user_id, robot_part_id, 1).await?
    {
        DbOutcome::Success(_) => db_ok(()),
        DbOutcome::Rejected(rejection) => db_reject(rejection),
    }
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

    let usage_by_part = user_robot_part_usage_counts_for_user(transaction, user_id).await?;

    let mut sellable_parts = Vec::new();
    for (robot_part_id, total_owned) in rows {
        let usage_count = usage_by_part.get(&robot_part_id).copied().unwrap_or(0);
        let unassigned = unassigned_robot_part_count(total_owned, usage_count);
        if unassigned > 0 {
            sellable_parts.push((robot_part_id, unassigned));
        }
    }

    Ok(sellable_parts)
}
