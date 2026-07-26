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
fn program_bridge_waits_across_cycles_for_scan_result() {
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
    spec.max_turns = 20;
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
        "mine should not run until the scan countdown finishes"
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
        "mine should run once oreDistance() finishes waiting for the scan"
    );
    // StartScan + consume + scan_time await ticks + compare/branch need more than
    // scan_time mining cycles when cpu_speed is 1.
    assert!(
        turns_until_mine > 6,
        "oreDistance() must wait out scan_time across mining cycles, not finish early; took {turns_until_mine}"
    );
    assert!(
        matches!(
            simulation.robot(0).scan_state,
            crate::ground::ScanState::Complete(_)
        ),
        "scan should complete once oreDistance() finishes waiting"
    );
}

#[test]
fn program_bridge_ore_type_waits_full_scan_time_across_cycles() {
    let source = "scan(); bool found = false; if (oreType() == 1) { mine(); }";
    let program = robominer_program::compile_executable_source(source)
        .expect("scan oreType wait program should compile");

    let mut ground = Ground::new(10, 10);
    ground.add_ore_heap(4, 4, 0, 2, 2);

    let mut spec = RobotSpec::test_robot();
    spec.cpu_speed = 10;
    spec.scan_time = 50;
    spec.scan_distance = 50;
    spec.max_turns = 20;
    let max_turns = spec.max_turns;

    let mut simulation = Simulation::new(
        ground,
        max_turns,
        vec![ScriptedRobot::from_executable_program(spec, &program)],
    );
    simulation.prepare_test_run();

    for _ in 0..3 {
        simulation.advance_test_turn();
    }

    assert!(
        matches!(
            simulation.robot(0).scan_state,
            crate::ground::ScanState::Scanning { .. }
        ),
        "scan must still be in progress after 3 mining cycles with scan_time=50 and cpu_speed=10"
    );
    assert_eq!(
        simulation.robot(0).actions_done()[6],
        0,
        "mine must not run before the full scan countdown"
    );

    while simulation.robot(0).actions_done()[6] == 0 && simulation.time() < max_turns {
        simulation.advance_test_turn();
    }

    assert!(
        simulation.time() >= 5,
        "oreType() wait should span multiple mining cycles; finished at time {}",
        simulation.time()
    );
    assert_eq!(
        simulation.robot(0).actions_done()[6],
        1,
        "mine should run after oreType() waits out the scan"
    );
    assert!(
        matches!(
            simulation.robot(0).scan_state,
            crate::ground::ScanState::Complete(_)
        ),
        "scan should be complete once oreType() returns"
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
    spec.max_turns = 20;
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
    // oreDistance() == 0; that must not livelock pending program motion state.
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
            !simulation
                .program_runner(0)
                .unwrap()
                .has_pending_program_motion(),
            "move(oreDistance()) with distance 0 must not leave pending_program_motion state"
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
        !simulation
            .program_runner(0)
            .unwrap()
            .has_pending_program_motion(),
        "runner should clear pending_program_motion after zero-distance move"
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

#[test]
fn scan_bridge_uses_pose_from_scan_start_not_completion() {
    // Ore sits on +Y from spawn. scan(45) from facing 45° finds it. After rotate(180)
    // the same relative scan would miss it if completion pose were used.
    let source = "scan(45); rotate(180); if (oreType() > 0) { mine(); }";
    let program = robominer_program::compile_executable_source(source)
        .expect("scan start-pose program should compile");

    let mut ground = Ground::new(10, 10);
    ground.at_mut(0, 4).add_ore(0, 8);

    let mut spec = RobotSpec::test_robot();
    spec.cpu_speed = 72;
    spec.scan_time = 6;
    spec.scan_distance = 10;
    spec.rotate_speed = 90;
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
        "scan should start before rotate"
    );
    assert!(
        simulation.robot(0).actions_done()[4] > 0,
        "rotate should run between scan start and oreType()"
    );
    assert_eq!(
        simulation.robot(0).actions_done()[6],
        1,
        "oreType() must use the pose from scan() start so the +Y ore is still found after rotating 180"
    );
}

#[test]
fn scan_bridge_zero_scan_time_completes_without_waiting() {
    let source = "scan(); mine();";
    let program = robominer_program::compile_executable_source(source)
        .expect("zero scan_time program should compile");

    let mut spec = RobotSpec::test_robot();
    spec.cpu_speed = 72;
    spec.scan_time = 0;
    spec.scan_distance = 50;
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
    assert!(
        matches!(
            simulation.robot(0).scan_state,
            crate::ground::ScanState::Complete(_)
        ),
        "scan_time 0 should complete immediately at start_scan"
    );
    assert_eq!(
        simulation.robot(0).actions_done()[6],
        1,
        "mine should run in the same cycle when scan_time is 0"
    );
}
