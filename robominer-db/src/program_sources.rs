use sqlx::MySqlPool;
use sqlx::mysql::MySqlQueryResult;

use crate::mappers::{ProgramSourceRow, program_source_rows, program_source_state_record};
use crate::mining_queue::robot_waiting_queue_count;
use crate::robots::robot_is_recharging;
use crate::users::{touch_user_last_login_time, user_exists};
use crate::{
    AppliedProgramSource, CreateProgramSourceRequest, CreatedProgramSource,
    ProgramSourceApplyWarning, ProgramSourceApplyWarningReason, ProgramSourceRecord,
    ProgramSourceVerification, ProgramSourceWriteRejection, ProgramSourceWriteRequest,
};

pub async fn get_program_source(
    pool: &MySqlPool,
    program_source_id: i64,
) -> Result<Option<String>, sqlx::Error> {
    sqlx::query_scalar("SELECT sourceCode FROM ProgramSource WHERE id = ?")
        .bind(program_source_id)
        .fetch_optional(pool)
        .await
}

pub async fn list_program_sources_for_user(
    pool: &MySqlPool,
    user_id: i64,
) -> Result<Vec<ProgramSourceRecord>, sqlx::Error> {
    sqlx::query_as::<_, ProgramSourceRow>(
        "SELECT id, userId, sourceName, sourceCode, verified, compiledSize, errorDescription \
         FROM ProgramSource \
         WHERE userId = ? \
         ORDER BY id",
    )
    .bind(user_id)
    .fetch_all(pool)
    .await
    .map(program_source_rows)
}

pub async fn list_program_source_states_for_user(
    pool: &MySqlPool,
    user_id: i64,
) -> Result<Vec<crate::ProgramSourceStateRecord>, sqlx::Error> {
    let rows = sqlx::query(
        "SELECT ProgramSource.id, ProgramSource.userId, ProgramSource.sourceName, \
                ProgramSource.sourceCode, ProgramSource.verified, ProgramSource.compiledSize, \
                ProgramSource.errorDescription, \
                (SELECT COUNT(*) FROM Robot WHERE Robot.programSourceId = ProgramSource.id) \
                  AS linkedRobotCount \
         FROM ProgramSource \
         WHERE ProgramSource.userId = ? \
         ORDER BY ProgramSource.id",
    )
    .bind(user_id)
    .fetch_all(pool)
    .await?;

    rows.into_iter().map(program_source_state_record).collect()
}

pub async fn insert_program_source(
    pool: &MySqlPool,
    user_id: i64,
    source_name: &str,
    source_code: &str,
) -> Result<i64, sqlx::Error> {
    let result: MySqlQueryResult = sqlx::query(
        "INSERT INTO ProgramSource \
         (userId, sourceName, sourceCode, verified, compiledSize, errorDescription) \
         VALUES (?, ?, ?, false, -1, '')",
    )
    .bind(user_id)
    .bind(source_name)
    .bind(source_code)
    .execute(pool)
    .await?;

    Ok(result.last_insert_id() as i64)
}

pub async fn create_program_source(
    pool: &MySqlPool,
    request: CreateProgramSourceRequest,
) -> Result<Result<CreatedProgramSource, ProgramSourceWriteRejection>, sqlx::Error> {
    let mut transaction = pool.begin().await?;

    if let Some(rejection) =
        validate_program_source_write(&request.source_name, &request.source_code)
    {
        transaction.rollback().await?;
        return Ok(Err(rejection));
    }

    if !user_exists(&mut transaction, request.user_id).await? {
        transaction.rollback().await?;
        return Ok(Err(ProgramSourceWriteRejection::UnknownUser));
    }

    let result = sqlx::query(
        "INSERT INTO ProgramSource \
         (userId, sourceName, sourceCode, verified, compiledSize, errorDescription) \
         VALUES (?, ?, ?, false, -1, '')",
    )
    .bind(request.user_id)
    .bind(&request.source_name)
    .bind(&request.source_code)
    .execute(&mut *transaction)
    .await?;

    touch_user_last_login_time(&mut transaction, request.user_id).await?;

    transaction.commit().await?;

    Ok(Ok(CreatedProgramSource {
        program_source_id: result.last_insert_id() as i64,
    }))
}

pub async fn get_program_source_verification(
    pool: &MySqlPool,
    program_source_id: i64,
) -> Result<Option<ProgramSourceVerification>, sqlx::Error> {
    sqlx::query_as::<_, (bool, i32, Option<String>)>(
        "SELECT verified, compiledSize, errorDescription \
         FROM ProgramSource \
         WHERE id = ?",
    )
    .bind(program_source_id)
    .fetch_optional(pool)
    .await
    .map(|row| {
        row.map(
            |(verified, compiled_size, error_description)| ProgramSourceVerification {
                verified,
                compiled_size,
                error_description: error_description.unwrap_or_default(),
            },
        )
    })
}

pub async fn delete_program_source(
    pool: &MySqlPool,
    program_source_id: i64,
) -> Result<(), sqlx::Error> {
    sqlx::query("DELETE FROM ProgramSource WHERE id = ?")
        .bind(program_source_id)
        .execute(pool)
        .await?;

    Ok(())
}

