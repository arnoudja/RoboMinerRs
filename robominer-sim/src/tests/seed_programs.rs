use super::helpers::*;
use crate::*;

#[test]
fn seeded_ai_1_moves_to_ore_and_mines_until_depleted() {
    let program = seed_ai_1();
    let mut ground = Ground::new(5, 5);
    ground.at_mut(1, 1).add_ore(0, 8);

    let spec = seeded_robot_spec(1, 6);

    let mut simulation = Simulation::new(
        ground,
        6,
        vec![ScriptedRobot::from_executable_program(spec, &program)],
    );

    simulation.run();

    let position = simulation.robot(0).position();
    assert_close(position.x, 1.5 * 45.0_f64.to_radians().cos());
    assert_close(position.y, 1.5 * 45.0_f64.to_radians().sin());
    assert_eq!(simulation.robot(0).ore_at(0), 8);
    assert_eq!(simulation.ground().at(1, 1).ore_at(0), 0);
    assert_eq!(simulation.robot(0).actions_done()[2], 1);
    assert_eq!(simulation.robot(0).actions_done()[6], 5);
}

#[test]
fn seeded_ai_2_mines_after_successful_probe_move() {
    let program = seed_ai_2();
    let mut ground = Ground::new(5, 5);
    ground.at_mut(1, 1).add_ore(0, 8);

    let mut spec = seeded_robot_spec(2, 6);
    spec.rotate_speed = 20;

    let mut simulation = Simulation::new(
        ground,
        6,
        vec![ScriptedRobot::from_executable_program(spec, &program)],
    );

    simulation.run();

    let position = simulation.robot(0).position();
    assert_close(position.x, 1.5 * 45.0_f64.to_radians().cos());
    assert_close(position.y, 1.5 * 45.0_f64.to_radians().sin());
    assert_eq!(simulation.robot(0).ore_at(0), 8);
    assert_eq!(simulation.ground().at(1, 1).ore_at(0), 0);
    assert_eq!(simulation.robot(0).actions_done()[2], 1);
    assert_eq!(simulation.robot(0).actions_done()[3], 0);
    assert_eq!(simulation.robot(0).actions_done()[6], 5);
}

#[test]
fn seeded_ai_2_uses_fallback_when_probe_move_is_blocked() {
    let program = seed_ai_2();

    let mut spec = seeded_robot_spec(2, 3);
    spec.rotate_speed = 20;

    let mut simulation = Simulation::new(
        Ground::new(1, 1),
        3,
        vec![ScriptedRobot::from_executable_program(spec, &program)],
    );

    simulation.run();

    let position = simulation.robot(0).position();
    assert_close(position.x, 0.0);
    assert_close(position.y, 0.0);
    assert_eq!(position.orientation, 65);
    assert_eq!(simulation.robot(0).actions_done()[2], 1);
    assert_eq!(simulation.robot(0).actions_done()[3], 1);
    assert_eq!(simulation.robot(0).actions_done()[4], 1);
    assert_eq!(simulation.robot(0).actions_done()[6], 0);
}

#[test]
fn seeded_ai_3_runs_nested_variable_loop_on_bounded_rally() {
    let program = seed_ai_3();
    let mut ground = Ground::new(8, 8);
    ground.at_mut(1, 1).add_ore(0, 8);

    let mut spec = seeded_robot_spec(3, 12);
    spec.rotate_speed = 30;

    let mut simulation = Simulation::new(
        ground,
        12,
        vec![ScriptedRobot::from_executable_program(spec, &program)],
    );

    simulation.run();

    assert_eq!(simulation.robot(0).ore_at(0), 8);
    assert_eq!(simulation.ground().at(1, 1).ore_at(0), 0);
    assert!(simulation.robot(0).actions_done()[2] >= 2);
    assert!(simulation.robot(0).actions_done()[4] >= 1);
    assert!(simulation.robot(0).actions_done()[6] >= 3);
}

#[test]
fn seeded_multi_robot_rally_mines_scores_and_avoids_overlap() {
    let seed_1 = seed_ai_1();
    let seed_2 = seed_ai_2();
    let seed_3 = seed_ai_3();
    let mut ground = Ground::new(8, 8);
    ground.at_mut(1, 1).add_ore(0, 8);
    ground.at_mut(1, 6).add_ore(1, 12);
    ground.at_mut(6, 1).add_ore(2, 20);

    let mut simulation = Simulation::new(
        ground,
        14,
        vec![
            ScriptedRobot::from_executable_program(seeded_robot_spec(1, 14), &seed_1),
            ScriptedRobot::from_executable_program(seeded_robot_spec(2, 14), &seed_2),
            ScriptedRobot::from_executable_program(seeded_robot_spec(3, 14), &seed_3),
        ],
    );

    simulation.run();

    assert_eq!(simulation.time(), 14);
    assert_eq!(simulation.robot(0).ore_at(0), 8);
    assert_eq!(simulation.robot(1).ore_at(1), 12);
    assert!(simulation.robot(2).ore_at(2) > 0);
    assert_eq!(simulation.ground().at(1, 1).ore_at(0), 0);
    assert_eq!(simulation.ground().at(1, 6).ore_at(1), 0);
    assert!(simulation.robot(0).calculate_score() > 0.0);
    assert!(simulation.robot(1).calculate_score() > 0.0);
    assert!(simulation.robot(2).calculate_score() > 0.0);

    for robot in 0..3 {
        assert!(
            simulation.robot(robot).actions_done()[2] > 0,
            "robot {robot} should move"
        );
        assert!(
            simulation.robot(robot).actions_done()[6] > 0,
            "robot {robot} should mine"
        );
    }

    for first in 0..2 {
        for second in (first + 1)..3 {
            let distance = simulation
                .robot(first)
                .position()
                .distance(&simulation.robot(second).position());
            assert!(
                distance >= 0.99,
                "robots {first} and {second} overlap at distance {distance}"
            );
        }
    }
}
