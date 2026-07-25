use sqlx::Row;

use super::super::super::{
    PendingRobotUpdateState, ProgramSourceUpdateState, RequestedRobotParts, RobotUpdateState,
};
use crate::mappers::{ProgramSourceRow, program_source_record, robot_part_record};
use crate::{ProgramSourceRecord, RobotPartRecord, UpdateRobotConfigRequest};

pub(crate) fn generated_robot_name(username: &str, robot_number: i64) -> String {
    let prefix: String = username.chars().take(10).collect();
    format!("{prefix}_{robot_number}")
}
pub(crate) async fn load_default_robot_parts(
    transaction: &mut sqlx::Transaction<'_, sqlx::MySql>,
) -> Result<Option<RequestedRobotParts>, sqlx::Error> {
    Ok(Some(RequestedRobotParts {
        ore_container: load_robot_part(transaction, 101)
            .await?
            .ok_or(sqlx::Error::RowNotFound)?,
        mining_unit: load_robot_part(transaction, 201)
            .await?
            .ok_or(sqlx::Error::RowNotFound)?,
        battery: load_robot_part(transaction, 301)
            .await?
            .ok_or(sqlx::Error::RowNotFound)?,
        memory_module: load_robot_part(transaction, 401)
            .await?
            .ok_or(sqlx::Error::RowNotFound)?,
        cpu: load_robot_part(transaction, 501)
            .await?
            .ok_or(sqlx::Error::RowNotFound)?,
        engine: load_robot_part(transaction, 601)
            .await?
            .ok_or(sqlx::Error::RowNotFound)?,
        ore_scanner: load_robot_part(transaction, 701)
            .await?
            .ok_or(sqlx::Error::RowNotFound)?,
    }))
}

pub(crate) async fn find_or_create_default_program_source(
    transaction: &mut sqlx::Transaction<'_, sqlx::MySql>,
    user_id: i64,
    max_size: i32,
) -> Result<ProgramSourceRecord, sqlx::Error> {
    if let Some(row) = sqlx::query_as::<_, ProgramSourceRow>(
        "SELECT id, userId, sourceName, sourceCode, verified, compiledSize, errorDescription \
         FROM ProgramSource \
         WHERE userId = ? AND compiledSize <= ? AND verified = true \
         ORDER BY id \
         LIMIT 1",
    )
    .bind(user_id)
    .bind(max_size)
    .fetch_optional(&mut **transaction)
    .await?
    {
        return Ok(program_source_record(row));
    }

    let source_code = default_program_source_code();
    let result = sqlx::query(
        "INSERT INTO ProgramSource \
         (userId, sourceName, sourceCode, verified, compiledSize, errorDescription) \
         VALUES (?, 'Default program', ?, true, 4, '')",
    )
    .bind(user_id)
    .bind(&source_code)
    .execute(&mut **transaction)
    .await?;

    Ok(ProgramSourceRecord {
        id: result.last_insert_id() as i64,
        user_id,
        source_name: "Default program".to_string(),
        source_code: Some(source_code),
        verified: true,
        compiled_size: 4,
        error_description: String::new(),
    })
}

pub(crate) fn default_program_source_code() -> String {
    "move(1);\nmine();".to_string()
}
pub(crate) async fn load_robot_for_update(
    transaction: &mut sqlx::Transaction<'_, sqlx::MySql>,
    robot_id: i64,
    user_id: i64,
) -> Result<Option<RobotUpdateState>, sqlx::Error> {
    let row = sqlx::query(
        "SELECT id, userId, sourceCode, oreContainerId, miningUnitId, batteryId, \
                memoryModuleId, cpuId, engineId, oreScannerId \
         FROM Robot \
         WHERE id = ? AND userId = ? \
         FOR UPDATE",
    )
    .bind(robot_id)
    .bind(user_id)
    .fetch_optional(&mut **transaction)
    .await?;

    row.map(|row| {
        Ok(RobotUpdateState {
            id: row.try_get("id")?,
            user_id: row.try_get("userId")?,
            source_code: row.try_get("sourceCode")?,
            ore_container_id: row.try_get("oreContainerId")?,
            mining_unit_id: row.try_get("miningUnitId")?,
            battery_id: row.try_get("batteryId")?,
            memory_module_id: row.try_get("memoryModuleId")?,
            cpu_id: row.try_get("cpuId")?,
            engine_id: row.try_get("engineId")?,
            ore_scanner_id: row.try_get("oreScannerId")?,
        })
    })
    .transpose()
}

pub(crate) fn robot_part_baseline(
    robot: &RobotUpdateState,
    pending: Option<&PendingRobotUpdateState>,
) -> [Option<i64>; 7] {
    if let Some(pending) = pending {
        [
            pending.ore_container_id,
            pending.mining_unit_id,
            pending.battery_id,
            pending.memory_module_id,
            pending.cpu_id,
            pending.engine_id,
            pending.ore_scanner_id,
        ]
    } else {
        [
            robot.ore_container_id,
            robot.mining_unit_id,
            robot.battery_id,
            robot.memory_module_id,
            robot.cpu_id,
            robot.engine_id,
            robot.ore_scanner_id,
        ]
    }
}

