use crate::MAX_ORE_TYPES;
use crate::ground::Ground;
use crate::position::Position;
use crate::robot::Robot;

#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) enum ActionResult {
    None,
    Mine,
    Value(f64),
    Move { direction: f64 },
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) struct GroundAnimationChange {
    pub(crate) x: usize,
    pub(crate) y: usize,
    pub(crate) time: i32,
    pub(crate) ore: [i32; MAX_ORE_TYPES],
}

pub(crate) fn process_move(robot: &mut Robot, speed: f64, time_fraction: f64) {
    robot.destination = robot
        .position
        .calculate_move_position(speed * time_fraction);
    robot.time_fraction = time_fraction;
    robot.current_speed = speed;
}

pub(crate) fn process_requested_move(robot: &mut Robot, distance: f64) {
    if distance > 0.0 {
        let time_fraction = (distance / robot.spec.forward_speed).min(1.0);
        process_move(robot, robot.spec.forward_speed, time_fraction);
    } else if distance < 0.0 {
        let time_fraction = (-distance / robot.spec.backward_speed).min(1.0);
        process_move(robot, -robot.spec.backward_speed, time_fraction);
    }
}

pub(crate) fn process_requested_rotation(robot: &mut Robot, rotation: f64) {
    if rotation > 0.0 {
        robot.time_fraction = (rotation / robot.spec.rotate_speed as f64).min(1.0);
        robot.target_rotation = robot.spec.rotate_speed;
    } else if rotation < 0.0 {
        robot.time_fraction = (-rotation / robot.spec.rotate_speed as f64).min(1.0);
        robot.target_rotation = -robot.spec.rotate_speed;
    }
}

pub(crate) fn find_collision_time(
    first_robot: &Robot,
    second_robot: &Robot,
    min_collision_time: f64,
    max_collision_time: f64,
) -> f64 {
    debug_assert!(min_collision_time <= max_collision_time);

    let first_start = position_at_time(
        first_robot.position,
        first_robot.destination,
        first_robot.time_fraction,
        min_collision_time.min(first_robot.time_fraction),
    );
    let second_start = position_at_time(
        second_robot.position,
        second_robot.destination,
        second_robot.time_fraction,
        min_collision_time.min(second_robot.time_fraction),
    );
    let first_end = position_at_time(
        first_robot.position,
        first_robot.destination,
        first_robot.time_fraction,
        max_collision_time.min(first_robot.time_fraction),
    );
    let second_end = position_at_time(
        second_robot.position,
        second_robot.destination,
        second_robot.time_fraction,
        max_collision_time.min(second_robot.time_fraction),
    );

    let start_distance = first_start.distance(&second_start)
        - first_robot.spec.robot_size / 2.0
        - second_robot.spec.robot_size / 2.0;
    let end_distance = first_end.distance(&second_end)
        - first_robot.spec.robot_size / 2.0
        - second_robot.spec.robot_size / 2.0;

    let first_travel_distance = first_start.distance(&first_end);
    let second_travel_distance = second_start.distance(&second_end);
    let total_travel_distance = first_travel_distance + second_travel_distance;

    if total_travel_distance > start_distance && total_travel_distance > end_distance {
        let first_speed = if first_robot.time_fraction > min_collision_time {
            first_robot.current_speed.abs()
        } else {
            0.0
        };
        let second_speed = if second_robot.time_fraction > min_collision_time {
            second_robot.current_speed.abs()
        } else {
            0.0
        };
        let total_speed = first_speed + second_speed;

        if total_speed > 0.0 {
            let min_collision_time_increase = 0.0_f64.max(start_distance / total_speed);
            let max_collision_time_decrease = 0.0_f64.max(end_distance / total_speed);

            if min_collision_time_increase + max_collision_time_decrease
                > max_collision_time - min_collision_time
            {
                // No collision.
            } else if min_collision_time_increase > 0.01 || max_collision_time_decrease > 0.1 {
                return find_collision_time(
                    first_robot,
                    second_robot,
                    min_collision_time + min_collision_time_increase,
                    max_collision_time - max_collision_time_decrease,
                );
            } else {
                let first_test = position_at_time(
                    first_robot.position,
                    first_robot.destination,
                    first_robot.time_fraction,
                    (min_collision_time + 0.01).min(first_robot.time_fraction),
                );
                let second_test = position_at_time(
                    second_robot.position,
                    second_robot.destination,
                    second_robot.time_fraction,
                    (min_collision_time + 0.01).min(second_robot.time_fraction),
                );
                let test_distance = first_test.distance(&second_test)
                    - first_robot.spec.robot_size / 2.0
                    - second_robot.spec.robot_size / 2.0;

                if test_distance < 0.0 && test_distance < start_distance {
                    return min_collision_time + min_collision_time_increase;
                }

                return find_collision_time(
                    first_robot,
                    second_robot,
                    min_collision_time + 0.01,
                    (max_collision_time - max_collision_time_decrease)
                        .max(min_collision_time + 0.01),
                );
            }
        }
    }

    1.0
}

