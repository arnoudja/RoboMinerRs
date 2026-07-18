use super::helpers::*;
use crate::physics::{
    apply_mining, check_wall_collision, find_collision_time, position_at_time, process_dump,
    process_move, process_requested_move, process_requested_rotation,
};
use crate::*;

fn bounds_for_ground(size_x: usize, size_y: usize, robot_size: f64) -> (f64, f64, f64, f64) {
    let half = robot_size / 2.0;
    (
        half - 0.5,
        half - 0.5,
        size_x as f64 - half - 0.5,
        size_y as f64 - half - 0.5,
    )
}

fn test_robot_at(position: Position, bounds: (f64, f64, f64, f64)) -> Robot {
    let mut robot = Robot::new(RobotSpec::test_robot());
    robot.position = position;
    robot.destination = position;
    robot.min_x = bounds.0;
    robot.min_y = bounds.1;
    robot.max_x = bounds.2;
    robot.max_y = bounds.3;
    robot
}

#[test]
fn position_at_time_interpolates_linearly() {
    let start = Position::new(0.0, 0.0, 0);
    let end = Position::new(10.0, 20.0, 0);

    assert_eq!(position_at_time(start, end, 1.0, 0.0), start);
    assert_eq!(position_at_time(start, end, 1.0, 1.0), end);

    let mid = position_at_time(start, end, 1.0, 0.5);
    assert_close(mid.x, 5.0);
    assert_close(mid.y, 10.0);
}

#[test]
fn process_move_sets_destination_speed_and_time_fraction() {
    let bounds = bounds_for_ground(5, 5, 1.0);
    let mut robot = test_robot_at(Position::new(1.0, 1.0, 0), bounds);

    process_move(&mut robot, 2.0, 0.5);

    assert_close(robot.current_speed, 2.0);
    assert_close(robot.time_fraction, 0.5);
    assert_close(robot.destination.x, 2.0);
    assert_close(robot.destination.y, 1.0);
}

#[test]
fn process_requested_move_uses_forward_and_backward_speeds() {
    let bounds = bounds_for_ground(5, 5, 1.0);
    let mut spec = RobotSpec::test_robot();
    spec.forward_speed = 4.0;
    spec.backward_speed = 2.0;

    let mut forward = Robot::new(spec.clone());
    forward.position = Position::new(1.0, 1.0, 0);
    forward.destination = forward.position;
    forward.min_x = bounds.0;
    forward.min_y = bounds.1;
    forward.max_x = bounds.2;
    forward.max_y = bounds.3;

    process_requested_move(&mut forward, 2.0);
    assert_close(forward.time_fraction, 0.5);
    assert_close(forward.current_speed, 4.0);
    assert_close(forward.destination.x, 3.0);

    let mut backward = Robot::new(spec);
    backward.position = Position::new(3.0, 1.0, 0);
    backward.destination = backward.position;
    backward.min_x = bounds.0;
    backward.min_y = bounds.1;
    backward.max_x = bounds.2;
    backward.max_y = bounds.3;

    process_requested_move(&mut backward, -1.0);
    assert_close(backward.time_fraction, 0.5);
    assert_close(backward.current_speed, -2.0);
    assert_close(backward.destination.x, 2.0);
}

#[test]
fn process_requested_move_zero_distance_is_no_op() {
    let bounds = bounds_for_ground(5, 5, 1.0);
    let mut robot = test_robot_at(Position::new(2.0, 2.0, 90), bounds);
    robot.time_fraction = 1.0;
    robot.current_speed = 5.0;

    process_requested_move(&mut robot, 0.0);

    assert_close(robot.time_fraction, 1.0);
    assert_close(robot.current_speed, 5.0);
    assert_eq!(robot.destination, robot.position);
}

#[test]
fn process_requested_rotation_sets_target_and_caps_time_fraction() {
    let mut robot = Robot::new(RobotSpec::test_robot());

    process_requested_rotation(&mut robot, 45.0);
    assert_eq!(robot.target_rotation, 90);
    assert_close(robot.time_fraction, 0.5);

    process_requested_rotation(&mut robot, -180.0);
    assert_eq!(robot.target_rotation, -90);
    assert_close(robot.time_fraction, 1.0);
}

#[test]
fn check_wall_collision_clips_westward_move_at_min_x() {
    let bounds = bounds_for_ground(5, 5, 1.0);
    let mut robot = test_robot_at(Position::new(bounds.0, 2.0, 180), bounds);
    robot.destination = robot.position.calculate_move_position(1.0);
    robot.time_fraction = 1.0;

    check_wall_collision(&mut robot);

    assert_close(robot.destination.x, bounds.0);
    assert_close(robot.destination.y, 2.0);
    assert_close(robot.time_fraction, 0.0);
}

#[test]
fn check_wall_collision_clips_diagonal_move_at_south_wall() {
    let bounds = bounds_for_ground(5, 5, 1.0);
    let mut robot = test_robot_at(Position::new(2.0, 1.5, 315), bounds);
    robot.destination = robot.position.calculate_move_position(2.0 * 2.0_f64.sqrt());
    robot.time_fraction = 1.0;

    check_wall_collision(&mut robot);

    assert_close(robot.destination.y, bounds.1);
    assert_close(robot.destination.x, 3.5);
    assert_close(robot.time_fraction, 0.75);
}

