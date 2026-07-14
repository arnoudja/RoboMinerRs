use crate::error::DomainError;
use crate::loadout::{
    MiningAreaLoadout, RallyLoadout, RallyQueueEntry, RobotLoadout, RobotLoadoutParts,
    mining_area_to_ground, mining_rally_queue_is_ready, robot_record_to_spec,
};
use robominer_test_support::{
    mining_rally_queue_record, ore_supply_record, robot_part, robot_record,
    unit_test_mining_area_record,
};

#[test]
fn converts_robot_record_to_simulator_spec() {
    let robot = robot_record(42);

    let spec = robot_record_to_spec(&robot).expect("record should convert");

    assert_eq!(spec.robot_id, 42);
    assert_eq!(spec.max_turns, 600);
    assert_eq!(spec.max_ore, 75);
    assert_eq!(spec.mining_speed, 9);
    assert_eq!(spec.cpu_speed, 3);
    assert_eq!(spec.forward_speed, 1.25);
    assert_eq!(spec.backward_speed, 0.75);
    assert_eq!(spec.rotate_speed, 30);
    assert_eq!(spec.robot_size, 1.5);
}

#[test]
fn robot_loadout_carries_parts_and_builds_spec() {
    let robot = robot_record(7);
    let parts = RobotLoadoutParts {
        ore_container: Some(robot_part(101, 1)),
        mining_unit: Some(robot_part(201, 2)),
        battery: Some(robot_part(301, 3)),
        memory_module: Some(robot_part(401, 4)),
        cpu: Some(robot_part(501, 5)),
        engine: Some(robot_part(601, 6)),
        ore_scanner: Some(robot_part(701, 7)),
    };
    let loadout = RobotLoadout::new(robot, parts);

    assert_eq!(loadout.parts.ore_container.as_ref().unwrap().id, 101);
    assert_eq!(loadout.parts.engine.as_ref().unwrap().type_id, 6);
    assert_eq!(loadout.simulator_spec().unwrap().robot_id, 7);
}

#[test]
fn missing_parts_are_valid_for_incomplete_robot_loadouts() {
    let loadout = RobotLoadout::new(robot_record(7), RobotLoadoutParts::empty());

    assert!(loadout.parts.ore_container.is_none());
    assert_eq!(loadout.simulator_spec().unwrap().robot_id, 7);
}

#[test]
fn rejects_robot_ids_that_do_not_fit_simulator_specs() {
    let error = robot_record_to_spec(&robot_record(i64::from(i32::MAX) + 1)).unwrap_err();

    assert!(matches!(error, DomainError::RobotIdOutOfRange(_)));
}

#[test]
fn mining_area_loadout_builds_seeded_simulator_ground() {
    let loadout = MiningAreaLoadout::new(
        unit_test_mining_area_record(1001),
        vec![ore_supply_record(1, 1001, 2, 10, 2)],
        RobotLoadout::new(robot_record(1), RobotLoadoutParts::empty()),
    );

    let ground = loadout
        .simulator_ground_with_seed(0)
        .expect("ground should build");

    assert_eq!(ground.size_x(), 4);
    assert_eq!(ground.size_y(), 4);
    assert_eq!(ground.at(2, 2).ore_at(0), 10);
    assert_eq!(loadout.ai_robot.simulator_spec().unwrap().robot_id, 1);
}

#[test]
fn mining_area_ore_slots_follow_legacy_descending_ore_id_order() {
    let area = unit_test_mining_area_record(1001);
    let supplies = vec![
        ore_supply_record(1, area.id, 1, 10, 1),
        ore_supply_record(2, area.id, 3, 30, 1),
    ];

    let ground = mining_area_to_ground(&area, &supplies, 0).expect("ground should build");

    assert_eq!(ground.at(1, 1).ore_at(0), 30);
    assert_eq!(ground.at(1, 2).ore_at(1), 10);
}

#[test]
fn mining_area_ground_rejects_invalid_dimensions() {
    let mut area = unit_test_mining_area_record(1001);
    area.size_x = 1;

    let error = mining_area_to_ground(&area, &[], 0).unwrap_err();

    assert!(matches!(error, DomainError::InvalidMiningAreaSize { .. }));
}

#[test]
fn mining_area_ground_rejects_invalid_ore_supplies() {
    let area = unit_test_mining_area_record(1001);
    let supplies = vec![ore_supply_record(1, area.id, 0, 10, 1)];

    let error = mining_area_to_ground(&area, &supplies, 0).unwrap_err();

    assert!(matches!(
        error,
        DomainError::InvalidMiningAreaOreSupply { .. }
    ));
}

#[test]
fn mining_area_ground_rejects_too_many_unique_ore_types() {
    let area = unit_test_mining_area_record(1001);
    let supplies = (1..=(robominer_sim::MAX_ORE_TYPES as i64 + 1))
        .map(|ore_id| ore_supply_record(ore_id, area.id, ore_id, 10, 1))
        .collect::<Vec<_>>();

    let error = mining_area_to_ground(&area, &supplies, 0).unwrap_err();

    assert!(matches!(
        error,
        DomainError::TooManyMiningAreaOreTypes { .. }
    ));
}

#[test]
fn mining_rally_queue_readiness_matches_legacy_start_rules() {
    assert!(!mining_rally_queue_is_ready(&[]));

    let waiting_queue = vec![
        mining_rally_queue_record(1, 1001, 11, 10),
        mining_rally_queue_record(2, 1001, 12, 20),
        mining_rally_queue_record(3, 1001, 13, 30),
    ];
    assert!(!mining_rally_queue_is_ready(&waiting_queue));

    let expiring_queue = vec![
        mining_rally_queue_record(1, 1001, 11, 9),
        mining_rally_queue_record(2, 1001, 12, 20),
        mining_rally_queue_record(3, 1001, 13, 30),
    ];
    assert!(mining_rally_queue_is_ready(&expiring_queue));

    let full_queue = vec![
        mining_rally_queue_record(1, 1001, 11, 10),
        mining_rally_queue_record(2, 1001, 12, 20),
        mining_rally_queue_record(3, 1001, 13, 30),
        mining_rally_queue_record(4, 1001, 14, 40),
    ];
    assert!(mining_rally_queue_is_ready(&full_queue));
}

#[test]
fn rally_loadout_carries_mining_area_queue_entries_and_ai_fill_count() {
    let mining_area = MiningAreaLoadout::new(
        unit_test_mining_area_record(1001),
        vec![ore_supply_record(1, 1001, 1, 10, 1)],
        RobotLoadout::new(robot_record(1), RobotLoadoutParts::empty()),
    );
    let queue_entries = vec![
        RallyQueueEntry::new(
            mining_rally_queue_record(10, 1001, 11, 9),
            RobotLoadout::new(robot_record(11), RobotLoadoutParts::empty()),
        ),
        RallyQueueEntry::new(
            mining_rally_queue_record(11, 1001, 12, 20),
            RobotLoadout::new(robot_record(12), RobotLoadoutParts::empty()),
        ),
    ];

    let rally = RallyLoadout::new(mining_area, queue_entries);

    assert_eq!(rally.mining_area.area.id, 1001);
    assert_eq!(rally.queue_entries.len(), 2);
    assert_eq!(rally.queue_entries[0].queue.queue.id, 10);
    assert_eq!(
        rally.queue_entries[1]
            .robot
            .simulator_spec()
            .unwrap()
            .robot_id,
        12
    );
    assert_eq!(rally.ai_robot_count(), 2);
}
