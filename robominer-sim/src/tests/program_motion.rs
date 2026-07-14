use super::helpers::*;
use crate::*;

#[test]
fn executable_move_uses_robot_max_cycles_property() {
    let program = robominer_program::compile_executable_source("move(robot.maxCycles);")
        .expect("source should compile to executable actions");

    let mut spec = RobotSpec::test_robot();
    spec.forward_speed = 1.0;
    spec.max_turns = 3;

    let mut simulation = Simulation::new(
        Ground::new(5, 5),
        3,
        vec![ScriptedRobot::from_executable_program(spec, &program)],
    );

    simulation.run();

    let position = simulation.robot(0).position();
    assert_close(position.x, 3.0 * 45.0_f64.to_radians().cos());
    assert_close(position.y, 3.0 * 45.0_f64.to_radians().sin());
    assert_eq!(simulation.robot(0).actions_done()[2], 3);
}

#[test]
fn executable_moves_are_split_across_turns() {
    let program = robominer_program::compile_executable_source("move(2.5);")
        .expect("source should compile to executable actions");

    let mut spec = RobotSpec::test_robot();
    spec.forward_speed = 1.0;
    spec.max_turns = 3;

    let mut simulation = Simulation::new(
        Ground::new(5, 5),
        3,
        vec![ScriptedRobot::from_executable_program(spec, &program)],
    );

    simulation.run();

    let position = simulation.robot(0).position();
    assert_close(position.x, 2.5 * 45.0_f64.to_radians().cos());
    assert_close(position.y, 2.5 * 45.0_f64.to_radians().sin());
    assert_eq!(simulation.robot(0).actions_done()[2], 3);
}

#[test]
fn executable_move_statement_matches_expression_behavior() {
    for source in [
        "move(2.5); rotate(90);",
        "if (move(2.5) > 2.4) { rotate(90); } else { mine(); }",
    ] {
        let program = robominer_program::compile_executable_source(source)
            .unwrap_or_else(|err| panic!("{source} should compile: {err}"));
        let mut spec = RobotSpec::test_robot();
        spec.forward_speed = 1.0;
        spec.rotate_speed = 90;
        spec.max_turns = 4;

        let mut simulation = Simulation::new(
            Ground::new(10, 10),
            4,
            vec![ScriptedRobot::from_executable_program(spec, &program)],
        );

        simulation.run();

        let position = simulation.robot(0).position();
        assert_close(position.x, 2.5 * 45.0_f64.to_radians().cos());
        assert_close(position.y, 2.5 * 45.0_f64.to_radians().sin());
        assert_eq!(position.orientation, 135, "{source}");
        assert_eq!(simulation.robot(0).actions_done()[2], 3, "{source}");
        assert_eq!(simulation.robot(0).actions_done()[4], 1, "{source}");
        assert_eq!(simulation.robot(0).actions_done()[6], 0, "{source}");
    }
}

#[test]
fn executable_rotate_statement_matches_expression_behavior() {
    let mut ground = Ground::new(5, 5);
    ground.at_mut(0, 0).add_ore(0, 8);

    for source in [
        "rotate(180); mine();",
        "if (rotate(180) == 180) { mine(); } else { move(1); }",
    ] {
        let program = robominer_program::compile_executable_source(source)
            .unwrap_or_else(|err| panic!("{source} should compile: {err}"));
        let mut spec = RobotSpec::test_robot();
        spec.rotate_speed = 90;
        spec.mining_speed = 4;
        spec.max_turns = 3;

        let mut simulation = Simulation::new(
            ground.clone(),
            3,
            vec![ScriptedRobot::from_executable_program(spec, &program)],
        );

        simulation.run();

        assert_eq!(
            simulation.robot(0).position().orientation,
            225,
            "{source}"
        );
        assert_eq!(simulation.robot(0).ore_at(0), 4, "{source}");
        assert_eq!(simulation.robot(0).actions_done()[4], 2, "{source}");
        assert_eq!(simulation.robot(0).actions_done()[6], 1, "{source}");
        assert_eq!(simulation.robot(0).actions_done()[2], 0, "{source}");
    }
}

#[test]
fn executable_move_return_value_detects_wall_limited_movement() {
    let program = robominer_program::compile_executable_source(
        "if (move(10) < 10) { rotate(90); } else { mine(); }",
    )
    .expect("source should compile with move action return values");

    let mut spec = RobotSpec::test_robot();
    spec.forward_speed = 10.0;
    spec.rotate_speed = 90;
    spec.max_turns = 2;

    let mut simulation = Simulation::new(
        Ground::new(5, 5),
        2,
        vec![ScriptedRobot::from_executable_program(spec, &program)],
    );

    simulation.run();

    let position = simulation.robot(0).position();
    assert_close(position.x, 4.0);
    assert_close(position.y, 4.0);
    assert_eq!(position.orientation, 135);
    assert_eq!(simulation.robot(0).actions_done()[2], 1);
    assert_eq!(simulation.robot(0).actions_done()[4], 1);
    assert_eq!(simulation.robot(0).actions_done()[6], 0);
}

