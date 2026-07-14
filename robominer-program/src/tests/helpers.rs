use crate::*;

pub(super) fn test_context(time_left: i32, action_result: Option<f64>) -> ExecutionContext {
    ExecutionContext::from_runtime(time_left, [0; 10], action_result)
}

pub(super) fn scan_context(
    time_left: i32,
    action_result: Option<f64>,
    scan_time: i32,
    started: bool,
    complete: bool,
    distance: f64,
    ore_type: f64,
) -> ExecutionContext {
    let mut context = ExecutionContext::from_runtime(time_left, [0; 10], action_result);
    context.scan_time = scan_time;
    context.scan_started = started;
    context.scan_complete = complete;
    context.scan_distance = distance;
    context.scan_ore_type = ore_type;
    context
}

pub(super) fn robot_context(cpu_speed: f64) -> ExecutionContext {
    let mut context = ExecutionContext::from_runtime(10, [0; 10], None);
    context.robot.cpu_speed = cpu_speed;
    context
}

pub(super) fn assert_valid_any_size(source: &str) {
    let result = verify_source(source);

    assert!(
        result.verified,
        "unexpected error: {}",
        result.error_description
    );
    assert!(result.compiled_size > 0);
    assert_eq!(result.error_description, "");
}
