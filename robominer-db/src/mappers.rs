use sqlx::Row;
use sqlx::mysql::MySqlRow;

use crate::types::*;

pub(crate) type ProgramSourceRow = (i64, i64, String, Option<String>, bool, i32, Option<String>);
pub(crate) type MiningAreaRow = (i64, String, i64, i32, i32, i32, i32, i32, i64);
pub(crate) type PoolRow = (i64, i64, i32);
pub(crate) type PoolItemRow = (i64, i64, i64, String, f64, i32);
pub(crate) type MiningRallyQueueRow = (
    i64,
    i64,
    i64,
    i64,
    Option<i64>,
    Option<i32>,
    Option<f64>,
    bool,
    i32,
);

pub(crate) fn program_source_rows(rows: Vec<ProgramSourceRow>) -> Vec<ProgramSourceRecord> {
    rows.into_iter().map(program_source_record).collect()
}

pub(crate) fn program_source_record(
    (id, user_id, source_name, source_code, verified, compiled_size, error_description): ProgramSourceRow,
) -> ProgramSourceRecord {
    ProgramSourceRecord {
        id,
        user_id,
        source_name,
        source_code,
        verified,
        compiled_size,
        error_description: error_description.unwrap_or_default(),
    }
}

pub(crate) fn program_source_state_record(
    row: MySqlRow,
) -> Result<ProgramSourceStateRecord, sqlx::Error> {
    Ok(ProgramSourceStateRecord {
        source: ProgramSourceRecord {
            id: row.try_get("id")?,
            user_id: row.try_get("userId")?,
            source_name: row.try_get("sourceName")?,
            source_code: row.try_get("sourceCode")?,
            verified: row.try_get("verified")?,
            compiled_size: row.try_get("compiledSize")?,
            error_description: row
                .try_get::<Option<String>, _>("errorDescription")?
                .unwrap_or_default(),
        },
        linked_robot_count: row.try_get("linkedRobotCount")?,
    })
}

pub(crate) fn robot_part_record(row: MySqlRow) -> Result<RobotPartRecord, sqlx::Error> {
    Ok(RobotPartRecord {
        id: row.try_get("id")?,
        type_id: row.try_get("typeId")?,
        tier_id: row.try_get("tierId")?,
        part_name: row.try_get("partName")?,
        ore_price_id: row.try_get("orePriceId")?,
        ore_capacity: row.try_get("oreCapacity")?,
        mining_capacity: row.try_get("miningCapacity")?,
        battery_capacity: row.try_get("batteryCapacity")?,
        memory_capacity: row.try_get("memoryCapacity")?,
        cpu_capacity: row.try_get("cpuCapacity")?,
        forward_capacity: row.try_get("forwardCapacity")?,
        backward_capacity: row.try_get("backwardCapacity")?,
        rotate_capacity: row.try_get("rotateCapacity")?,
        recharge_time: row.try_get("rechargeTime")?,
        scan_time: row.try_get("scanTime")?,
        scan_distance: row.try_get("scanDistance")?,
        weight: row.try_get("weight")?,
        volume: row.try_get("volume")?,
        power_usage: row.try_get("powerUsage")?,
    })
}

pub(crate) fn robot_record(row: MySqlRow) -> Result<RobotRecord, sqlx::Error> {
    Ok(RobotRecord {
        id: row.try_get("id")?,
        user_id: row.try_get("userId")?,
        robot_name: row.try_get("robotName")?,
        source_code: row.try_get("sourceCode")?,
        program_source_id: row.try_get("programSourceId")?,
        ore_container_id: row.try_get("oreContainerId")?,
        mining_unit_id: row.try_get("miningUnitId")?,
        battery_id: row.try_get("batteryId")?,
        memory_module_id: row.try_get("memoryModuleId")?,
        cpu_id: row.try_get("cpuId")?,
        engine_id: row.try_get("engineId")?,
        ore_scanner_id: row.try_get("oreScannerId")?,
        recharge_time: row.try_get("rechargeTime")?,
        max_ore: row.try_get("maxOre")?,
        mining_speed: row.try_get("miningSpeed")?,
        max_turns: row.try_get("maxTurns")?,
        memory_size: row.try_get("memorySize")?,
        cpu_speed: row.try_get("cpuSpeed")?,
        forward_speed: row.try_get("forwardSpeed")?,
        backward_speed: row.try_get("backwardSpeed")?,
        rotate_speed: row.try_get("rotateSpeed")?,
        robot_size: row.try_get("robotSize")?,
        scan_time: row.try_get("scanTime")?,
        scan_distance: row.try_get("scanDistance")?,
        total_mining_runs: row.try_get("totalMiningRuns")?,
    })
}