#[test]
fn check_wall_collision_leaves_in_bounds_destination_unchanged() {
    let bounds = bounds_for_ground(5, 5, 1.0);
    let mut robot = test_robot_at(Position::new(2.0, 2.0, 0), bounds);
    robot.destination = Position::new(3.0, 2.0, 0);
    robot.time_fraction = 1.0;

    check_wall_collision(&mut robot);

    assert_close(robot.destination.x, 3.0);
    assert_close(robot.time_fraction, 1.0);
}

#[test]
fn check_wall_collision_clips_northward_move_at_max_y() {
    let bounds = bounds_for_ground(5, 5, 1.0);
    let mut robot = test_robot_at(Position::new(2.0, bounds.3 - 1.0, 90), bounds);
    robot.destination = robot.position.calculate_move_position(2.0);
    robot.time_fraction = 1.0;

    check_wall_collision(&mut robot);

    assert_close(robot.destination.y, bounds.3);
    assert_close(robot.time_fraction, 0.5);
}

#[test]
fn find_collision_time_detects_converging_robots() {
    let spec = RobotSpec::test_robot();
    let mut first = Robot::new(spec.clone());
    first.position = Position::new(0.0, 0.0, 0);
    first.destination = Position::new(2.0, 0.0, 0);
    first.time_fraction = 1.0;
    first.current_speed = 2.0;

    let mut second = Robot::new(spec);
    second.position = Position::new(4.0, 0.0, 180);
    second.destination = Position::new(2.0, 0.0, 180);
    second.time_fraction = 1.0;
    second.current_speed = 2.0;

    let collision_time = find_collision_time(&first, &second, 0.0, 1.0);

    assert!(
        collision_time < 1.0,
        "expected collision before move completes, got {collision_time}"
    );
    assert!(
        collision_time > 0.0,
        "expected collision after move starts, got {collision_time}"
    );
}

#[test]
fn find_collision_time_returns_one_when_robots_move_apart() {
    let spec = RobotSpec::test_robot();
    let mut first = Robot::new(spec.clone());
    first.position = Position::new(0.0, 0.0, 0);
    first.destination = Position::new(1.0, 0.0, 0);
    first.time_fraction = 1.0;
    first.current_speed = 1.0;

    let mut second = Robot::new(spec);
    second.position = Position::new(4.0, 0.0, 180);
    second.destination = Position::new(3.0, 0.0, 180);
    second.time_fraction = 1.0;
    second.current_speed = 1.0;

    assert_close(find_collision_time(&first, &second, 0.0, 1.0), 1.0);
}

#[test]
fn find_collision_time_returns_one_for_stationary_robots() {
    let spec = RobotSpec::test_robot();
    let mut first = Robot::new(spec.clone());
    first.position = Position::new(0.0, 0.0, 0);
    first.destination = first.position;
    first.time_fraction = 1.0;

    let mut second = Robot::new(spec);
    second.position = Position::new(3.0, 0.0, 0);
    second.destination = second.position;
    second.time_fraction = 1.0;

    assert_close(find_collision_time(&first, &second, 0.0, 1.0), 1.0);
}

#[test]
fn apply_mining_transfers_ore_proportional_to_time_fraction() {
    let bounds = bounds_for_ground(5, 5, 1.0);
    let mut ground = Ground::new(5, 5);
    let center = Position::new(0.5, 0.5, 0);
    ground.at_position_mut(center).add_ore(0, 10);

    let mut robot = test_robot_at(Position::new(0.0, 0.0, 0), bounds);
    robot.target_mining[0] = 6;
    robot.time_fraction = 0.5;

    let change = apply_mining(&mut ground, &mut robot, 3);

    assert_eq!(robot.ore_at(0), 3);
    assert_eq!(robot.last_mined, 3);
    assert_eq!(ground.at_position(center).ore_at(0), 7);
    assert!(change.is_some());
    let change = change.unwrap();
    assert_eq!(change.x, 0);
    assert_eq!(change.y, 0);
    assert_eq!(change.time, 3);
}

#[test]
fn apply_mining_returns_none_when_nothing_mined() {
    let bounds = bounds_for_ground(5, 5, 1.0);
    let mut ground = Ground::new(5, 5);
    let mut robot = test_robot_at(Position::new(0.0, 0.0, 0), bounds);
    robot.target_mining[0] = 5;
    robot.time_fraction = 0.0;

    assert!(apply_mining(&mut ground, &mut robot, 1).is_none());
    assert_eq!(robot.ore_at(0), 0);
}

#[test]
fn apply_mining_respects_available_ground_ore() {
    let bounds = bounds_for_ground(5, 5, 1.0);
    let mut ground = Ground::new(5, 5);
    let center = Position::new(0.5, 0.5, 0);
    ground.at_position_mut(center).add_ore(1, 2);

    let mut robot = test_robot_at(Position::new(0.0, 0.0, 0), bounds);
    robot.target_mining[1] = 10;
    robot.time_fraction = 1.0;

    apply_mining(&mut ground, &mut robot, 1);

    assert_eq!(robot.ore_at(1), 2);
    assert_eq!(ground.at_position(center).ore_at(1), 0);
}

