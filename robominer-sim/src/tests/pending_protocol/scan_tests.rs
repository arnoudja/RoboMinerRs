use crate::*;

#[test]
fn program_bridge_scan_and_mine_in_same_cpu_loop() {
    let program = robominer_program::compile_executable_source("scan(); mine();")
        .expect("scan program should compile");
    assert!(program.requires_runtime());

    let mut spec = RobotSpec::test_robot();
    spec.cpu_speed = 72;
    spec.scan_time = 6;
    spec.scan_distance = 5;
    spec.max_turns = 1;

    let mut simulation = Simulation::new(
        Ground::new(5, 5),
        1,
        vec![ScriptedRobot::from_executable_program(spec, &program)],
    );
    simulation.prepare_test_run();
    simulation.advance_test_turn();

    assert_eq!(
        simulation.robot(0).actions_done()[ROBOT_ACTION_TYPE_SCAN],
        1
    );
    assert_eq!(simulation.robot(0).actions_done()[6], 1);
}

#[test]
fn program_bridge_extends_cpu_budget_while_awaiting_scan_result() {
    let source = "scan(); if (oreDistance() < 0) { rotate(0); } mine();";
    let program = robominer_program::compile_executable_source(source)
        .expect("scan condition program should compile");
    assert!(program.requires_runtime());

    let mut ground = Ground::new(10, 10);
    ground.add_ore_heap(4, 4, 0, 2, 2);

    let mut spec = RobotSpec::test_robot();
    spec.cpu_speed = 1;
    spec.scan_time = 6;
    spec.scan_distance = 50;
    spec.max_turns = 6;
    let max_turns = spec.max_turns;

    let mut simulation = Simulation::new(
        ground,
        max_turns,
        vec![ScriptedRobot::from_executable_program(spec, &program)],
    );
    simulation.prepare_test_run();
    simulation.advance_test_turn();

    assert_eq!(
        simulation.robot(0).actions_done()[ROBOT_ACTION_TYPE_SCAN],
        1,
        "first cycle should start the scan"
    );
    assert_eq!(
        simulation.robot(0).actions_done()[6],
        0,
        "mine should not run until the scan result is read on a later cycle"
    );
    assert!(
        simulation.program_runner(0).unwrap().pending_scan_start(),
        "scan() should still await its start result after one CPU cycle"
    );
    assert!(
        matches!(
            simulation.robot(0).scan_state,
            crate::ground::ScanState::Scanning { .. }
        ),
        "scan should still be in progress after the first one-CPU cycle"
    );

    let mut turns_until_mine = 1;
    while simulation.robot(0).actions_done()[6] == 0 && turns_until_mine < max_turns {
        simulation.advance_test_turn();
        turns_until_mine += 1;
    }

    assert_eq!(
        simulation.robot(0).actions_done()[6],
        1,
        "extended CPU budget should finish the scan read and reach mine()"
    );
    assert!(
        turns_until_mine >= 2,
        "mine should not run on the first cycle when cpu_speed=1 and scan_time=6"
    );
    assert!(
        matches!(
            simulation.robot(0).scan_state,
            crate::ground::ScanState::Complete(_)
        ),
        "scan should complete once oreDistance() is evaluated"
    );
}

#[test]
fn scan_bridge_reads_ore_type_after_scan_completes() {
    let source = "scan(); if (oreType() == 1) { mine(); }";
    let program = robominer_program::compile_executable_source(source)
        .expect("scan oreType program should compile");

    let mut ground = Ground::new(10, 10);
    ground.add_ore_heap(4, 4, 0, 2, 2);

    let mut spec = RobotSpec::test_robot();
    spec.cpu_speed = 1;
    spec.scan_time = 6;
    spec.scan_distance = 50;
    spec.max_turns = 6;
    let max_turns = spec.max_turns;

    let mut simulation = Simulation::new(
        ground,
        max_turns,
        vec![ScriptedRobot::from_executable_program(spec, &program)],
    );
    simulation.prepare_test_run();
    simulation.advance_test_turn();

    assert_eq!(
        simulation.robot(0).actions_done()[ROBOT_ACTION_TYPE_SCAN],
        1
    );
    assert_eq!(simulation.robot(0).actions_done()[6], 0);

    while simulation.robot(0).actions_done()[6] == 0 && simulation.time() < max_turns {
        simulation.advance_test_turn();
    }

    assert_eq!(
        simulation.robot(0).actions_done()[6],
        1,
        "oreType() should read scan results and reach mine()"
    );
}

