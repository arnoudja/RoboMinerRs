use robominer_db::{MiningRallyQueueRecord, MySqlPool, RobotPartRecord, RobotRecord};
use robominer_sim::MAX_ORE_TYPES;

use crate::constants::{RALLY_EXPIRY_START_SECONDS, RALLY_SIZE};
use crate::error::{DomainError, RobotPartSlot};

use super::types::{
    MiningAreaLoadout, PoolItemLoadout, PoolLoadout, RallyLoadout, RallyQueueEntry, RobotLoadout,
    RobotLoadoutParts,
};

pub fn mining_rally_queue_is_ready(queue_rows: &[MiningRallyQueueRecord]) -> bool {
    !queue_rows.is_empty()
        && (queue_rows.len() >= RALLY_SIZE
            || queue_rows[0].seconds_left < RALLY_EXPIRY_START_SECONDS)
}

pub async fn load_robot_loadout(
    pool: &MySqlPool,
    robot_id: i64,
) -> Result<Option<RobotLoadout>, DomainError> {
    let Some(robot) = robominer_db::get_robot(pool, robot_id).await? else {
        return Ok(None);
    };

    let parts = load_robot_parts(pool, &robot).await?;

    Ok(Some(RobotLoadout::new(robot, parts)))
}

pub async fn load_mining_area_loadout(
    pool: &MySqlPool,
    mining_area_id: i64,
) -> Result<Option<MiningAreaLoadout>, DomainError> {
    let Some(area) = robominer_db::get_mining_area(pool, mining_area_id).await? else {
        return Ok(None);
    };

    let ore_supplies = robominer_db::list_mining_area_ore_supplies(pool, area.id).await?;
    let ai_robot = load_robot_loadout(pool, area.ai_robot_id).await?.ok_or(
        DomainError::ReferencedAiRobotMissing {
            mining_area_id: area.id,
            robot_id: area.ai_robot_id,
        },
    )?;

    Ok(Some(MiningAreaLoadout::new(area, ore_supplies, ai_robot)))
}

pub async fn load_next_rally_loadout(
    pool: &MySqlPool,
    mining_area_id: i64,
) -> Result<Option<RallyLoadout>, DomainError> {
    let Some(mining_area) = load_mining_area_loadout(pool, mining_area_id).await? else {
        return Ok(None);
    };

    let queue_rows =
        robominer_db::list_next_mining_rally_queue_for_area(pool, mining_area_id).await?;

    if !mining_rally_queue_is_ready(&queue_rows) {
        return Ok(None);
    }

    let mut queue_entries = Vec::with_capacity(queue_rows.len());
    let ore_type_ids = area_ore_type_ids(&mining_area.ore_supplies);

    for queue in queue_rows {
        let robot = load_robot_loadout(pool, queue.queue.robot_id)
            .await?
            .ok_or(DomainError::ReferencedQueueRobotMissing {
                mining_queue_id: queue.queue.id,
                robot_id: queue.queue.robot_id,
            })?;

        let user_caps =
            robominer_db::list_user_depot_max_allowed(pool, robot.robot.user_id).await?;
        let robot = robot.with_depot_capacity(depot_capacity_for_ore_types(
            &ore_type_ids,
            &user_caps,
        ));

        queue_entries.push(RallyQueueEntry::new(queue, robot));
    }

    Ok(Some(RallyLoadout::new(mining_area, queue_entries)))
}

pub async fn load_pool_loadout(
    pool: &MySqlPool,
    pool_id: i64,
) -> Result<Option<PoolLoadout>, DomainError> {
    let Some(pool_record) = robominer_db::get_pool(pool, pool_id).await? else {
        return Ok(None);
    };

    let mining_area = load_mining_area_loadout(pool, pool_record.mining_area_id)
        .await?
        .ok_or(DomainError::ReferencedPoolMiningAreaMissing {
            pool_id: pool_record.id,
            mining_area_id: pool_record.mining_area_id,
        })?;
    let item_rows = robominer_db::list_pool_items(pool, pool_record.id).await?;
    let mut items = Vec::with_capacity(item_rows.len());

    for item in item_rows {
        let robot = load_robot_loadout(pool, item.robot_id).await?.ok_or(
            DomainError::ReferencedPoolRobotMissing {
                pool_item_id: item.id,
                robot_id: item.robot_id,
            },
        )?;

        items.push(PoolItemLoadout::new(item, robot));
    }

    Ok(Some(PoolLoadout::new(pool_record, mining_area, items)))
}

