use super::super::helpers::*;
use crate::*;

pub(super) fn wait_status_animation_data() -> String {
    let ground = Ground::new(4, 4);
    let mut spec = RobotSpec::test_robot();
    spec.max_turns = 2;

    let mut simulation = Simulation::new(
        ground,
        2,
        vec![ScriptedRobot::new(
            spec,
            vec![RobotAction::Wait, RobotAction::Wait],
        )],
    );
    simulation.run_with_animation(&[])
}

pub(super) fn scan_status_animation_data() -> String {
    let source = "scan(); if (oreDistance() < 0) { rotate(0); } mine();";
    let program = seeded_program(source);

    let mut ground = Ground::new(10, 10);
    ground.add_ore_heap(4, 4, 0, 2, 2);

    let mut spec = RobotSpec::test_robot();
    spec.cpu_speed = 1;
    spec.scan_time = 6;
    spec.scan_distance = 50;
    spec.max_turns = 4;

    let mut simulation = Simulation::new(
        ground,
        4,
        vec![ScriptedRobot::from_executable_program(spec, &program)],
    );
    simulation.run_with_animation(&[])
}

pub(super) fn zero_status_animation_data() -> String {
    let program = seeded_program("move(0); rotate(0);");
    let mut spec = RobotSpec::test_robot();
    spec.max_turns = 3;
    spec.cpu_speed = 72;

    let mut simulation = Simulation::new(
        Ground::new(4, 4),
        3,
        vec![ScriptedRobot::from_executable_program(spec, &program)],
    );
    simulation.run_with_animation(&[])
}

pub(super) fn cpu_status_animation_data() -> String {
    let program = seeded_program("int x = 1; int y = 2; mine();");
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
    simulation.run_with_animation(&[])
}

pub(super) fn battery_status_animation_data() -> String {
    let ground = Ground::new(4, 4);
    let mut short = RobotSpec::test_robot();
    short.max_turns = 1;
    let mut long = RobotSpec::test_robot();
    long.max_turns = 3;

    let mut simulation = Simulation::new(
        ground,
        3,
        vec![
            ScriptedRobot::new(short, vec![RobotAction::Wait; 3]),
            ScriptedRobot::new(long, vec![RobotAction::Wait; 3]),
        ],
    );
    simulation.run_with_animation(&[])
}

pub(super) fn no_chunk_status_animation_data() -> String {
    let program = seeded_program("move(1);");
    let mut spec = RobotSpec::test_robot();
    spec.forward_speed = 0.0;
    spec.max_turns = 2;
    spec.cpu_speed = 72;

    let mut simulation = Simulation::new(
        Ground::new(4, 4),
        2,
        vec![ScriptedRobot::from_executable_program(spec, &program)],
    );
    simulation.run_with_animation(&[])
}

pub(super) fn wall_status_animation_data() -> String {
    let mut spec = RobotSpec::test_robot();
    spec.max_turns = 1;
    spec.backward_speed = 1.0;

    let mut simulation = Simulation::new(
        Ground::new(4, 4),
        1,
        vec![ScriptedRobot::new(spec, vec![RobotAction::Backward])],
    );
    simulation.run_with_animation(&[])
}

pub(super) fn robot_status_animation_data() -> String {
    let mut mover = RobotSpec::test_robot();
    mover.forward_speed = 1.0;
    mover.robot_size = 1.0;
    mover.max_turns = 1;

    let mut blocker = RobotSpec::test_robot();
    blocker.robot_size = 1.0;
    blocker.max_turns = 1;

    let mut simulation = Simulation::new(
        Ground::new(2, 2),
        1,
        vec![
            ScriptedRobot::new(mover, vec![RobotAction::Forward]),
            ScriptedRobot::new(blocker, vec![RobotAction::Wait]),
        ],
    );
    simulation.run_with_animation(&[])
}