#[test]
fn executable_move_return_value_detects_robot_collision() {
    let program = robominer_program::compile_executable_source(
        "if (move(3) < 3) { rotate(90); } else { mine(); }",
    )
    .expect("source should compile with move action return values");

    let mut spec = RobotSpec::test_robot();
    spec.forward_speed = 3.0;
    spec.rotate_speed = 90;
    spec.max_turns = 2;

    let mut simulation = Simulation::new(
        Ground::new(5, 5),
        2,
        vec![
            ScriptedRobot::from_executable_program(spec.clone(), &program),
            ScriptedRobot::new(spec, vec![RobotAction::Forward, RobotAction::Wait]),
        ],
    );

    simulation.run();

    assert_eq!(simulation.robot(0).position().orientation, 135);
    assert_eq!(simulation.robot(0).actions_done()[2], 1);
    assert_eq!(simulation.robot(0).actions_done()[4], 1);
    assert_eq!(simulation.robot(0).actions_done()[6], 0);
}

#[test]
fn executable_long_move_expression_returns_after_all_chunks() {
    let program = robominer_program::compile_executable_source(
        "if (move(2.5) > 2.4) { rotate(90); } else { mine(); }",
    )
    .expect("source should compile with multi-turn move expressions");

    let mut spec = RobotSpec::test_robot();
    spec.forward_speed = 1.0;
    spec.rotate_speed = 90;
    spec.max_turns = 4;

    let mut simulation = Simulation::new(
        Ground::new(10, 10),
        4,
        vec![ScriptedRobot::from_executable_program(spec, &program)],
    );

    simulation.run();

    let position = simulation.robot(0).position();
    assert_close(position.x, 2.5 * 45.0_f64.to_radians().cos());
    assert_close(position.y, 2.5 * 45.0_f64.to_radians().sin());
    assert_eq!(position.orientation, 135);
    assert_eq!(simulation.robot(0).actions_done()[2], 3);
    assert_eq!(simulation.robot(0).actions_done()[4], 1);
    assert_eq!(simulation.robot(0).actions_done()[6], 0);
}

#[test]
fn executable_long_rotate_expression_returns_after_all_chunks() {
    let program = robominer_program::compile_executable_source(
        "if (rotate(180) == 180) { mine(); } else { move(1); }",
    )
    .expect("source should compile with multi-turn rotate expressions");
    let mut ground = Ground::new(5, 5);
    ground.at_mut(0, 0).add_ore(0, 8);

    let mut spec = RobotSpec::test_robot();
    spec.rotate_speed = 90;
    spec.mining_speed = 4;
    spec.max_turns = 3;

    let mut simulation = Simulation::new(
        ground,
        3,
        vec![ScriptedRobot::from_executable_program(spec, &program)],
    );

    simulation.run();

    assert_eq!(simulation.robot(0).position().orientation, 225);
    assert_eq!(simulation.robot(0).ore_at(0), 4);
    assert_eq!(simulation.robot(0).actions_done()[4], 2);
    assert_eq!(simulation.robot(0).actions_done()[6], 1);
    assert_eq!(simulation.robot(0).actions_done()[2], 0);
}

#[test]
fn simplified_homing_program_trace() {
    let source = r#"{
move(4);
mine();
rotate(20);
move(4);

rotate(160);

while (robot.xPos > 0 && robot.yPos > 0)
move(robot.forwardSpeed);

if (robot.yPos > 0) {
rotate(45);
move(robot.yPos);
}

while (true);
}"#;
    let program = robominer_program::compile_executable_source(source).unwrap();
    let mut ground = Ground::new(20, 20);
    ground.at_mut(0, 0).add_ore(0, 50);
    let mut spec = RobotSpec::test_robot();
    spec.max_turns = 500;
    spec.forward_speed = 1.42;
    let mut simulation = Simulation::new(
        ground,
        500,
        vec![ScriptedRobot::from_executable_program(spec, &program)],
    );
    simulation.run();
    let robot = simulation.robot(0);
    let (x_pos, y_pos, _orientation) = robominer_program::rally_robot_pose(
        robot.center_position().x,
        robot.center_position().y,
        robot.center_position().orientation,
        robot.initial_center_x,
        robot.initial_center_y,
        robot.initial_orientation,
    );
    assert!(y_pos.abs() < 0.01, "expected yPos home, got {y_pos}");
    assert!(x_pos.abs() < 0.01, "expected xPos home, got {x_pos}");
}