pub async fn load_next_pool_rally_loadout(
    pool: &MySqlPool,
    pool_id: i64,
) -> Result<Option<PoolLoadout>, DomainError> {
    let Some(pool_record) = robominer_db::get_pool(pool, pool_id).await? else {
        return Ok(None);
    };

    let mining_area = load_mining_area_loadout(pool, pool_record.mining_area_id)
        .await?
        .ok_or(DomainError::ReferencedPoolMiningAreaMissing {
            pool_id: pool_record.id,
            mining_area_id: pool_record.mining_area_id,
        })?;
    let item_rows = robominer_db::list_next_pool_rally_items(pool, pool_record.id).await?;
    let mut items = Vec::with_capacity(item_rows.len());

    for item in item_rows {
        let robot = load_robot_loadout(pool, item.robot_id).await?.ok_or(
            DomainError::ReferencedPoolRobotMissing {
                pool_item_id: item.id,
                robot_id: item.robot_id,
            },
        )?;

        items.push(PoolItemLoadout::new(item, robot));
    }

    Ok(Some(PoolLoadout::new(pool_record, mining_area, items)))
}

async fn load_robot_parts(
    pool: &MySqlPool,
    robot: &RobotRecord,
) -> Result<RobotLoadoutParts, DomainError> {
    Ok(RobotLoadoutParts {
        ore_container: load_optional_robot_part(
            pool,
            robot.id,
            RobotPartSlot::OreContainer,
            robot.ore_container_id,
        )
        .await?,
        mining_unit: load_optional_robot_part(
            pool,
            robot.id,
            RobotPartSlot::MiningUnit,
            robot.mining_unit_id,
        )
        .await?,
        battery: load_optional_robot_part(pool, robot.id, RobotPartSlot::Battery, robot.battery_id)
            .await?,
        memory_module: load_optional_robot_part(
            pool,
            robot.id,
            RobotPartSlot::MemoryModule,
            robot.memory_module_id,
        )
        .await?,
        cpu: load_optional_robot_part(pool, robot.id, RobotPartSlot::Cpu, robot.cpu_id).await?,
        engine: load_optional_robot_part(pool, robot.id, RobotPartSlot::Engine, robot.engine_id)
            .await?,
        ore_scanner: load_optional_robot_part(
            pool,
            robot.id,
            RobotPartSlot::OreScanner,
            robot.ore_scanner_id,
        )
        .await?,
    })
}

async fn load_optional_robot_part(
    pool: &MySqlPool,
    robot_id: i64,
    slot: RobotPartSlot,
    part_id: Option<i64>,
) -> Result<Option<RobotPartRecord>, DomainError> {
    let Some(part_id) = part_id else {
        return Ok(None);
    };

    robominer_db::get_robot_part(pool, part_id)
        .await?
        .ok_or(DomainError::ReferencedRobotPartMissing {
            robot_id,
            slot,
            part_id,
        })
        .map(Some)
}

/// Ore-type slot order matching rally/sim `legacy_ore_ids` (unique ore ids, highest first).
fn area_ore_type_ids(ore_supplies: &[robominer_db::MiningAreaOreSupplyRecord]) -> Vec<i64> {
    let mut supplies = ore_supplies.to_vec();
    supplies.sort_by(|left, right| {
        right
            .ore_id
            .cmp(&left.ore_id)
            .then_with(|| left.id.cmp(&right.id))
    });

    let mut ids = Vec::new();
    for supply in &supplies {
        if !ids.contains(&supply.ore_id) {
            ids.push(supply.ore_id);
        }
    }
    ids
}

fn depot_capacity_for_ore_types(ore_type_ids: &[i64], user_caps: &[(i64, i32)]) -> [i32; MAX_ORE_TYPES] {
    let mut capacity = [0; MAX_ORE_TYPES];
    for (slot, ore_id) in ore_type_ids.iter().enumerate().take(MAX_ORE_TYPES) {
        capacity[slot] = user_caps
            .iter()
            .find(|(id, _)| id == ore_id)
            .map(|(_, cap)| *cap)
            .unwrap_or(0);
    }
    capacity
}
