use super::helpers::*;
use crate::*;

#[test]
fn initializes_first_robot_like_legacy_rally() {
    let mut simulation =
        Simulation::new(Ground::new(5, 7), 0, vec![robot_with_actions(Vec::new())]);

    simulation.run();

    let position = simulation.robot(0).position();
    assert_close(position.x, 0.0);
    assert_close(position.y, 0.0);
    assert_eq!(position.orientation, 45);
    assert_eq!(simulation.time(), 0);
}

#[test]
fn moves_and_rotates_robot_by_scripted_actions() {
    let mut spec = RobotSpec::test_robot();
    spec.forward_speed = 2.0_f64.sqrt();
    spec.rotate_speed = 90;
    spec.max_turns = 2;

    let mut simulation = Simulation::new(
        Ground::new(5, 5),
        2,
        vec![ScriptedRobot::new(
            spec,
            vec![RobotAction::Forward, RobotAction::RotateRight],
        )],
    );

    simulation.run();

    let position = simulation.robot(0).position();
    assert_close(position.x, 1.0);
    assert_close(position.y, 1.0);
    assert_eq!(position.orientation, 135);
    assert_eq!(simulation.robot(0).actions_done()[2], 1);
    assert_eq!(simulation.robot(0).actions_done()[4], 1);
}

#[test]
fn clamps_movement_at_walls_and_adjusts_time_fraction() {
    let mut spec = RobotSpec::test_robot();
    spec.backward_speed = 2.0_f64.sqrt();
    spec.max_turns = 1;

    let mut simulation = Simulation::new(
        Ground::new(5, 5),
        1,
        vec![ScriptedRobot::new(spec, vec![RobotAction::Backward])],
    );

    simulation.run();

    let position = simulation.robot(0).position();
    assert_close(position.x, 0.0);
    assert_close(position.y, 0.0);
    assert_eq!(position.orientation, 45);
}

#[test]
fn cardinal_orientation_move_avoids_trig_epsilon_at_west_wall() {
    let start = Position::new(0.0, 2.0, 270);
    let destination = start.calculate_move_position(1.42);

    assert_close(start.x, destination.x);
    assert_close(destination.y, 0.58);
}

#[test]
fn multi_robot_collision_clips_converging_moves() {
    let mut spec = RobotSpec::test_robot();
    spec.forward_speed = 3.0;
    spec.max_turns = 1;

    let mut simulation = Simulation::new(
        Ground::new(5, 5),
        1,
        vec![
            ScriptedRobot::new(spec.clone(), vec![RobotAction::Forward]),
            ScriptedRobot::new(spec, vec![RobotAction::Forward]),
        ],
    );

    simulation.run();

    let first = simulation.robot(0).position();
    let second = simulation.robot(1).position();
    assert!(
        first.distance(&second) >= 0.99,
        "robots should stop before overlapping: {first:?} vs {second:?}"
    );
    assert!(
        first.distance(&second) < 4.0,
        "robots should still move toward each other"
    );
    assert_eq!(simulation.robot(0).actions_done()[2], 1);
    assert_eq!(simulation.robot(1).actions_done()[2], 1);
}