#[test]
fn scan_bridge_ore_distance_moves_toward_detected_ore() {
    let source = "scan(); move(oreDistance()); mine();";
    let program = robominer_program::compile_executable_source(source)
        .expect("scan oreDistance program should compile");

    let mut ground = Ground::new(10, 10);
    ground.add_ore_heap(4, 4, 0, 2, 2);

    let mut spec = RobotSpec::test_robot();
    spec.cpu_speed = 72;
    spec.scan_time = 6;
    spec.scan_distance = 50;
    spec.forward_speed = 1.0;
    spec.max_turns = 20;
    let max_turns = spec.max_turns;

    let mut simulation = Simulation::new(
        ground,
        max_turns,
        vec![ScriptedRobot::from_executable_program(spec, &program)],
    );
    simulation.prepare_test_run();

    while simulation.robot(0).actions_done()[6] == 0 && simulation.time() < max_turns {
        simulation.advance_test_turn();
    }

    assert!(
        simulation.robot(0).actions_done()[ROBOT_ACTION_TYPE_SCAN] > 0,
        "scan should run before move(oreDistance())"
    );
    assert!(
        simulation.robot(0).actions_done()[2] > 0,
        "move(oreDistance()) should run after scan completes"
    );
    assert!(
        simulation.robot(0).actions_done()[6] > 0,
        "mine should run after moving toward detected ore"
    );
}

#[test]
fn scan_bridge_stand_on_ore_move_ore_distance_reaches_mine() {
    // Robot center starts in cell (0,0). Scanning while standing on ore yields
    // oreDistance() == 0; that must not livelock pending move state.
    let source = "scan(); move(oreDistance()); mine();";
    let program = robominer_program::compile_executable_source(source)
        .expect("stand-on-ore program should compile");

    let mut ground = Ground::new(10, 10);
    ground.at_mut(0, 0).add_ore(0, 8);

    let mut spec = RobotSpec::test_robot();
    spec.cpu_speed = 72;
    spec.scan_time = 6;
    spec.scan_distance = 50;
    spec.forward_speed = 1.0;
    spec.max_turns = 10;
    let max_turns = spec.max_turns;

    let mut simulation = Simulation::new(
        ground,
        max_turns,
        vec![ScriptedRobot::from_executable_program(spec, &program)],
    );
    simulation.prepare_test_run();

    while simulation.robot(0).actions_done()[6] == 0 && simulation.time() < max_turns {
        assert!(
            !simulation.program_runner(0).unwrap().has_pending_physical(),
            "move(oreDistance()) with distance 0 must not leave pending physical state"
        );
        simulation.advance_test_turn();
    }

    assert!(
        simulation.robot(0).actions_done()[ROBOT_ACTION_TYPE_SCAN] > 0,
        "scan should run"
    );
    assert!(
        simulation.robot(0).actions_done()[6] > 0,
        "mine should run after move(oreDistance()) returns 0 while standing on ore"
    );
    assert!(
        !simulation.program_runner(0).unwrap().has_pending_physical(),
        "runner should clear pending physical after zero-distance move"
    );
}

#[test]
fn scan_bridge_directional_scan_finds_ore_off_axis() {
    // Robot 0 spawns facing 45°; scan(45) sweeps +Y where the ore sits.
    let source = "scan(45); if (oreType() > 0) { mine(); }";
    let program = robominer_program::compile_executable_source(source)
        .expect("directional scan program should compile");

    let mut ground = Ground::new(10, 10);
    ground.at_mut(0, 4).add_ore(0, 8);

    let mut spec = RobotSpec::test_robot();
    spec.cpu_speed = 72;
    spec.scan_time = 6;
    spec.scan_distance = 10;
    spec.max_turns = 2;

    let mut simulation = Simulation::new(
        ground,
        spec.max_turns,
        vec![ScriptedRobot::from_executable_program(spec, &program)],
    );
    simulation.prepare_test_run();
    simulation.advance_test_turn();

    assert_eq!(
        simulation.robot(0).actions_done()[ROBOT_ACTION_TYPE_SCAN],
        1
    );
    assert!(
        simulation.robot(0).actions_done()[6] > 0,
        "directional scan should detect off-axis ore and mine"
    );
}