pub(crate) async fn load_pending_robot_changes_for_update(
    transaction: &mut sqlx::Transaction<'_, sqlx::MySql>,
    robot_id: i64,
) -> Result<Option<PendingRobotUpdateState>, sqlx::Error> {
    let row = sqlx::query(
        "SELECT sourceCode, oreContainerId, miningUnitId, batteryId, memoryModuleId, \
                cpuId, engineId, oreScannerId \
         FROM PendingRobotChanges \
         WHERE robotId = ? \
         FOR UPDATE",
    )
    .bind(robot_id)
    .fetch_optional(&mut **transaction)
    .await?;

    row.map(|row| {
        Ok(PendingRobotUpdateState {
            source_code: row.try_get("sourceCode")?,
            ore_container_id: row.try_get("oreContainerId")?,
            mining_unit_id: row.try_get("miningUnitId")?,
            battery_id: row.try_get("batteryId")?,
            memory_module_id: row.try_get("memoryModuleId")?,
            cpu_id: row.try_get("cpuId")?,
            engine_id: row.try_get("engineId")?,
            ore_scanner_id: row.try_get("oreScannerId")?,
        })
    })
    .transpose()
}

pub(crate) async fn load_program_source_for_update(
    transaction: &mut sqlx::Transaction<'_, sqlx::MySql>,
    program_source_id: i64,
    user_id: i64,
) -> Result<Option<ProgramSourceUpdateState>, sqlx::Error> {
    let row = sqlx::query_as::<_, (Option<String>, bool, i32)>(
        "SELECT sourceCode, verified, compiledSize \
         FROM ProgramSource \
         WHERE id = ? AND userId = ? \
         FOR UPDATE",
    )
    .bind(program_source_id)
    .bind(user_id)
    .fetch_optional(&mut **transaction)
    .await?;

    Ok(row.map(
        |(source_code, verified, compiled_size)| ProgramSourceUpdateState {
            source_code,
            verified,
            compiled_size,
        },
    ))
}

pub(crate) async fn load_requested_robot_parts(
    transaction: &mut sqlx::Transaction<'_, sqlx::MySql>,
    request: &UpdateRobotConfigRequest,
) -> Result<Option<RequestedRobotParts>, sqlx::Error> {
    let Some(ore_container) =
        load_robot_part_for_update(transaction, request.ore_container_id).await?
    else {
        return Ok(None);
    };
    let Some(mining_unit) = load_robot_part_for_update(transaction, request.mining_unit_id).await?
    else {
        return Ok(None);
    };
    let Some(battery) = load_robot_part_for_update(transaction, request.battery_id).await? else {
        return Ok(None);
    };
    let Some(memory_module) =
        load_robot_part_for_update(transaction, request.memory_module_id).await?
    else {
        return Ok(None);
    };
    let Some(cpu) = load_robot_part_for_update(transaction, request.cpu_id).await? else {
        return Ok(None);
    };
    let Some(engine) = load_robot_part_for_update(transaction, request.engine_id).await? else {
        return Ok(None);
    };
    let Some(ore_scanner) = load_robot_part_for_update(transaction, request.ore_scanner_id).await?
    else {
        return Ok(None);
    };

    Ok(Some(RequestedRobotParts {
        ore_container,
        mining_unit,
        battery,
        memory_module,
        cpu,
        engine,
        ore_scanner,
    }))
}

async fn load_robot_part_for_update(
    transaction: &mut sqlx::Transaction<'_, sqlx::MySql>,
    robot_part_id: i64,
) -> Result<Option<RobotPartRecord>, sqlx::Error> {
    let row = sqlx::query(
        "SELECT id, typeId, tierId, partName, orePriceId, oreCapacity, miningCapacity, \
                batteryCapacity, memoryCapacity, cpuCapacity, forwardCapacity, backwardCapacity, \
                rotateCapacity, rechargeTime, scanTime, scanDistance, weight, volume, powerUsage \
         FROM RobotPart \
         WHERE id = ? \
         FOR UPDATE",
    )
    .bind(robot_part_id)
    .fetch_optional(&mut **transaction)
    .await?;

    row.map(robot_part_record).transpose()
}

pub(crate) async fn load_robot_part(
    transaction: &mut sqlx::Transaction<'_, sqlx::MySql>,
    robot_part_id: i64,
) -> Result<Option<RobotPartRecord>, sqlx::Error> {
    let row = sqlx::query(
        "SELECT id, typeId, tierId, partName, orePriceId, oreCapacity, miningCapacity, \
                batteryCapacity, memoryCapacity, cpuCapacity, forwardCapacity, backwardCapacity, \
                rotateCapacity, rechargeTime, scanTime, scanDistance, weight, volume, powerUsage \
         FROM RobotPart \
         WHERE id = ?",
    )
    .bind(robot_part_id)
    .fetch_optional(&mut **transaction)
    .await?;

    row.map(robot_part_record).transpose()
}

pub(crate) fn valid_robot_name(robot_name: &str) -> bool {
    !robot_name.is_empty()
        && robot_name.len() <= 15
        && robot_name
            .bytes()
            .all(|byte| byte.is_ascii_alphanumeric() || byte == b'_')
}