#[test]
fn process_dump_all_transfers_robot_cargo_to_ground() {
    let bounds = bounds_for_ground(5, 5, 1.0);
    let mut ground = Ground::new(5, 5);
    let mut robot = test_robot_at(Position::new(0.0, 0.0, 0), bounds);
    robot.add_ore(0, 5);
    robot.add_ore(2, 3);

    let (dumped, change) = process_dump(&mut ground, &mut robot, None, 7);

    assert_eq!(dumped, 8);
    assert_eq!(robot.total_ore(), 0);
    assert_eq!(ground.at(0, 0).ore_at(0), 5);
    assert_eq!(ground.at(0, 0).ore_at(2), 3);
    assert!(change.is_some());
    assert_eq!(change.unwrap().time, 7);
}

#[test]
fn process_dump_single_ore_type_leaves_other_cargo() {
    let bounds = bounds_for_ground(5, 5, 1.0);
    let mut ground = Ground::new(5, 5);
    let mut robot = test_robot_at(Position::new(1.0, 1.0, 0), bounds);
    robot.add_ore(0, 4);
    robot.add_ore(1, 2);

    let (dumped, change) = process_dump(&mut ground, &mut robot, Some(0), 2);

    assert_eq!(dumped, 4);
    assert_eq!(robot.ore_at(0), 0);
    assert_eq!(robot.ore_at(1), 2);
    assert_eq!(ground.at(1, 1).ore_at(0), 4);
    assert!(change.is_some());
}

#[test]
fn process_dump_empty_robot_returns_zero_without_change() {
    let bounds = bounds_for_ground(5, 5, 1.0);
    let mut ground = Ground::new(5, 5);
    let mut robot = test_robot_at(Position::new(0.0, 0.0, 0), bounds);

    let (dumped, change) = process_dump(&mut ground, &mut robot, Some(0), 1);

    assert_eq!(dumped, 0);
    assert!(change.is_none());
}

#[test]
fn process_dump_at_home_fills_depot_up_to_capacity() {
    let bounds = bounds_for_ground(5, 5, 1.0);
    let mut ground = Ground::new(5, 5);
    let mut robot = test_robot_at(Position::new(0.0, 0.0, 0), bounds);
    let center = robot.center_position();
    robot.initial_center_x = center.x;
    robot.initial_center_y = center.y;
    robot.set_depot_capacity({
        let mut capacity = [0; MAX_ORE_TYPES];
        capacity[0] = 10;
        capacity
    });
    robot.add_ore(0, 7);

    let (dumped, change) = process_dump(&mut ground, &mut robot, None, 3);

    assert_eq!(dumped, 7);
    assert_eq!(robot.ore_at(0), 0);
    assert_eq!(robot.depot()[0], 7);
    assert_eq!(ground.at(0, 0).ore_at(0), 0);
    assert!(change.is_none());
    assert_eq!(robot.result_ore()[0], 7);
    assert!(robot.calculate_score() > 0.0);
}

#[test]
fn process_dump_at_home_overflow_goes_to_ground() {
    let bounds = bounds_for_ground(5, 5, 1.0);
    let mut ground = Ground::new(5, 5);
    let mut robot = test_robot_at(Position::new(0.0, 0.0, 0), bounds);
    let center = robot.center_position();
    robot.initial_center_x = center.x;
    robot.initial_center_y = center.y;
    robot.set_depot_capacity({
        let mut capacity = [0; MAX_ORE_TYPES];
        capacity[0] = 3;
        capacity
    });
    robot.add_ore(0, 8);

    let (dumped, change) = process_dump(&mut ground, &mut robot, Some(0), 4);

    assert_eq!(dumped, 8);
    assert_eq!(robot.depot()[0], 3);
    assert_eq!(robot.ore_at(0), 0);
    assert_eq!(ground.at(0, 0).ore_at(0), 5);
    assert!(change.is_some());
    assert_eq!(robot.result_ore()[0], 3);
}

#[test]
fn process_dump_away_from_home_ignores_depot_capacity() {
    let bounds = bounds_for_ground(5, 5, 1.0);
    let mut ground = Ground::new(5, 5);
    let mut robot = test_robot_at(Position::new(2.0, 2.0, 0), bounds);
    robot.initial_center_x = 0.5;
    robot.initial_center_y = 0.5;
    robot.set_depot_capacity({
        let mut capacity = [0; MAX_ORE_TYPES];
        capacity[0] = 100;
        capacity
    });
    robot.add_ore(0, 6);

    let (dumped, change) = process_dump(&mut ground, &mut robot, None, 5);

    assert_eq!(dumped, 6);
    assert_eq!(robot.depot()[0], 0);
    assert_eq!(ground.at(2, 2).ore_at(0), 6);
    assert!(change.is_some());
}
