use super::super::helpers::*;
use crate::*;

#[test]
fn animation_data_empty_program_finishes_without_hanging() {
    let program = seeded_program("{}");
    let ground = Ground::new(4, 4);

    let mut spec = RobotSpec::test_robot();
    spec.max_turns = 3;
    spec.cpu_speed = 4;

    let mut simulation = Simulation::new(
        ground,
        3,
        vec![ScriptedRobot::from_executable_program(spec, &program)],
    );
    let data = simulation.run_with_animation(&[]);
    let payload: serde_json::Value =
        serde_json::from_str(&data).expect("empty program animation should finish");
    let locations = payload["robots"]["robot"][0]["locations"]
        .as_array()
        .expect("robot locations");
    assert!(
        locations.len() >= 2,
        "empty program should still record turn samples: {data}"
    );
}

#[test]
fn animation_data_empty_program_stays_finite_under_high_cpu_speed() {
    let program = seeded_program("{}");
    let ground = Ground::new(4, 4);

    let mut spec = RobotSpec::test_robot();
    spec.max_turns = 5;
    spec.cpu_speed = 10_000;

    let mut simulation = Simulation::new(
        ground,
        5,
        vec![ScriptedRobot::from_executable_program(spec, &program)],
    );
    let data = simulation.run_with_animation(&[]);
    let payload: serde_json::Value =
        serde_json::from_str(&data).expect("high-cpu empty program should finish");
    let locations = payload["robots"]["robot"][0]["locations"]
        .as_array()
        .expect("robot locations");
    assert_eq!(
        locations.len(),
        6,
        "Done must charge CPU so empty programs cannot livelock: {data}"
    );
}
