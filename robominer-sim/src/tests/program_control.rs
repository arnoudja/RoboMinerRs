use super::helpers::*;
use crate::*;

#[test]
fn compiled_program_actions_drive_simulation() {
    let program = robominer_program::compile_executable_source(
        "move(1.4142135623730951); rotate(90); mine();",
    )
    .expect("source should compile to executable actions");
    let mut ground = Ground::new(5, 5);
    ground.at_mut(1, 1).add_ore(0, 8);

    let mut spec = RobotSpec::test_robot();
    spec.forward_speed = 2.0_f64.sqrt();
    spec.rotate_speed = 90;
    spec.mining_speed = 4;
    spec.max_turns = 3;

    let mut simulation = Simulation::new(
        ground,
        3,
        vec![ScriptedRobot::from_executable_program(spec, &program)],
    );

    simulation.run();

    let position = simulation.robot(0).position();
    assert_close(position.x, 1.0);
    assert_close(position.y, 1.0);
    assert_eq!(position.orientation, 135);
    assert_eq!(simulation.robot(0).ore_at(0), 4);
    assert_eq!(simulation.ground().at(1, 1).ore_at(0), 4);
}

#[test]
fn compiled_program_actions_restart_after_finishing() {
    let program = robominer_program::compile_executable_source("move(1); mine();")
        .expect("default source should compile to executable actions");
    let mut ground = Ground::new(5, 5);
    ground.at_mut(1, 1).add_ore(0, 8);

    let mut spec = RobotSpec::test_robot();
    spec.forward_speed = 1.0;
    spec.mining_speed = 4;
    spec.max_turns = 4;

    let mut simulation = Simulation::new(
        ground,
        4,
        vec![ScriptedRobot::from_executable_program(spec, &program)],
    );

    simulation.run();

    assert_eq!(simulation.robot(0).actions_done()[2], 2);
    assert_eq!(simulation.robot(0).actions_done()[6], 2);
    assert_eq!(simulation.robot(0).ore_at(0), 6);
    assert_eq!(simulation.ground().at(1, 1).ore_at(0), 2);
}

#[test]
fn executable_while_uses_runtime_time_expression() {
    let program =
        robominer_program::compile_executable_source("while (time() > 0) { rotate(90); }")
            .expect("source should compile to executable control flow");

    let mut spec = RobotSpec::test_robot();
    spec.rotate_speed = 90;
    spec.max_turns = 3;

    let mut simulation = Simulation::new(
        Ground::new(5, 5),
        3,
        vec![ScriptedRobot::from_executable_program(spec, &program)],
    );

    simulation.run();

    assert_eq!(simulation.robot(0).position().orientation, 225);
    assert_eq!(simulation.robot(0).actions_done()[4], 2);
}

#[test]
fn executable_if_uses_runtime_ore_expression() {
    let program = robominer_program::compile_executable_source(
        "mine(); if (ore(1) > 0) { dump(1); } else { rotate(90); }",
    )
    .expect("source should compile to executable control flow");
    let mut ground = Ground::new(5, 5);
    ground.at_mut(0, 0).add_ore(0, 8);

    let mut spec = RobotSpec::test_robot();
    spec.mining_speed = 4;
    spec.max_turns = 2;

    let mut simulation = Simulation::new(
        ground,
        2,
        vec![ScriptedRobot::from_executable_program(spec, &program)],
    );

    simulation.run();

    assert_eq!(simulation.robot(0).total_ore(), 0);
    assert_eq!(simulation.ground().at(0, 0).ore_at(0), 8);
    assert_eq!(simulation.robot(0).actions_done()[7], 1);
    assert_eq!(simulation.robot(0).actions_done()[4], 0);
}

#[test]
fn executable_while_mine_repeats_until_tile_is_depleted() {
    let program = robominer_program::compile_executable_source("while (mine());")
        .expect("source should compile with mine action return values");
    let mut ground = Ground::new(5, 5);
    ground.at_mut(0, 0).add_ore(0, 8);

    let mut spec = RobotSpec::test_robot();
    spec.mining_speed = 4;
    spec.max_turns = 6;

    let mut simulation = Simulation::new(
        ground,
        6,
        vec![ScriptedRobot::from_executable_program(spec, &program)],
    );

    simulation.run();

    assert_eq!(simulation.robot(0).ore_at(0), 8);
    assert_eq!(simulation.ground().at(0, 0).ore_at(0), 0);
    assert_eq!(simulation.robot(0).actions_done()[6], 6);
}

#[test]
fn executable_while_true_semicolon_does_not_restart_program() {
    for source in [
        "rotate(180);\nwhile (true);",
        "rotate(180);\nwhile (true) {}",
    ] {
        let program = robominer_program::compile_executable_source(source)
            .unwrap_or_else(|err| panic!("{source} should compile: {err}"));
        let mut spec = RobotSpec::test_robot();
        spec.rotate_speed = 30;
        spec.max_turns = 6;

        let mut simulation = Simulation::new(
            Ground::new(5, 5),
            6,
            vec![ScriptedRobot::from_executable_program(spec, &program)],
        );

        simulation.run();

        assert_eq!(simulation.robot(0).position().orientation, 225, "{source}");
        assert_eq!(simulation.robot(0).actions_done()[4], 6, "{source}");
    }
}