pub async fn update_program_source(
    pool: &MySqlPool,
    request: ProgramSourceWriteRequest,
) -> Result<Result<(), ProgramSourceWriteRejection>, sqlx::Error> {
    let mut transaction = pool.begin().await?;

    if let Some(rejection) =
        validate_program_source_write(&request.source_name, &request.source_code)
    {
        transaction.rollback().await?;
        return Ok(Err(rejection));
    }

    if !program_source_belongs_to_user(&mut transaction, request.program_source_id, request.user_id)
        .await?
    {
        transaction.rollback().await?;
        return Ok(Err(ProgramSourceWriteRejection::UnknownProgramSource));
    }

    sqlx::query(
        "UPDATE ProgramSource \
         SET sourceName = ?, sourceCode = ?, verified = false \
         WHERE id = ? AND userId = ?",
    )
    .bind(&request.source_name)
    .bind(&request.source_code)
    .bind(request.program_source_id)
    .bind(request.user_id)
    .execute(&mut *transaction)
    .await?;

    touch_user_last_login_time(&mut transaction, request.user_id).await?;

    transaction.commit().await?;
    Ok(Ok(()))
}

pub async fn delete_program_source_for_user(
    pool: &MySqlPool,
    user_id: i64,
    program_source_id: i64,
) -> Result<Result<(), ProgramSourceWriteRejection>, sqlx::Error> {
    let mut transaction = pool.begin().await?;

    if !program_source_belongs_to_user(&mut transaction, program_source_id, user_id).await? {
        transaction.rollback().await?;
        return Ok(Err(ProgramSourceWriteRejection::UnknownProgramSource));
    }

    if program_source_robot_count(&mut transaction, program_source_id).await? > 0 {
        transaction.rollback().await?;
        return Ok(Err(ProgramSourceWriteRejection::SourceInUse));
    }

    sqlx::query("DELETE FROM ProgramSource WHERE id = ? AND userId = ?")
        .bind(program_source_id)
        .bind(user_id)
        .execute(&mut *transaction)
        .await?;

    touch_user_last_login_time(&mut transaction, user_id).await?;

    transaction.commit().await?;
    Ok(Ok(()))
}

pub async fn set_valid_program_source(
    pool: &MySqlPool,
    program_source_id: i64,
    compiled_size: i32,
) -> Result<(), sqlx::Error> {
    sqlx::query(
        "UPDATE ProgramSource \
         SET errorDescription = '', verified = true, compiledSize = ? \
         WHERE id = ?",
    )
    .bind(compiled_size)
    .bind(program_source_id)
    .execute(pool)
    .await?;

    Ok(())
}

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

pub async fn set_invalid_program_source(
    pool: &MySqlPool,
    program_source_id: i64,
    error_description: &str,
) -> Result<(), sqlx::Error> {
    sqlx::query(
        "UPDATE ProgramSource \
         SET errorDescription = ?, verified = false, compiledSize = -1 \
         WHERE id = ?",
    )
    .bind(error_description)
    .bind(program_source_id)
    .execute(pool)
    .await?;

    Ok(())
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

async fn program_source_belongs_to_user(
    transaction: &mut sqlx::Transaction<'_, sqlx::MySql>,
    program_source_id: i64,
    user_id: i64,
) -> Result<bool, sqlx::Error> {
    let exists: Option<i64> =
        sqlx::query_scalar("SELECT id FROM ProgramSource WHERE id = ? AND userId = ? FOR UPDATE")
            .bind(program_source_id)
            .bind(user_id)
            .fetch_optional(&mut **transaction)
            .await?;

    Ok(exists.is_some())
}

async fn program_source_robot_count(
    transaction: &mut sqlx::Transaction<'_, sqlx::MySql>,
    program_source_id: i64,
) -> Result<i64, sqlx::Error> {
    sqlx::query_scalar("SELECT COUNT(*) FROM Robot WHERE programSourceId = ?")
        .bind(program_source_id)
        .fetch_one(&mut **transaction)
        .await
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

pub(crate) fn validate_program_source_write(
    source_name: &str,
    source_code: &str,
) -> Option<ProgramSourceWriteRejection> {
    if source_name.is_empty() {
        return Some(ProgramSourceWriteRejection::EmptySourceName);
    }
    if source_code.is_empty() {
        return Some(ProgramSourceWriteRejection::EmptySourceCode);
    }
    None
}

#[cfg(test)]
mod tests {
    use super::validate_program_source_write;
    use crate::ProgramSourceWriteRejection;

    #[test]
    fn validate_program_source_write_requires_name_and_code() {
        assert_eq!(
            validate_program_source_write("", "mine();"),
            Some(ProgramSourceWriteRejection::EmptySourceName)
        );
        assert_eq!(
            validate_program_source_write("main", ""),
            Some(ProgramSourceWriteRejection::EmptySourceCode)
        );
        assert_eq!(validate_program_source_write("main", "mine();"), None);
    }
}
