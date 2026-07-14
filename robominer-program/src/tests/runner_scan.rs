use crate::*;

use super::helpers::*;

#[test]
fn executable_scan_starts_with_start_scan_action() {
    let program = compile_executable_source("scan();").expect("scan should compile");
    let mut runner = program.runner();
    let mut context = scan_context(10, None, 6, false, false, -1.0, 0.0);

    assert_eq!(
        runner.next_action(&mut context),
        Some(ExecutableAction::StartScan(0.0))
    );

    let mut after_start = scan_context(10, Some(6.0), 6, true, false, -1.0, 0.0);
    assert_eq!(runner.next_action(&mut after_start), None);
}

#[test]
fn executable_ore_distance_waits_for_in_progress_scan() {
    let program = compile_executable_source("oreDistance();").expect("oreDistance should compile");
    let mut runner = program.runner();
    let mut scanning = scan_context(10, None, 6, true, false, -1.0, 0.0);

    assert_eq!(
        runner.next_action(&mut scanning),
        Some(ExecutableAction::AwaitScanResult)
    );
}

#[test]
fn executable_search_loop_starts_with_scan() {
    let program = compile_executable_source(
        "scan(); while (oreType() == 0) { move(1); scan(); }",
    )
    .expect("search loop should compile");
    let mut runner = program.runner();

    let mut start = test_context(50, None);
    assert_eq!(
        runner.next_action(&mut start),
        Some(ExecutableAction::StartScan(0.0))
    );
}

#[test]
fn scan_with_direction_does_not_restart_as_forward_scan() {
    let program = compile_executable_source("scan(90);").expect("scan(90) should compile");
    let mut runner = program.runner();

    let mut context = test_context(10, None);
    assert_eq!(
        runner.next_action(&mut context),
        Some(ExecutableAction::StartScan(90.0))
    );

    let mut after_scan = test_context(10, Some(6.0));
    assert!(matches!(
        runner.step(&mut after_scan),
        ProgramStep::Cpu | ProgramStep::Done
    ));
    assert_eq!(runner.next_action(&mut test_context(10, None)), None);
    assert!(!runner.awaits_scan_result());
}

#[test]
fn executable_program_starts_scan_after_move() {
    let program = compile_executable_source("move(10); scan();")
        .expect("move then scan should compile");
    let mut runner = program.runner();

    let mut start = test_context(10, None);
    assert_eq!(
        runner.next_action(&mut start),
        Some(ExecutableAction::Move(10.0))
    );
    let mut after_move = test_context(10, Some(10.0));
    assert_eq!(
        runner.next_action(&mut after_move),
        Some(ExecutableAction::StartScan(0.0))
    );
}

#[test]
fn executable_search_loop_does_not_confuse_move_distance_with_ore_type() {
    let program = compile_executable_source(
        "scan(); while (oreType() == 0) { move(10); scan(); }",
    )
    .expect("search loop should compile");
    let mut runner = program.runner();

    let mut start = test_context(20, None);
    assert_eq!(
        runner.next_action(&mut start),
        Some(ExecutableAction::StartScan(0.0))
    );

    let mut after_scan = scan_context(20, Some(6.0), 6, true, true, -1.0, 0.0);
    assert_eq!(
        runner.next_action(&mut after_scan),
        Some(ExecutableAction::Move(10.0))
    );
}

#[test]
fn executable_program_else_if_scans_restart_scan_direction() {
    let program = compile_executable_source(
        "scan(); if (oreType() > 0) {} else { scan(30); }",
    )
    .expect("else-if scan chain should compile");
    let mut runner = program.runner();

    let mut context = test_context(10, None);
    assert_eq!(
        runner.next_action(&mut context),
        Some(ExecutableAction::StartScan(0.0))
    );

    let mut after_forward = scan_context(10, Some(6.0), 6, true, true, -1.0, 0.0);
    assert_eq!(
        runner.next_action(&mut after_forward),
        Some(ExecutableAction::StartScan(30.0))
    );
}