pub(crate) fn robot_config_state_record(
    row: MySqlRow,
) -> Result<RobotConfigStateRecord, sqlx::Error> {
    Ok(RobotConfigStateRecord {
        robot_id: row.try_get("robotId")?,
        robot_name: row.try_get("robotName")?,
        program_source_id: row.try_get("programSourceId")?,
        ore_container_id: row.try_get("oreContainerId")?,
        ore_container_name: row.try_get("oreContainerName")?,
        mining_unit_id: row.try_get("miningUnitId")?,
        mining_unit_name: row.try_get("miningUnitName")?,
        battery_id: row.try_get("batteryId")?,
        battery_name: row.try_get("batteryName")?,
        memory_module_id: row.try_get("memoryModuleId")?,
        memory_module_name: row.try_get("memoryModuleName")?,
        cpu_id: row.try_get("cpuId")?,
        cpu_name: row.try_get("cpuName")?,
        engine_id: row.try_get("engineId")?,
        engine_name: row.try_get("engineName")?,
        ore_scanner_id: row.try_get("oreScannerId")?,
        ore_scanner_name: row.try_get("oreScannerName")?,
        recharge_time: row.try_get("rechargeTime")?,
        max_ore: row.try_get("maxOre")?,
        mining_speed: row.try_get("miningSpeed")?,
        max_turns: row.try_get("maxTurns")?,
        memory_size: row.try_get("memorySize")?,
        cpu_speed: row.try_get("cpuSpeed")?,
        forward_speed: row.try_get("forwardSpeed")?,
        backward_speed: row.try_get("backwardSpeed")?,
        rotate_speed: row.try_get("rotateSpeed")?,
        robot_size: row.try_get("robotSize")?,
        scan_time: row.try_get("scanTime")?,
        scan_distance: row.try_get("scanDistance")?,
        change_pending: row.try_get("changePending")?,
    })
}

pub(crate) fn shop_robot_part_catalog_record(
    row: MySqlRow,
) -> Result<ShopRobotPartCatalogRecord, sqlx::Error> {
    Ok(ShopRobotPartCatalogRecord {
        robot_part_id: row.try_get("id")?,
        type_id: row.try_get("typeId")?,
        tier_id: row.try_get("tierId")?,
        tier_name: row.try_get("oreName")?,
        part_name: row.try_get("partName")?,
        ore_capacity: row.try_get("oreCapacity")?,
        mining_capacity: row.try_get("miningCapacity")?,
        battery_capacity: row.try_get("batteryCapacity")?,
        memory_capacity: row.try_get("memoryCapacity")?,
        cpu_capacity: row.try_get("cpuCapacity")?,
        forward_capacity: row.try_get("forwardCapacity")?,
        backward_capacity: row.try_get("backwardCapacity")?,
        rotate_capacity: row.try_get("rotateCapacity")?,
        recharge_time: row.try_get("rechargeTime")?,
        scan_time: row.try_get("scanTime")?,
        scan_distance: row.try_get("scanDistance")?,
        weight: row.try_get("weight")?,
        volume: row.try_get("volume")?,
        power_usage: row.try_get("powerUsage")?,
    })
}

pub(crate) fn mining_area_rows(rows: Vec<MiningAreaRow>) -> Vec<MiningAreaRecord> {
    rows.into_iter().map(mining_area_record).collect()
}

pub(crate) fn mining_area_record(
    (
        id,
        area_name,
        ore_price_id,
        size_x,
        size_y,
        max_moves,
        mining_time,
        tax_rate,
        ai_robot_id,
    ): MiningAreaRow,
) -> MiningAreaRecord {
    MiningAreaRecord {
        id,
        area_name,
        ore_price_id,
        size_x,
        size_y,
        max_moves,
        mining_time,
        tax_rate,
        ai_robot_id,
    }
}

pub(crate) fn mining_rally_queue_rows(
    rows: Vec<MiningRallyQueueRow>,
) -> Vec<MiningRallyQueueRecord> {
    let mut seen_users = Vec::new();

    rows.into_iter()
        .filter(|row| {
            if seen_users.contains(&row.3) {
                false
            } else {
                seen_users.push(row.3);
                true
            }
        })
        .take(4)
        .map(mining_rally_queue_record)
        .collect()
}

fn mining_rally_queue_record(
    (
        id,
        mining_area_id,
        robot_id,
        user_id,
        rally_result_id,
        player_number,
        score,
        claimed,
        seconds_left,
    ): MiningRallyQueueRow,
) -> MiningRallyQueueRecord {
    MiningRallyQueueRecord {
        queue: MiningQueueRecord {
            id,
            mining_area_id,
            robot_id,
            rally_result_id,
            player_number,
            score,
            claimed,
        },
        user_id,
        seconds_left,
    }
}

pub(crate) fn pool_record((id, mining_area_id, required_runs): PoolRow) -> PoolRecord {
    PoolRecord {
        id,
        mining_area_id,
        required_runs,
    }
}

pub(crate) fn pool_item_rows(rows: Vec<PoolItemRow>) -> Vec<PoolItemRecord> {
    rows.into_iter().map(pool_item_record).collect()
}

pub(crate) fn next_pool_rally_item_rows(rows: Vec<PoolItemRow>) -> Vec<PoolItemRecord> {
    let first_runs_done = rows.first().map(|row| row.5);

    rows.into_iter()
        .filter(|row| Some(row.5) == first_runs_done)
        .map(pool_item_record)
        .collect()
}

fn pool_item_record(
    (id, pool_id, robot_id, source_code, total_score, runs_done): PoolItemRow,
) -> PoolItemRecord {
    PoolItemRecord {
        id,
        pool_id,
        robot_id,
        source_code,
        total_score,
        runs_done,
    }
}