#[test]
fn slow_cpu_waits_when_program_needs_more_instructions_than_cycle_budget() {
    let program = robominer_program::compile_executable_source("int x = 1; int y = 2; mine();")
        .expect("program should compile");
    let mut ground = Ground::new(5, 5);
    ground.at_mut(0, 0).add_ore(0, 8);

    let mut spec = RobotSpec::test_robot();
    spec.cpu_speed = 2;
    spec.mining_speed = 8;
    spec.max_turns = 2;

    let mut simulation = Simulation::new(
        ground,
        2,
        vec![ScriptedRobot::from_executable_program(spec, &program)],
    );

    simulation.run();

    assert!(
        simulation.robot(0).actions_done()[1] > 0,
        "slow CPU should insert Wait actions when instruction budget is exhausted"
    );
    assert_eq!(simulation.robot(0).actions_done()[6], 1);
    assert!(simulation.robot(0).ore_at(0) > 0);
}

#[test]
fn executable_do_while_mines_once_per_cycle_while_false_never_mines() {
    let do_program = robominer_program::compile_executable_source("do { mine(); } while (false);")
        .expect("do-while should compile for simulation");
    let while_program = robominer_program::compile_executable_source("while (false) { mine(); }")
        .expect("while-false should compile for simulation");
    let mut ground = Ground::new(5, 5);
    ground.at_mut(0, 0).add_ore(0, 100);

    let mut spec = RobotSpec::test_robot();
    spec.mining_speed = 8;
    spec.max_turns = 2;

    let mut do_simulation = Simulation::new(
        ground.clone(),
        2,
        vec![ScriptedRobot::from_executable_program(
            spec.clone(),
            &do_program,
        )],
    );
    do_simulation.run();

    let mut while_simulation = Simulation::new(
        ground,
        2,
        vec![ScriptedRobot::from_executable_program(spec, &while_program)],
    );
    while_simulation.run();

    assert_eq!(do_simulation.robot(0).actions_done()[6], 2);
    assert_eq!(while_simulation.robot(0).actions_done()[6], 0);
    assert!(do_simulation.robot(0).ore_at(0) > 0);
}

#[test]
fn executable_dump_return_value_can_drive_branch() {
    let program = robominer_program::compile_executable_source(
        "mine(); if (dump(1) > 0) { rotate(90); } else { move(1); }",
    )
    .expect("source should compile with dump action return values");
    let mut ground = Ground::new(5, 5);
    ground.at_mut(0, 0).add_ore(0, 6);

    let mut spec = RobotSpec::test_robot();
    spec.mining_speed = 4;
    spec.rotate_speed = 90;
    spec.max_turns = 3;

    let mut simulation = Simulation::new(
        ground,
        3,
        vec![ScriptedRobot::from_executable_program(spec, &program)],
    );

    simulation.run();

    assert_eq!(simulation.robot(0).total_ore(), 0);
    assert_eq!(simulation.robot(0).position().orientation, 135);
    assert_eq!(simulation.robot(0).actions_done()[7], 1);
    assert_eq!(simulation.robot(0).actions_done()[4], 1);
    assert_eq!(simulation.robot(0).actions_done()[2], 0);
}

#[test]
fn executable_variables_drive_simulation_loop() {
    let program = robominer_program::compile_executable_source(
        "int steps = 0; while (steps < 2) { move(1); steps++; };",
    )
    .expect("source should compile with variable loop state");

    let mut spec = RobotSpec::test_robot();
    spec.forward_speed = 1.0;
    spec.max_turns = 2;

    let mut simulation = Simulation::new(
        Ground::new(5, 5),
        2,
        vec![ScriptedRobot::from_executable_program(spec, &program)],
    );

    simulation.run();

    let position = simulation.robot(0).position();
    assert_close(position.x, 2.0 * 45.0_f64.to_radians().cos());
    assert_close(position.y, 2.0 * 45.0_f64.to_radians().sin());
    assert_eq!(simulation.robot(0).actions_done()[2], 2);
}

#[test]
fn executable_variable_action_argument_drives_simulation() {
    let program = robominer_program::compile_executable_source("int rot = 90; rotate(rot);")
        .expect("source should compile with variable action arguments");

    let mut spec = RobotSpec::test_robot();
    spec.rotate_speed = 90;
    spec.max_turns = 1;

    let mut simulation = Simulation::new(
        Ground::new(5, 5),
        1,
        vec![ScriptedRobot::from_executable_program(spec, &program)],
    );

    simulation.run();

    assert_eq!(simulation.robot(0).position().orientation, 135);
    assert_eq!(simulation.robot(0).actions_done()[4], 1);
}
