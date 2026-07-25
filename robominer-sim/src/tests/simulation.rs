use super::helpers::*;
use crate::*;

#[test]
fn mines_ore_using_legacy_distribution_rules() {
    let mut ground = Ground::new(5, 5);
    ground.at_mut(0, 0).add_ore(0, 10);
    ground.at_mut(0, 0).add_ore(1, 6);

    let mut spec = RobotSpec::test_robot();
    spec.mining_speed = 5;
    spec.max_turns = 1;

    let mut simulation = Simulation::new(
        ground,
        1,
        vec![ScriptedRobot::new(spec, vec![RobotAction::Mine])],
    );

    simulation.run();

    assert_eq!(simulation.robot(0).ore_at(0), 3);
    assert_eq!(simulation.robot(0).ore_at(1), 2);
    assert_eq!(simulation.robot(0).last_mined(), 5);
    assert_eq!(simulation.ground().at(0, 0).ore_at(0), 7);
    assert_eq!(simulation.ground().at(0, 0).ore_at(1), 4);
}

#[test]
fn dump_all_returns_carried_ore_to_current_ground_unit() {
    let mut ground = Ground::new(5, 5);
    ground.at_mut(0, 0).add_ore(0, 10);
    ground.at_mut(0, 0).add_ore(1, 6);

    let mut spec = RobotSpec::test_robot();
    spec.mining_speed = 5;
    spec.max_turns = 2;

    let mut simulation = Simulation::new(
        ground,
        2,
        vec![ScriptedRobot::new(
            spec,
            vec![RobotAction::Mine, RobotAction::DumpAll],
        )],
    );

    simulation.run();

    assert_eq!(simulation.robot(0).total_ore(), 0);
    assert_eq!(simulation.ground().at(0, 0).ore_at(0), 10);
    assert_eq!(simulation.ground().at(0, 0).ore_at(1), 6);
}

#[test]
fn score_matches_legacy_ore_tiers() {
    let ore = ore_amounts(&[(0, 35), (1, 100), (2, 500)]);

    assert_close(calculate_score(ore), 999.99);
}

#[test]
fn ore_heap_matches_legacy_radial_shape() {
    let mut ground = Ground::new(5, 5);

    ground.add_ore_heap(2, 2, 0, 10, 2);

    assert_eq!(ground.at(2, 2).ore_at(0), 10);
    assert_eq!(ground.at(1, 2).ore_at(0), 5);
    assert_eq!(ground.at(2, 1).ore_at(0), 5);
    assert_eq!(ground.at(0, 2).ore_at(0), 0);
    assert_eq!(ground.at(1, 1).ore_at(0), 3);
}

#[test]
fn dump_at_home_then_while_mine_remines_overflow_but_cargo_is_not_full() {
    // Fill exactly max_ore from the spawn cell, bank some into the depot, remine overflow.
    // Cargo ends below max_ore: the depot kept part of the load. That is expected, not a bug.
    // Trailing while(true); prevents program restart from dumping the remined cargo again.
    let program = seeded_program("while (mine()); dump(); while (mine()); while (true);");
    let mut ground = Ground::new(6, 6);
    ground.at_mut(0, 0).add_ore(0, 20);

    let mut spec = RobotSpec::test_robot();
    spec.max_ore = 20;
    spec.mining_speed = 5;
    spec.max_turns = 80;
    spec.cpu_speed = 72;

    let mut capacity = [0; MAX_ORE_TYPES];
    capacity[0] = 5;

    let mut simulation = Simulation::new(
        ground,
        80,
        vec![ScriptedRobot::from_executable_program(spec, &program).with_depot_capacity(capacity)],
    );

    simulation.run();

    let center = simulation.robot(0).center_position();
    let cx = center.x as usize;
    let cy = center.y as usize;
    let ground_left = simulation.ground().at(cx, cy).ore_at(0);
    let cargo = simulation.robot(0).total_ore();
    let depot = simulation.robot(0).depot()[0];

    assert_eq!(depot, 5, "depot should take capacity first");
    assert_eq!(
        ground_left, 0,
        "center cell should be emptied by while(mine()), left {ground_left}"
    );
    assert_eq!(
        cargo, 15,
        "remined overflow is max_ore minus depot deposit; cargo is intentionally not full"
    );
}

#[test]
fn dump_at_home_adjacent_heap_ore_survives_while_mine_on_center() {
    // Neighboring heap cells stay on the map; mine() only digs the center cell.
    let program = seeded_program("while (mine()); dump(); while (mine()); while (true);");
    let mut ground = Ground::new(8, 8);
    ground.at_mut(0, 0).add_ore(0, 20);
    ground.at_mut(1, 0).add_ore(0, 12);
    ground.at_mut(0, 1).add_ore(0, 8);

    let mut spec = RobotSpec::test_robot();
    spec.max_ore = 20;
    spec.mining_speed = 5;
    spec.max_turns = 80;
    spec.cpu_speed = 72;

    let mut capacity = [0; MAX_ORE_TYPES];
    capacity[0] = 5;

    let mut simulation = Simulation::new(
        ground,
        80,
        vec![ScriptedRobot::from_executable_program(spec, &program).with_depot_capacity(capacity)],
    );
    simulation.run();

    let center = simulation.robot(0).center_position();
    let cx = center.x as usize;
    let cy = center.y as usize;
    assert_eq!(simulation.ground().at(cx, cy).ore_at(0), 0);
    assert_eq!(simulation.ground().at(1, 0).ore_at(0), 12);
    assert_eq!(simulation.ground().at(0, 1).ore_at(0), 8);
    assert_eq!(simulation.robot(0).total_ore(), 15);
    assert!(simulation.robot(0).total_ore() < 20);
}

#[test]
fn scan_from_spawn_corner_misses_side_adjacent_ore() {
    // Robot 0 spawns facing 45°. Ore on (1,0)/(0,1) is visible on the map but not on that ray
    // once the center cell is empty — matching "ore on map, scanner sees nothing".
    let mut ground = Ground::new(8, 8);
    ground.at_mut(1, 0).add_ore(0, 12);
    ground.at_mut(0, 1).add_ore(0, 8);

    let origin = Position::new(0.5, 0.5, 45);
    let result = ground.scan_ore(origin, 0.0, 5, &[1, 2, 3]);
    assert!(
        result.ore_type == 0.0 && result.distance < 0.0,
        "diagonal spawn scan should miss side-adjacent ore, got {result:?}"
    );

    // Standing on overflow underfoot is still visible at distance 0.
    ground.at_mut(0, 0).add_ore(0, 7);
    let underfoot = ground.scan_ore(origin, 0.0, 5, &[1, 2, 3]);
    assert_eq!(underfoot.ore_type, 1.0);
    assert_eq!(underfoot.distance, 0.0);
}
