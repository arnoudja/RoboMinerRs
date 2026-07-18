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
fn animation_data_uses_versioned_json_payload_shape() {
    let program = seeded_program("mine();");
    let mut ground = Ground::new(4, 4);
    ground.at_mut(0, 0).add_ore(0, 8);

    let mut spec = RobotSpec::test_robot();
    spec.max_turns = 1;
    spec.mining_speed = 4;

    let mut simulation = Simulation::new(
        ground,
        1,
        vec![ScriptedRobot::from_executable_program(spec, &program)],
    );
    let data = simulation.run_with_animation(&[OreAnimationData {
        ore_id: 1,
        max_amount: 8,
    }]);

    let payload: serde_json::Value =
        serde_json::from_str(&data).expect("animation payload should be JSON");
    assert_eq!(payload["v"], 1);
    assert_eq!(payload["robots"]["robot"][0]["robotnr"], 0);
    assert_eq!(payload["robots"]["robot"][0]["locations"][0]["l"], 1);
    assert_eq!(payload["robots"]["robot"][0]["locations"][1]["A"], 4);
    assert_eq!(payload["robots"]["robot"][0]["locations"][1]["a"], 6);
    assert_eq!(payload["robots"]["robot"][0]["locations"][1]["l"], 1);
    assert_eq!(payload["ground"]["sizeX"], 4);
    assert_eq!(payload["ground"]["sizeY"], 4);
    assert_eq!(payload["ground"]["positions"][0]["x"], 0);
    assert_eq!(payload["ground"]["positions"][0]["c"][0]["A"], 8);
    assert_eq!(payload["ground"]["positions"][0]["c"][1]["t"], 1);
    assert_eq!(payload["ground"]["positions"][0]["c"][1]["A"], 4);
    assert_eq!(payload["oreTypes"]["A"]["id"], 1);
    assert_eq!(payload["oreTypes"]["A"]["max"], 8);
    assert!(!data.contains('<'));
}

#[test]
fn animation_data_records_wait_action_index_on_idle_cycles() {
    let ground = Ground::new(4, 4);
    let mut spec = RobotSpec::test_robot();
    spec.max_turns = 2;

    let mut simulation = Simulation::new(
        ground,
        2,
        vec![ScriptedRobot::new(
            spec,
            vec![RobotAction::Wait, RobotAction::Wait],
        )],
    );
    let data = simulation.run_with_animation(&[]);

    assert!(
        data.contains(r#""a":1"#),
        "wait cycles should emit action index 1: {data}"
    );
}

#[test]
fn animation_data_records_scan_action_index_while_scanning() {
    let source = "scan(); if (oreDistance() < 0) { rotate(0); } mine();";
    let program = seeded_program(source);

    let mut ground = Ground::new(10, 10);
    ground.add_ore_heap(4, 4, 0, 2, 2);

    let mut spec = RobotSpec::test_robot();
    spec.cpu_speed = 1;
    spec.scan_time = 6;
    spec.scan_distance = 50;
    spec.max_turns = 4;

    let mut simulation = Simulation::new(
        ground,
        4,
        vec![ScriptedRobot::from_executable_program(spec, &program)],
    );
    let data = simulation.run_with_animation(&[]);

    assert!(
        data.contains(r#""a":0"#),
        "scan-busy wait cycles should emit action index 0: {data}"
    );
    let scan_marks = data.matches(r#""a":0"#).count();
    assert!(
        scan_marks >= 2,
        "expected multiple scan-busy cycles, found {scan_marks} in {data}"
    );
}

#[test]
fn animation_data_records_source_line_for_program_actions() {
    let program = seeded_program("scan();\nmine();");
    let mut ground = Ground::new(4, 4);
    ground.at_mut(0, 0).add_ore(0, 8);

    let mut spec = RobotSpec::test_robot();
    spec.max_turns = 8;
    spec.scan_time = 2;
    spec.cpu_speed = 72;
    spec.mining_speed = 4;

    let mut simulation = Simulation::new(
        ground,
        8,
        vec![ScriptedRobot::from_executable_program(spec, &program)],
    );
    let data = simulation.run_with_animation(&[]);

    assert!(
        data.contains(r#""l":1"#) || data.contains(r#""l":2"#),
        "program animation should include source lines: {data}"
    );
    assert!(
        data.contains(r#""a":6"#) && data.contains(r#""l":"#),
        "mine cycles should include a source line: {data}"
    );
    assert!(
        !data.contains("src:"),
        "program source must not be embedded in shared animation data: {data}"
    );
}
