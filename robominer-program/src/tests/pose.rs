use crate::*;

use super::helpers::*;

#[test]
fn rally_map_position_tracks_map_east_and_north() {
    let (x_pos, y_pos) = rally_map_position(3.5, 4.0, 1.5, 1.0);
    assert!((x_pos - 2.0).abs() < f64::EPSILON);
    assert!((y_pos - 3.0).abs() < f64::EPSILON);
}

#[test]
fn rally_map_position_is_zero_at_initial_center() {
    for (x, y) in [(0.5, 0.5), (9.5, 0.5), (0.5, 9.5), (9.5, 9.5)] {
        let (x_pos, y_pos) = rally_map_position(x, y, x, y);
        assert!((x_pos).abs() < f64::EPSILON, "x at ({x},{y})");
        assert!((y_pos).abs() < f64::EPSILON, "y at ({x},{y})");
    }
}

#[test]
fn rally_robot_pose_spawn_is_origin_with_orientation_135() {
    for spawn_orientation in [45, 315, 135, 225] {
        let (x_pos, y_pos, orientation) =
            rally_robot_pose(2.5, 3.5, spawn_orientation, 2.5, 3.5, spawn_orientation);
        assert!((x_pos).abs() < f64::EPSILON);
        assert!((y_pos).abs() < f64::EPSILON);
        assert!(
            (orientation - 135.0).abs() < f64::EPSILON,
            "spawn ori {spawn_orientation}"
        );
    }
}

#[test]
fn rally_robot_pose_tracks_orientation_and_position_delta() {
    let (x_pos, y_pos, orientation) = rally_robot_pose(0.5, 1.5, 90, 0.5, 0.5, 45);
    assert!((x_pos).abs() < f64::EPSILON);
    assert!((y_pos - 1.0).abs() < f64::EPSILON);
    assert!((orientation - 180.0).abs() < f64::EPSILON);
}

#[test]
fn rally_pose_properties_evaluate_from_context() {
    let program =
        compile_executable_source("move(robot.orientation);").expect("program should compile");
    let mut runner = program.runner();
    let mut context = robot_context(1.0);
    context.robot.orientation = 180.0;

    loop {
        match runner.step(&mut context) {
            ProgramStep::Action(ExecutableAction::Move(distance)) => {
                assert!((distance - 180.0).abs() < f64::EPSILON);
                break;
            }
            ProgramStep::Cpu => {}
            ProgramStep::Done => panic!("program finished without move"),
            other => panic!("unexpected step {other:?}"),
        }
    }

    let program = compile_executable_source("move(robot.xPos + robot.yPos);")
        .expect("program should compile");
    let mut runner = program.runner();
    let mut context = robot_context(1.0);
    context.robot.x_pos = 0.25;
    context.robot.y_pos = 1.75;

    loop {
        match runner.step(&mut context) {
            ProgramStep::Action(ExecutableAction::Move(distance)) => {
                assert!((distance - 2.0).abs() < f64::EPSILON);
                break;
            }
            ProgramStep::Cpu => {}
            ProgramStep::Done => panic!("program finished without move"),
            other => panic!("unexpected step {other:?}"),
        }
    }
}
