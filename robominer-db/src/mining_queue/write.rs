use sqlx::MySqlPool;

use crate::users::touch_user_last_login_time;

use super::read::mining_queue_item_cancelable;
use crate::assets::{
    can_pay_ore_costs, deduct_ore_costs, list_ore_price_amounts, ore_refund_fits_without_clamp_tx,
    refund_full_ore_costs,
};
use crate::{
    CancelMiningQueueBatchResult, CancelMiningQueueRejection, CancelMiningQueueRequest,
    CanceledMiningQueue, DbOutcome, EnqueueMiningRejection, EnqueueMiningRequest, EnqueuedMining,
    db_ok, db_reject,
};

pub async fn enqueue_mining(
    pool: &MySqlPool,
    request: EnqueueMiningRequest,
) -> Result<DbOutcome<EnqueuedMining, EnqueueMiningRejection>, sqlx::Error> {
    let mut transaction = pool.begin().await?;

    if !lock_robot_for_enqueue(&mut transaction, request.robot_id, request.user_id).await? {
        transaction.rollback().await?;
        return db_reject(EnqueueMiningRejection::UnknownRobot);
    }

    let Some(ore_price_id) = sqlx::query_scalar!(
        r#"SELECT orePriceId AS "ore_price_id!: i64" FROM MiningArea WHERE id = ?"#,
        request.mining_area_id
    )
    .fetch_optional(&mut *transaction)
    .await?
    else {
        transaction.rollback().await?;
        return db_reject(EnqueueMiningRejection::UnknownMiningArea);
    };

    if !user_has_mining_area(&mut transaction, request.user_id, request.mining_area_id).await? {
        transaction.rollback().await?;
        return db_reject(EnqueueMiningRejection::MiningAreaUnavailable);
    }

    let mining_queue_size = user_mining_queue_size(&mut transaction, request.user_id).await?;
    let waiting_count = robot_waiting_queue_count(&mut transaction, request.robot_id).await?;

    if waiting_count >= mining_queue_size {
        transaction.rollback().await?;
        return db_reject(EnqueueMiningRejection::QueueFull);
    }

    let requested_count = if request.fill {
        (mining_queue_size - waiting_count) as u64
    } else {
        1
    };
    let costs = list_ore_price_amounts(&mut transaction, ore_price_id).await?;
    let mut inserted_queues = 0;

    for _ in 0..requested_count {
        if !can_pay_ore_costs(&mut transaction, request.user_id, &costs).await? {
            break;
        }

        deduct_ore_costs(&mut transaction, request.user_id, &costs).await?;
        insert_mining_queue(&mut transaction, request.robot_id, request.mining_area_id).await?;
        inserted_queues += 1;
    }

    if inserted_queues == 0 {
        transaction.rollback().await?;
        return db_reject(EnqueueMiningRejection::InsufficientFunds);
    }

    touch_user_last_login_time(&mut transaction, request.user_id).await?;

    transaction.commit().await?;

    db_ok(EnqueuedMining { inserted_queues })
}

pub async fn cancel_mining_queue(
    pool: &MySqlPool,
    request: CancelMiningQueueRequest,
) -> Result<DbOutcome<CanceledMiningQueue, CancelMiningQueueRejection>, sqlx::Error> {
    let mut transaction = pool.begin().await?;
    let outcome = cancel_mining_queue_in_transaction(&mut transaction, request).await?;
    match outcome {
        DbOutcome::Success(value) => {
            touch_user_last_login_time(&mut transaction, request.user_id).await?;
            transaction.commit().await?;
            db_ok(value)
        }
        DbOutcome::Rejected(rejection) => {
            transaction.rollback().await?;
            db_reject(rejection)
        }
    }
}

pub async fn cancel_mining_queue_batch(
    pool: &MySqlPool,
    user_id: i64,
    mining_queue_ids: &[i64],
    require_refund_fits: bool,
) -> Result<CancelMiningQueueBatchResult, sqlx::Error> {
    let mut transaction = pool.begin().await?;
    let mut batch = CancelMiningQueueBatchResult::default();

    for &mining_queue_id in mining_queue_ids {
        match cancel_mining_queue_in_transaction(
            &mut transaction,
            CancelMiningQueueRequest {
                user_id,
                mining_queue_id,
                require_refund_fits,
            },
        )
        .await?
        {
            DbOutcome::Success(_) => batch.cleared += 1,
            DbOutcome::Rejected(CancelMiningQueueRejection::RefundWouldClamp) => {
                batch.skipped += 1;
            }
            DbOutcome::Rejected(rejection) => {
                batch.failed += 1;
                batch.last_rejection = Some(rejection);
                *batch.rejection_counts.entry(rejection).or_default() += 1;
            }
        }
    }

    if batch.cleared > 0 {
        touch_user_last_login_time(&mut transaction, user_id).await?;
        transaction.commit().await?;
    } else {
        transaction.rollback().await?;
    }

    Ok(batch)
}

