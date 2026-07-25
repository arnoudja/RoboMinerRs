use sqlx::MySqlPool;

use crate::mining_queue::robot_waiting_queue_count;
use crate::robots::robot_is_recharging;
use crate::users::touch_user_last_login_time;
use crate::{AppliedProgramSource, ProgramSourceApplyWarning, ProgramSourceApplyWarningReason};

pub async fn apply_verified_program_source_to_idle_robots(
    pool: &MySqlPool,
    user_id: i64,
    program_source_id: i64,
) -> Result<AppliedProgramSource, sqlx::Error> {
    let mut transaction = pool.begin().await?;

    let Some((source_code, verified, compiled_size)) = sqlx::query_as::<_, (String, bool, i32)>(
        "SELECT sourceCode, verified, compiledSize \
             FROM ProgramSource \
             WHERE id = ? AND userId = ? \
             FOR UPDATE",
    )
    .bind(program_source_id)
    .bind(user_id)
    .fetch_optional(&mut *transaction)
    .await?
    else {
        transaction.commit().await?;
        return Ok(AppliedProgramSource {
            applied_robots: 0,
            warnings: Vec::new(),
        });
    };

    if !verified {
        transaction.commit().await?;
        return Ok(AppliedProgramSource {
            applied_robots: 0,
            warnings: Vec::new(),
        });
    }

    let robots = list_program_source_robots(&mut transaction, user_id, program_source_id).await?;
    let mut applied_robots = 0;
    let mut warnings = Vec::new();

    for robot in robots {
        if robot.memory_size < compiled_size {
            warnings.push(ProgramSourceApplyWarning {
                robot_name: robot.robot_name,
                reason: ProgramSourceApplyWarningReason::NotEnoughMemory,
            });
            continue;
        }

        let waiting_queue_count = robot_waiting_queue_count(&mut transaction, robot.id).await?;
        let recharging = robot_is_recharging(&mut transaction, robot.id).await?;
        let still_queued = waiting_queue_count > 0 && !recharging;

        if still_queued {
            if robot.has_pending {
                update_pending_program_source(&mut transaction, robot.id, &source_code).await?;
            } else {
                insert_pending_program_source_from_robot(
                    &mut transaction,
                    robot.id,
                    user_id,
                    &source_code,
                )
                .await?;
            }
            applied_robots += 1;
        } else {
            if robot.has_pending {
                delete_pending_robot_program_source(&mut transaction, robot.id).await?;
            }
            sqlx::query("UPDATE Robot SET sourceCode = ? WHERE id = ? AND userId = ?")
                .bind(&source_code)
                .bind(robot.id)
                .bind(user_id)
                .execute(&mut *transaction)
                .await?;
            applied_robots += 1;
        }
    }

    touch_user_last_login_time(&mut transaction, user_id).await?;

    transaction.commit().await?;

    Ok(AppliedProgramSource {
        applied_robots,
        warnings,
    })
}

#[derive(Debug, Clone)]
struct ProgramSourceRobotState {
    id: i64,
    robot_name: String,
    memory_size: i32,
    has_pending: bool,
}

async fn insert_pending_program_source_from_robot(
    transaction: &mut sqlx::Transaction<'_, sqlx::MySql>,
    robot_id: i64,
    user_id: i64,
    source_code: &str,
) -> Result<(), sqlx::Error> {
    sqlx::query(
        "INSERT INTO PendingRobotChanges \
         (robotId, sourceCode, oreContainerId, miningUnitId, batteryId, memoryModuleId, \
          cpuId, engineId, oreScannerId, oldOreContainerId, oldMiningUnitId, oldBatteryId, \
          oldMemoryModuleId, oldCpuId, oldEngineId, oldOreScannerId, rechargeTime, maxOre, \
          miningSpeed, maxTurns, memorySize, cpuSpeed, forwardSpeed, backwardSpeed, rotateSpeed, \
          robotSize, scanTime, scanDistance, changesCommitTime) \
         SELECT id, ?, oreContainerId, miningUnitId, batteryId, memoryModuleId, cpuId, engineId, \
                oreScannerId, oreContainerId, miningUnitId, batteryId, memoryModuleId, cpuId, \
                engineId, oreScannerId, rechargeTime, maxOre, miningSpeed, maxTurns, memorySize, \
                cpuSpeed, forwardSpeed, backwardSpeed, rotateSpeed, robotSize, scanTime, scanDistance, \
                NULL \
         FROM Robot \
         WHERE id = ? AND userId = ?",
    )
    .bind(source_code)
    .bind(robot_id)
    .bind(user_id)
    .execute(&mut **transaction)
    .await?;

    Ok(())
}

async fn update_pending_program_source(
    transaction: &mut sqlx::Transaction<'_, sqlx::MySql>,
    robot_id: i64,
    source_code: &str,
) -> Result<(), sqlx::Error> {
    sqlx::query("UPDATE PendingRobotChanges SET sourceCode = ? WHERE robotId = ?")
        .bind(source_code)
        .bind(robot_id)
        .execute(&mut **transaction)
        .await?;

    Ok(())
}

async fn delete_pending_robot_program_source(
    transaction: &mut sqlx::Transaction<'_, sqlx::MySql>,
    robot_id: i64,
) -> Result<(), sqlx::Error> {
    sqlx::query("DELETE FROM PendingRobotChanges WHERE robotId = ?")
        .bind(robot_id)
        .execute(&mut **transaction)
        .await?;

    Ok(())
}

async fn list_program_source_robots(
    transaction: &mut sqlx::Transaction<'_, sqlx::MySql>,
    user_id: i64,
    program_source_id: i64,
) -> Result<Vec<ProgramSourceRobotState>, sqlx::Error> {
    let rows = sqlx::query_as::<_, (i64, String, i32, bool)>(
        "SELECT Robot.id, Robot.robotName, \
                COALESCE(PendingRobotChanges.memorySize, Robot.memorySize) AS memorySize, \
                PendingRobotChanges.robotId IS NOT NULL AS hasPending \
         FROM Robot \
         LEFT JOIN PendingRobotChanges ON PendingRobotChanges.robotId = Robot.id \
         WHERE Robot.userId = ? AND Robot.programSourceId = ? \
         ORDER BY Robot.id \
         FOR UPDATE",
    )
    .bind(user_id)
    .bind(program_source_id)
    .fetch_all(&mut **transaction)
    .await?;

    Ok(rows
        .into_iter()
        .map(
            |(id, robot_name, memory_size, has_pending)| ProgramSourceRobotState {
                id,
                robot_name,
                memory_size,
                has_pending,
            },
        )
        .collect())
}
