use crate::*;

#[test]
fn search_loop_repeats_move_after_scan_finds_nothing() {
    let source = r#"bool found = false;
while (!found) { move(2); scan(); if (oreType() > 0) { found = true; } }"#;
    let program = robominer_program::compile_executable_source(source)
        .expect("minimal search loop should compile");

    let mut spec = RobotSpec::test_robot();
    spec.forward_speed = 1.0;
    spec.scan_time = 6;
    spec.scan_distance = 8;
    spec.max_turns = 40;

    let mut simulation = Simulation::new_with_ore_ids(
        Ground::new(20, 20),
        spec.max_turns,
        vec![ScriptedRobot::from_executable_program(spec, &program)],
        vec![1_i64],
    );

    simulation.run();

    assert!(
        simulation.robot(0).actions_done()[2] > 5,
        "move(2) should repeat when scan finds nothing, got {} move actions",
        simulation.robot(0).actions_done()[2]
    );
}

#[test]
fn user_search_program_continues_moving_when_no_ore_detected() {
    let source = r#"bool found = false;
while (!found)
{
move(10);
scan();
if (oreType() > 0) { found = true; }
else {
    scan(30);
    if (oreType() > 0) { rotate(30); found = true; }
    else {
        scan(-30);
        if (oreType() > 0) { rotate(-30); found = true; }
    }
}
}
while (true)
{
if (move(1) < 0.9)
{
    move(-1);
    rotate(135);
    rotate(135);
    rotate(135);
    rotate(135);
    move(1);
}
while (mine());
}"#;
    let program = robominer_program::compile_executable_source(source)
        .expect("user search program should compile");

    let mut spec = RobotSpec::test_robot();
    spec.forward_speed = 1.0;
    spec.scan_time = 6;
    spec.scan_distance = 8;
    spec.rotate_speed = 30;
    spec.max_turns = 60;

    let mut simulation = Simulation::new_with_ore_ids(
        Ground::new(20, 20),
        spec.max_turns,
        vec![ScriptedRobot::from_executable_program(spec, &program)],
        vec![1_i64],
    );

    let start_x = simulation.robot(0).position().x;
    simulation.run();
    let moves = simulation.robot(0).actions_done()[2];
    let end_x = simulation.robot(0).position().x;

    assert!(
        moves > 3,
        "expected repeated move(10) attempts, got {moves} move actions"
    );
    assert!(
        end_x > start_x + 0.5,
        "robot should travel forward while searching (start={start_x}, end={end_x})"
    );
}

#[test]
fn search_program_detects_ore_ahead_without_stalling() {
    let source = r#"bool found = false;
while (!found)
{
move(2);
scan();
if (oreType() > 0) { found = true; }
else {
    scan(30);
    if (oreType() > 0) { rotate(30); found = true; }
    else {
        scan(-30);
        if (oreType() > 0) { rotate(-30); found = true; }
    }
}
}"#;
    let program = robominer_program::compile_executable_source(source)
        .expect("search program should compile");

    let mut ground = Ground::new(20, 20);
    let ahead_x = 2;
    let ahead_y = 2;
    ground.at_mut(ahead_x, ahead_y).add_ore(0, 5);

    let mut spec = RobotSpec::test_robot();
    spec.forward_speed = 1.0;
    spec.scan_time = 6;
    spec.scan_distance = 8;
    spec.rotate_speed = 30;
    spec.max_turns = 40;

    let mut simulation = Simulation::new_with_ore_ids(
        ground,
        spec.max_turns,
        vec![ScriptedRobot::from_executable_program(spec, &program)],
        vec![1_i64],
    );

    simulation.run();

    let moves = simulation.robot(0).actions_done()[2];
    let scans = simulation.robot(0).actions_done()[ROBOT_ACTION_TYPE_SCAN];
    assert!(
        moves > 0,
        "robot should keep moving while scanning, got {moves} move actions"
    );
    assert!(
        scans > 0,
        "search program should record scan actions, got {scans}"
    );
}

#[test]
fn search_loop_keeps_found_false_and_repeats_move() {
    let source = "bool found = false; while (!found) { move(2); scan(); if (oreType() > 0) { found = true; } }";
    let program =
        robominer_program::compile_executable_source(source).expect("search loop should compile");
    assert!(
        program.requires_runtime(),
        "search program must use the runtime interpreter, not static expansion"
    );

    let mut spec = RobotSpec::test_robot();
    spec.forward_speed = 1.0;
    spec.scan_time = 6;
    spec.scan_distance = 8;
    spec.max_turns = 30;

    let mut simulation = Simulation::new_with_ore_ids(
        Ground::new(20, 20),
        spec.max_turns,
        vec![ScriptedRobot::from_executable_program(spec, &program)],
        vec![1_i64],
    );

    simulation.run();

    let runner = simulation
        .program_runner(0)
        .expect("search loop robot should run an executable program");
    assert_eq!(
        runner.runtime_variable("found"),
        0.0,
        "scan must not set found from move distance"
    );
    assert!(
        simulation.robot(0).actions_done()[2] > 5,
        "robot should keep attempting move(2), got {} move actions",
        simulation.robot(0).actions_done()[2]
    );
}
