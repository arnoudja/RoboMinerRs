//! Shared robot part builders for unit tests in this crate.

#[cfg(test)]
use robominer_db::RobotPartRecord;

#[cfg(test)]
pub fn sample_part(id: i64, type_id: i64) -> RobotPartRecord {
    part_with_options(id, type_id, Some(1), 50)
}

#[cfg(test)]
pub fn sample_part_with_tier(id: i64, type_id: i64, tier_id: i64) -> RobotPartRecord {
    part_with_options(id, type_id, Some(tier_id), 50)
}

#[cfg(test)]
pub fn sample_part_with_tier_option(
    id: i64,
    type_id: i64,
    tier_id: Option<i64>,
) -> RobotPartRecord {
    part_with_options(id, type_id, tier_id, 50)
}

#[cfg(test)]
pub fn sample_part_with_memory(id: i64, type_id: i64, memory: i32) -> RobotPartRecord {
    part_with_caps(id, type_id, memory, 2, 8, 1, 6, 3, 2)
}

#[cfg(test)]
pub fn part_with_caps(
    id: i64,
    type_id: i64,
    memory: i32,
    weight: i32,
    volume: i32,
    power_usage: i32,
    forward: i32,
    backward: i32,
    rotate: i32,
) -> RobotPartRecord {
    RobotPartRecord {
        id,
        type_id,
        tier_id: Some(1),
        part_name: format!("part-{id}"),
        ore_price_id: 1,
        ore_capacity: 2,
        mining_capacity: 2,
        battery_capacity: 20,
        memory_capacity: memory,
        cpu_capacity: 5,
        forward_capacity: forward,
        backward_capacity: backward,
        rotate_capacity: rotate,
        recharge_time: 1,
        scan_time: 1,
        scan_distance: 1,
        weight,
        volume,
        power_usage,
    }
}

#[cfg(test)]
fn part_with_options(
    id: i64,
    type_id: i64,
    tier_id: Option<i64>,
    memory_capacity: i32,
) -> RobotPartRecord {
    RobotPartRecord {
        tier_id,
        ..part_with_caps(id, type_id, memory_capacity, 2, 8, 1, 6, 3, 2)
    }
}
