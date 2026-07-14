use crate::*;

pub(super) fn assert_close(actual: f64, expected: f64) {
    assert!(
        (actual - expected).abs() < 0.000_001,
        "expected {expected}, got {actual}"
    );
}

pub(super) fn robot_with_actions(actions: Vec<RobotAction>) -> ScriptedRobot {
    ScriptedRobot::new(RobotSpec::test_robot(), actions)
}

pub(super) fn seeded_robot_spec(robot_id: i32, max_turns: i32) -> RobotSpec {
    let mut spec = RobotSpec::test_robot();
    spec.robot_id = robot_id;
    spec.max_turns = max_turns;
    spec.max_ore = 100;
    spec.mining_speed = 4;
    spec.cpu_speed = 4;
    spec.forward_speed = 1.5;
    spec.backward_speed = 1.0;
    spec.rotate_speed = 24;
    spec.robot_size = 1.0;
    spec
}

pub(super) fn seeded_program(source: &str) -> robominer_program::ExecutableProgram {
    robominer_program::compile_executable_source(source)
        .expect("seeded program should compile to executable program")
}

pub(super) fn seed_ai_1() -> robominer_program::ExecutableProgram {
    seeded_program(robominer_program::compatibility_fixture_source("seed_ai_1"))
}

pub(super) fn seed_ai_2() -> robominer_program::ExecutableProgram {
    seeded_program(robominer_program::compatibility_fixture_source("seed_ai_2"))
}

pub(super) fn seed_ai_3() -> robominer_program::ExecutableProgram {
    seeded_program(robominer_program::compatibility_fixture_source("seed_ai_3"))
}
