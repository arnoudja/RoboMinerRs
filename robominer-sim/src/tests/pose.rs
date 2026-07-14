use super::helpers::*;
use crate::*;

#[test]
fn rally_position_tracks_east_and_north_from_initial() {
    let mut spec = RobotSpec::test_robot();
    spec.robot_size = 1.0;
    spec.forward_speed = 1.0;
    spec.max_turns = 5;

    let mut simulation = Simulation::new(
        Ground::new(10, 10),
        5,
        vec![ScriptedRobot::new(
            spec,
            vec![
                RobotAction::Rotate(-45.0),
                RobotAction::Move(1.0),
                RobotAction::Rotate(90.0),
                RobotAction::Move(1.0),
            ],
        )],
    );
    simulation.run();

    let robot = simulation.robot(0);
    let center = robot.center_position();
    let (x_pos, y_pos, _) = robominer_program::rally_robot_pose(
        center.x,
        center.y,
        center.orientation,
        robot.initial_center_x,
        robot.initial_center_y,
        robot.initial_orientation,
    );

    assert_close(x_pos, 1.0);
    assert_close(y_pos, 1.0);
}

#[test]
fn rally_pose_properties_start_at_origin() {
    let mut simulation = Simulation::new(
        Ground::new(10, 10),
        1,
        vec![ScriptedRobot::from_executable_program(
            RobotSpec::test_robot(),
            &seeded_program("while (false) { move(1); }"),
        )],
    );

    simulation.run();

    let robot = simulation.robot(0);
    let position = robot.position();
    let half = robot.spec().robot_size / 2.0;
    let center_x = position.x + half;
    let center_y = position.y + half;
    let (x_pos, y_pos, orientation) = robominer_program::rally_robot_pose(
        center_x,
        center_y,
        position.orientation,
        center_x,
        center_y,
        position.orientation,
    );

    assert_close(x_pos, 0.0);
    assert_close(y_pos, 0.0);
    assert_close(orientation, 135.0);
}

#[test]
fn rally_pose_after_rotate_and_move() {
    let mut spec = RobotSpec::test_robot();
    spec.robot_size = 1.0;
    spec.forward_speed = 1.0;
    spec.rotate_speed = 90;
    spec.max_turns = 2;

    let mut simulation = Simulation::new(
        Ground::new(10, 10),
        2,
        vec![ScriptedRobot::new(
            spec,
            vec![RobotAction::Rotate(45.0), RobotAction::Move(1.0)],
        )],
    );
    simulation.run();

    let robot = simulation.robot(0);
    let position = robot.position();
    let half = robot.spec().robot_size / 2.0;
    let (x_pos, y_pos, orientation) = robominer_program::rally_robot_pose(
        position.x + half,
        position.y + half,
        position.orientation,
        0.5,
        0.5,
        45,
    );

    assert_close(x_pos, 0.0);
    assert!(y_pos > 0.0);
    assert_close(orientation, 180.0);
}