async fn cancel_mining_queue_in_transaction(
    transaction: &mut sqlx::Transaction<'_, sqlx::MySql>,
    request: CancelMiningQueueRequest,
) -> Result<DbOutcome<CanceledMiningQueue, CancelMiningQueueRejection>, sqlx::Error> {
    let Some((robot_id, owner_id, rally_result_id, mining_end_time_is_null, mining_area_id)) =
        sqlx::query_as::<_, (i64, i64, Option<i64>, bool, i64)>(
            "SELECT MiningQueue.robotId, Robot.userId, MiningQueue.rallyResultId, \
                    MiningQueue.miningEndTime IS NULL, MiningQueue.miningAreaId \
             FROM MiningQueue \
             INNER JOIN Robot ON Robot.id = MiningQueue.robotId \
             WHERE MiningQueue.id = ? \
             FOR UPDATE",
        )
        .bind(request.mining_queue_id)
        .fetch_optional(&mut **transaction)
        .await?
    else {
        return db_reject(CancelMiningQueueRejection::UnknownQueue);
    };

    if owner_id != request.user_id {
        return db_reject(CancelMiningQueueRejection::WrongOwner);
    }

    let earlier_unfinished_queue_count: i64 = sqlx::query_scalar(
        "SELECT COUNT(*) \
         FROM MiningQueue \
         WHERE robotId = ? \
           AND id < ? \
           AND (miningEndTime IS NULL OR miningEndTime > NOW())",
    )
    .bind(robot_id)
    .bind(request.mining_queue_id)
    .fetch_one(&mut **transaction)
    .await?;

    if !mining_queue_item_cancelable(
        rally_result_id,
        mining_end_time_is_null,
        earlier_unfinished_queue_count,
    ) {
        return db_reject(CancelMiningQueueRejection::NotCancelable);
    }

    let Some(ore_price_id) = sqlx::query_scalar!(
        r#"SELECT orePriceId AS "ore_price_id!: i64" FROM MiningArea WHERE id = ?"#,
        mining_area_id
    )
    .fetch_optional(&mut **transaction)
    .await?
    else {
        return db_reject(CancelMiningQueueRejection::UnknownQueue);
    };
    let costs = list_ore_price_amounts(transaction, ore_price_id).await?;
    if request.require_refund_fits
        && !ore_refund_fits_without_clamp_tx(transaction, request.user_id, &costs).await?
    {
        return db_reject(CancelMiningQueueRejection::RefundWouldClamp);
    }
    refund_full_ore_costs(transaction, request.user_id, &costs).await?;

    sqlx::query!(
        r#"DELETE FROM MiningQueue WHERE id = ?"#,
        request.mining_queue_id
    )
    .execute(&mut **transaction)
    .await?;

    db_ok(CanceledMiningQueue {
        mining_queue_id: request.mining_queue_id,
    })
}

async fn lock_robot_for_enqueue(
    transaction: &mut sqlx::Transaction<'_, sqlx::MySql>,
    robot_id: i64,
    user_id: i64,
) -> Result<bool, sqlx::Error> {
    // FOR UPDATE is kept on a runtime query; ownership check shape is covered by
    // nearby compile-checked SELECTs and enqueue integration tests.
    let exists: Option<i64> =
        sqlx::query_scalar("SELECT id FROM Robot WHERE id = ? AND userId = ? FOR UPDATE")
            .bind(robot_id)
            .bind(user_id)
            .fetch_optional(&mut **transaction)
            .await?;

    Ok(exists.is_some())
}

async fn user_has_mining_area(
    transaction: &mut sqlx::Transaction<'_, sqlx::MySql>,
    user_id: i64,
    mining_area_id: i64,
) -> Result<bool, sqlx::Error> {
    let exists = sqlx::query_scalar!(
        r#"
SELECT miningAreaId AS "mining_area_id!: i64"
FROM UserMiningArea
WHERE userId = ? AND miningAreaId = ?
        "#,
        user_id,
        mining_area_id
    )
    .fetch_optional(&mut **transaction)
    .await?;

    Ok(exists.is_some())
}

async fn user_mining_queue_size(
    transaction: &mut sqlx::Transaction<'_, sqlx::MySql>,
    user_id: i64,
) -> Result<i64, sqlx::Error> {
    sqlx::query_scalar!(
        r#"SELECT miningQueueSize AS "size!: i64" FROM User WHERE id = ?"#,
        user_id
    )
    .fetch_one(&mut **transaction)
    .await
}

pub(crate) async fn robot_waiting_queue_count(
    transaction: &mut sqlx::Transaction<'_, sqlx::MySql>,
    robot_id: i64,
) -> Result<i64, sqlx::Error> {
    sqlx::query_scalar!(
        r#"
SELECT COUNT(*) AS "count!: i64"
FROM MiningQueue
WHERE robotId = ?
  AND (miningEndTime IS NULL OR miningEndTime > NOW())
        "#,
        robot_id
    )
    .fetch_one(&mut **transaction)
    .await
}

async fn insert_mining_queue(
    transaction: &mut sqlx::Transaction<'_, sqlx::MySql>,
    robot_id: i64,
    mining_area_id: i64,
) -> Result<(), sqlx::Error> {
    sqlx::query!(
        r#"
INSERT INTO MiningQueue (miningAreaId, robotId)
VALUES (?, ?)
        "#,
        mining_area_id,
        robot_id
    )
    .execute(&mut **transaction)
    .await?;

    Ok(())
}
