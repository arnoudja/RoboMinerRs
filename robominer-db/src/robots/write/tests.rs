use super::helpers::{generated_robot_name, robot_part_baseline, valid_robot_name};
use super::super::parameters::robot_parameters_for_parts;
use super::super::{PendingRobotUpdateState, RequestedRobotParts, RobotUpdateState};
use crate::RobotPartRecord;

fn scaled_robot_part(scale: i32) -> RobotPartRecord {
    RobotPartRecord {
        id: 1,
        type_id: 1,
        tier_id: Some(1),
        part_name: "scaled".to_string(),
        ore_price_id: 1,
        ore_capacity: 2 * scale,
        mining_capacity: 3 * scale,
        battery_capacity: 10 * scale,
        memory_capacity: 4 * scale,
        cpu_capacity: 5 * scale,
        forward_capacity: 6 * scale,
        backward_capacity: 3 * scale,
        rotate_capacity: 2 * scale,
        recharge_time: scale,
        scan_time: 0,
        scan_distance: 0,
        weight: 2 * scale,
        volume: 8 * scale,
        power_usage: scale,
    }
}

fn scaled_robot_parts(scale: i32) -> RequestedRobotParts {
    let part = scaled_robot_part(scale);
    RequestedRobotParts {
        ore_container: part.clone(),
        mining_unit: part.clone(),
        battery: part.clone(),
        memory_module: part.clone(),
        cpu: part.clone(),
        engine: part.clone(),
        ore_scanner: part.clone(),
    }
}

fn sample_robot_state(part_ids: [i64; 7]) -> RobotUpdateState {
    RobotUpdateState {
        id: 1,
        user_id: 2,
        source_code: "mine();".to_string(),
        ore_container_id: Some(part_ids[0]),
        mining_unit_id: Some(part_ids[1]),
        battery_id: Some(part_ids[2]),
        memory_module_id: Some(part_ids[3]),
        cpu_id: Some(part_ids[4]),
        engine_id: Some(part_ids[5]),
        ore_scanner_id: Some(part_ids[6]),
    }
}

#[test]
fn robot_parameters_match_scaled_default_parts() {
    let parameters = robot_parameters_for_parts(&scaled_robot_parts(2))
        .expect("scaled parts should produce parameters");

    assert_eq!(parameters.recharge_time, 14);
    assert_eq!(parameters.max_ore, 28);
    assert_eq!(parameters.mining_speed, 42);
    assert_eq!(parameters.max_turns, 10);
    assert_eq!(parameters.memory_size, 56);
    assert_eq!(parameters.cpu_speed, 70);
    assert!((parameters.forward_speed - 9.0).abs() < f64::EPSILON);
    assert!((parameters.backward_speed - 4.5).abs() < f64::EPSILON);
    assert_eq!(parameters.rotate_speed, 20);
}

#[test]
fn robot_parameters_reject_invalid_part_totals() {
    let mut parts = scaled_robot_parts(2);
    for slot in [
        &mut parts.ore_container,
        &mut parts.mining_unit,
        &mut parts.battery,
        &mut parts.memory_module,
        &mut parts.cpu,
        &mut parts.engine,
        &mut parts.ore_scanner,
    ] {
        slot.power_usage = 0;
    }
    assert!(robot_parameters_for_parts(&parts).is_none());

    parts = scaled_robot_parts(2);
    for slot in [
        &mut parts.ore_container,
        &mut parts.mining_unit,
        &mut parts.battery,
        &mut parts.memory_module,
        &mut parts.cpu,
        &mut parts.engine,
        &mut parts.ore_scanner,
    ] {
        slot.weight = 0;
    }
    assert!(robot_parameters_for_parts(&parts).is_none());

    parts = scaled_robot_parts(2);
    for slot in [
        &mut parts.ore_container,
        &mut parts.mining_unit,
        &mut parts.battery,
        &mut parts.memory_module,
        &mut parts.cpu,
        &mut parts.engine,
        &mut parts.ore_scanner,
    ] {
        slot.volume = -1;
    }
    assert!(robot_parameters_for_parts(&parts).is_none());
}

#[test]
fn valid_robot_name_enforces_length_and_charset() {
    assert!(valid_robot_name("valid_name1"));
    assert!(valid_robot_name(&"a".repeat(15)));
    assert!(!valid_robot_name(""));
    assert!(!valid_robot_name(&"a".repeat(16)));
    assert!(!valid_robot_name("bad-name"));
    assert!(!valid_robot_name("café"));
}

#[test]
fn generated_robot_name_truncates_long_usernames() {
    assert_eq!(generated_robot_name("short", 3), "short_3");
    assert_eq!(
        generated_robot_name("verylongusername", 4),
        "verylongus_4"
    );
}

#[test]
fn robot_part_baseline_prefers_pending_part_ids() {
    let robot = sample_robot_state([101, 201, 301, 401, 501, 601, 701]);
    assert_eq!(
        robot_part_baseline(&robot, None),
        [
            Some(101),
            Some(201),
            Some(301),
            Some(401),
            Some(501),
            Some(601),
            Some(701),
        ]
    );

    let pending = PendingRobotUpdateState {
        source_code: "mine();".to_string(),
        ore_container_id: Some(111),
        mining_unit_id: Some(211),
        battery_id: Some(311),
        memory_module_id: Some(411),
        cpu_id: Some(511),
        engine_id: Some(611),
        ore_scanner_id: Some(711),
    };
    assert_eq!(
        robot_part_baseline(&robot, Some(&pending)),
        [
            Some(111),
            Some(211),
            Some(311),
            Some(411),
            Some(511),
            Some(611),
            Some(711),
        ]
    );
}