pub(crate) fn position_at_time(
    start_position: Position,
    end_position: Position,
    travel_time: f64,
    time: f64,
) -> Position {
    debug_assert!(time <= travel_time);

    let mut position = start_position;

    if time > 0.0 && travel_time > 0.0 {
        position.x += (end_position.x - start_position.x) * time / travel_time;
        position.y += (end_position.y - start_position.y) * time / travel_time;
    }

    position
}

pub(crate) fn check_wall_collision(robot: &mut Robot) {
    let old_position = robot.position;
    let mut new_position = robot.destination;

    if new_position.x < robot.min_x {
        if old_position.x <= robot.min_x {
            new_position = old_position;
        } else {
            let target_delta_x = old_position.x - new_position.x;
            let real_delta_x = old_position.x - robot.min_x;

            if target_delta_x > 0.01 {
                let relative = real_delta_x / target_delta_x;
                let target_delta_y = new_position.y - old_position.y;
                let real_delta_y = relative * target_delta_y;

                new_position.y = old_position.y + real_delta_y;
            }

            new_position.x = robot.min_x;
        }
    } else if new_position.x > robot.max_x {
        if old_position.x >= robot.max_x {
            new_position = old_position;
        } else {
            let target_delta_x = new_position.x - old_position.x;
            let real_delta_x = robot.max_x - old_position.x;

            if target_delta_x > 0.01 {
                let relative = real_delta_x / target_delta_x;
                let target_delta_y = new_position.y - old_position.y;
                let real_delta_y = relative * target_delta_y;

                new_position.y = old_position.y + real_delta_y;
            }

            new_position.x = robot.max_x;
        }
    }

    if new_position.y < robot.min_y {
        if old_position.y <= robot.min_y {
            new_position = old_position;
        } else {
            let target_delta_y = old_position.y - new_position.y;
            let real_delta_y = old_position.y - robot.min_y;

            if target_delta_y > 0.01 {
                let relative = real_delta_y / target_delta_y;
                let target_delta_x = new_position.x - old_position.x;
                let real_delta_x = relative * target_delta_x;

                new_position.x = old_position.x + real_delta_x;
            }

            new_position.y = robot.min_y;
        }
    } else if new_position.y > robot.max_y {
        if old_position.y >= robot.max_y {
            new_position = old_position;
        } else {
            let target_delta_y = new_position.y - old_position.y;
            let real_delta_y = robot.max_y - old_position.y;

            if target_delta_y > 0.01 {
                let relative = real_delta_y / target_delta_y;
                let target_delta_x = new_position.x - old_position.x;
                let real_delta_x = relative * target_delta_x;

                new_position.x = old_position.x + real_delta_x;
            }

            new_position.y = robot.max_y;
        }
    }

    if new_position != robot.destination {
        let target_distance = old_position.distance(&robot.destination);
        let actual_distance = old_position.distance(&new_position);

        robot.destination = new_position;

        if new_position == old_position {
            robot.time_fraction = 0.0;
        } else if target_distance > 0.0 {
            robot.time_fraction *= actual_distance / target_distance;
        }
    }
}

pub(crate) fn apply_mining(
    ground: &mut Ground,
    robot: &mut Robot,
    time: i32,
) -> Option<GroundAnimationChange> {
    let position = robot.center_position();
    let x = position.x as usize;
    let y = position.y as usize;
    let ground_unit = ground.at_position_mut(position);
    let mut mined = false;

    for ore_type in 0..MAX_ORE_TYPES {
        let mining_amount = ((robot.target_mining[ore_type] as f64) * robot.time_fraction) as i32;

        let mining_amount = mining_amount.min(ground_unit.ore_at(ore_type));

        if mining_amount > 0 {
            ground_unit.remove_ore(ore_type, mining_amount);
            robot.add_ore(ore_type, mining_amount);
            mined = true;
        }
    }

    mined.then(|| GroundAnimationChange {
        x,
        y,
        time,
        ore: *ground_unit.ore(),
    })
}

pub(crate) fn process_dump(
    ground: &mut Ground,
    robot: &mut Robot,
    ore_type: Option<usize>,
    time: i32,
) -> (i32, Option<GroundAnimationChange>) {
    let position = robot.center_position();
    let x = position.x as usize;
    let y = position.y as usize;
    let ground_unit = ground.at_position_mut(position);
    let mut dumped = 0;

    match ore_type {
        Some(ore_type) if ore_type < MAX_ORE_TYPES => {
            if robot.ore_at(ore_type) > 0 {
                dumped = robot.ore_at(ore_type);
                ground_unit.add_ore(ore_type, dumped);
                robot.clear_ore(ore_type);
            }
        }
        _ => {
            for ore_type in 0..MAX_ORE_TYPES {
                if robot.ore_at(ore_type) > 0 {
                    dumped += robot.ore_at(ore_type);
                    ground_unit.add_ore(ore_type, robot.ore_at(ore_type));
                    robot.clear_ore(ore_type);
                }
            }
        }
    }

    let change = (dumped > 0).then(|| GroundAnimationChange {
        x,
        y,
        time,
        ore: *ground_unit.ore(),
    });

    (dumped, change)
}
