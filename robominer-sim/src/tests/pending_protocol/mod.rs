mod move_tests;
mod scan_tests;

pub(super) fn runtime_move_program(source: &str) -> robominer_program::ExecutableProgram {
    let program = robominer_program::compile_executable_source(source)
        .unwrap_or_else(|err| panic!("{source} should compile: {err}"));
    assert!(
        program.requires_runtime(),
        "{source} should use the runtime program bridge"
    );
    program
}

pub(super) fn protocol_simulation(source: &str, max_turns: i32) -> crate::Simulation {
    let mut spec = crate::RobotSpec::test_robot();
    spec.forward_speed = 1.0;
    spec.backward_speed = 1.0;
    spec.rotate_speed = 90;
    spec.max_turns = max_turns;

    crate::Simulation::new(
        crate::Ground::new(10, 10),
        max_turns,
        vec![crate::ScriptedRobot::from_executable_program(
            spec,
            &runtime_move_program(source),
        )],
    )
}
