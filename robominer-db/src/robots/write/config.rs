use sqlx::MySqlPool;

use super::super::parameters::{robot_is_recharging, robot_parameters_for_parts};
use super::helpers::{
    default_program_source_code, find_or_create_default_program_source, generated_robot_name,
    load_default_robot_parts, load_pending_robot_changes_for_update,
    load_program_source_for_update, load_requested_robot_parts, load_robot_for_update,
    robot_part_baseline, user_has_unassigned_parts_for_update, valid_robot_name,
};
use super::pending::{
    delete_pending_robot_config, insert_pending_robot_config, update_pending_robot_config,
    update_robot_config_immediately, update_robot_header,
};
use crate::catalog::DEFAULT_PART_IDS;
use crate::mining_queue::robot_waiting_queue_count;
use crate::shop::add_user_robot_part_asset;
use crate::users::touch_user_last_login_time;
use crate::{
    DbOutcome, UpdateRobotConfigRejection, UpdateRobotConfigRequest, UpdatedRobotConfig, db_ok,
    db_reject,
};

pub async fn update_robot_config(
    pool: &MySqlPool,
    request: UpdateRobotConfigRequest,
) -> Result<DbOutcome<UpdatedRobotConfig, UpdateRobotConfigRejection>, sqlx::Error> {
    let mut transaction = pool.begin().await?;

    let Some(robot) =
        load_robot_for_update(&mut transaction, request.robot_id, request.user_id).await?
    else {
        transaction.rollback().await?;
        return db_reject(UpdateRobotConfigRejection::UnknownRobot);
    };

    let pending = load_pending_robot_changes_for_update(&mut transaction, request.robot_id).await?;
    let has_pending = pending.is_some();

    if !valid_robot_name(&request.robot_name) {
        transaction.rollback().await?;
        return db_reject(UpdateRobotConfigRejection::InvalidRobotName);
    }

    let Some(program_source) = load_program_source_for_update(
        &mut transaction,
        request.program_source_id,
        request.user_id,
    )
    .await?
    else {
        transaction.rollback().await?;
        return db_reject(UpdateRobotConfigRejection::UnknownProgramSource);
    };

    let Some(parts) = load_requested_robot_parts(&mut transaction, &request).await? else {
        transaction.rollback().await?;
        return db_reject(UpdateRobotConfigRejection::UnknownRobotPart);
    };

    if parts.memory_module.memory_capacity < program_source.compiled_size {
        transaction.rollback().await?;
        return db_reject(UpdateRobotConfigRejection::ProgramTooLarge);
    }

    let baseline_parts = robot_part_baseline(&robot, pending.as_ref());
    if !user_has_unassigned_parts_for_update(
        &mut transaction,
        robot.user_id,
        robot.id,
        &baseline_parts,
        &request,
    )
    .await?
    {
        transaction.rollback().await?;
        return db_reject(UpdateRobotConfigRejection::NoUnassignedRobotPart);
    }

    let Some(parameters) = robot_parameters_for_parts(&parts) else {
        transaction.rollback().await?;
        return db_reject(UpdateRobotConfigRejection::InvalidRobotPartConfiguration);
    };

    let source_code = if program_source.verified {
        program_source.source_code.unwrap_or_default()
    } else if let Some(pending) = &pending {
        pending.source_code.clone()
    } else {
        robot.source_code.clone()
    };
    let waiting_queue_count = robot_waiting_queue_count(&mut transaction, request.robot_id).await?;
    let recharging = robot_is_recharging(&mut transaction, request.robot_id).await?;
    let still_queued = waiting_queue_count > 0 && !recharging;

    if has_pending {
        if still_queued {
            update_pending_robot_config(&mut transaction, &request, &source_code, &parameters)
                .await?;
            update_robot_header(&mut transaction, &request).await?;
        } else {
            delete_pending_robot_config(&mut transaction, request.robot_id).await?;
            update_robot_config_immediately(&mut transaction, &request, &source_code, &parameters)
                .await?;
        }
    } else if still_queued {
        insert_pending_robot_config(
            &mut transaction,
            &robot,
            &request,
            &source_code,
            &parameters,
        )
        .await?;
        update_robot_header(&mut transaction, &request).await?;
    } else {
        update_robot_config_immediately(&mut transaction, &request, &source_code, &parameters)
            .await?;
    }

    touch_user_last_login_time(&mut transaction, request.user_id).await?;

    transaction.commit().await?;

    db_ok(UpdatedRobotConfig {
        robot_id: request.robot_id,
        pending: still_queued,
    })
}
pub(crate) async fn user_robot_count(
    transaction: &mut sqlx::Transaction<'_, sqlx::MySql>,
    user_id: i64,
) -> Result<i64, sqlx::Error> {
    sqlx::query_scalar("SELECT COUNT(*) FROM Robot WHERE userId = ?")
        .bind(user_id)
        .fetch_one(&mut **transaction)
        .await
}
pub(crate) async fn add_default_robot_for_user(
    transaction: &mut sqlx::Transaction<'_, sqlx::MySql>,
    user_id: i64,
) -> Result<(), sqlx::Error> {
    let username: String = sqlx::query_scalar("SELECT username FROM User WHERE id = ?")
        .bind(user_id)
        .fetch_one(&mut **transaction)
        .await?;
    let robot_count = user_robot_count(transaction, user_id).await?;
    let robot_name = generated_robot_name(&username, robot_count + 1);

    for robot_part_id in DEFAULT_PART_IDS {
        add_user_robot_part_asset(transaction, user_id, robot_part_id).await?;
    }

    let Some(parts) = load_default_robot_parts(transaction).await? else {
        return Err(sqlx::Error::RowNotFound);
    };
    let Some(parameters) = robot_parameters_for_parts(&parts) else {
        return Err(sqlx::Error::RowNotFound);
    };

    let program_source = find_or_create_default_program_source(
        transaction,
        user_id,
        parts.memory_module.memory_capacity,
    )
    .await?;

    let source_code = program_source
        .source_code
        .unwrap_or_else(default_program_source_code);

    sqlx::query!(
        "INSERT INTO Robot \
         (userId, robotName, sourceCode, programSourceId, oreContainerId, miningUnitId, \
          batteryId, memoryModuleId, cpuId, engineId, oreScannerId, rechargeTime, maxOre, \
          miningSpeed, maxTurns, memorySize, cpuSpeed, forwardSpeed, backwardSpeed, rotateSpeed, \
          robotSize, scanTime, scanDistance) \
         VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)",
        user_id,
        robot_name,
        source_code,
        program_source.id,
        DEFAULT_PART_IDS[0],
        DEFAULT_PART_IDS[1],
        DEFAULT_PART_IDS[2],
        DEFAULT_PART_IDS[3],
        DEFAULT_PART_IDS[4],
        DEFAULT_PART_IDS[5],
        DEFAULT_PART_IDS[6],
        parameters.recharge_time,
        parameters.max_ore,
        parameters.mining_speed,
        parameters.max_turns,
        parameters.memory_size,
        parameters.cpu_speed,
        parameters.forward_speed,
        parameters.backward_speed,
        parameters.rotate_speed,
        parameters.robot_size,
        parameters.scan_time,
        parameters.scan_distance
    )
    .execute(&mut **transaction)
    .await?;

    Ok(())
}
