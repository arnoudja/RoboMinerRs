use super::super::helpers::*;
use super::status_fixtures::{
    battery_status_animation_data, cpu_status_animation_data, no_chunk_status_animation_data,
    robot_status_animation_data, scan_status_animation_data, wait_status_animation_data,
    wall_status_animation_data, zero_status_animation_data,
};
use crate::*;

#[test]
fn animation_data_records_wait_action_index_on_idle_cycles() {
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
    let data = simulation.run_with_animation(&[]);

    assert!(
        data.contains(r#""a":1"#),
        "wait cycles should emit action index 1: {data}"
    );
}

#[test]
fn animation_data_records_scan_action_index_while_scanning() {
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
    let data = simulation.run_with_animation(&[]);

    assert!(
        data.contains(r#""a":0"#),
        "scan-busy wait cycles should emit action index 0: {data}"
    );
    let scan_marks = data.matches(r#""a":0"#).count();
    assert!(
        scan_marks >= 2,
        "expected multiple scan-busy cycles, found {scan_marks} in {data}"
    );
}

#[test]
fn animation_data_records_stuck_status_strings() {
    struct Case {
        name: &'static str,
        expected_status: &'static str,
        data: fn() -> String,
    }

    let cases = [
        Case {
            name: "wait",
            expected_status: "wait",
            data: wait_status_animation_data,
        },
        Case {
            name: "scan",
            expected_status: "scan",
            data: scan_status_animation_data,
        },
        Case {
            name: "zero",
            expected_status: "zero",
            data: zero_status_animation_data,
        },
        Case {
            name: "cpu",
            expected_status: "cpu",
            data: cpu_status_animation_data,
        },
        Case {
            name: "battery",
            expected_status: "battery",
            data: battery_status_animation_data,
        },
        Case {
            name: "motion",
            expected_status: "motion",
            data: no_chunk_status_animation_data,
        },
        Case {
            name: "wall",
            expected_status: "wall",
            data: wall_status_animation_data,
        },
        Case {
            name: "robot",
            expected_status: "robot",
            data: robot_status_animation_data,
        },
    ];

    for case in cases {
        let data = (case.data)();
        assert!(
            data.contains(&format!(r#""s":"{}""#, case.expected_status)),
            "{} should emit stuck status {}: {data}",
            case.name,
            case.expected_status
        );
    }
}

#[test]
fn animation_data_omits_wall_status_for_partial_wall_clip() {
    let mut spec = RobotSpec::test_robot();
    spec.forward_speed = 10.0;
    spec.max_turns = 1;

    let mut simulation = Simulation::new(
        Ground::new(5, 5),
        1,
        vec![ScriptedRobot::new(spec, vec![RobotAction::Forward])],
    );
    let data = simulation.run_with_animation(&[]);

    assert!(
        !data.contains(r#""s":"wall""#),
        "partial wall clip must not be labeled stuck: {data}"
    );
}
