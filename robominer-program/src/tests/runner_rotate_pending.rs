use crate::*;

use super::helpers::*;

#[test]
fn statement_rotate_reemits_until_action_result_is_provided() {
    let program =
        compile_executable_source("rotate(180); mine();").expect("program should compile");
    let mut runner = program.runner();
    let mut context = test_context(5, None);

    assert_eq!(
        runner.step(&mut context),
        ProgramStep::Action(ExecutableAction::Rotate(180.0))
    );
    assert!(runner.has_pending_physical());

    assert_eq!(
        runner.step(&mut test_context(5, None)),
        ProgramStep::Action(ExecutableAction::Rotate(180.0))
    );
    assert!(runner.has_pending_physical());

    let mut after_rotate = test_context(5, Some(180.0));
    assert_eq!(
        runner.next_action(&mut after_rotate),
        Some(ExecutableAction::Mine)
    );
    assert!(!runner.has_pending_physical());
}

#[test]
fn dynamic_rotate_in_expression_reemits_until_action_result_is_provided() {
    let program = compile_executable_source("int rot = 180; if (rotate(rot) == 180) { mine(); }")
        .expect("program should compile");
    let mut runner = program.runner();
    let mut context = test_context(5, None);

    loop {
        match runner.step(&mut context) {
            ProgramStep::Action(ExecutableAction::Rotate(180.0)) => break,
            ProgramStep::Cpu => {}
            other => panic!("unexpected step before rotate: {other:?}"),
        }
    }
    assert!(runner.has_pending_physical());

    assert_eq!(
        runner.step(&mut test_context(5, None)),
        ProgramStep::Action(ExecutableAction::Rotate(180.0))
    );

    let mut after_rotate = test_context(5, Some(180.0));
    assert_eq!(
        runner.next_action(&mut after_rotate),
        Some(ExecutableAction::Mine)
    );
}

#[test]
fn expression_rotate_condition_receives_traveled_angle() {
    let program =
        compile_executable_source("if (rotate(180) == 180) { mine(); } else { move(1); }")
            .expect("program should compile");
    let mut runner = program.runner();
    let mut context = test_context(5, None);

    assert!(matches!(
        runner.step(&mut context),
        ProgramStep::Action(ExecutableAction::Rotate(180.0))
    ));
    assert!(runner.has_pending_physical());

    context.action_result = Some(180.0);
    assert_eq!(
        runner.next_action(&mut context),
        Some(ExecutableAction::Mine)
    );
}

#[test]
fn expression_rotate_blocked_selects_false_branch() {
    let program =
        compile_executable_source("if (rotate(180) == 180) { mine(); } else { move(1); }")
            .expect("program should compile");
    let mut runner = program.runner();
    let mut context = test_context(5, None);

    assert!(matches!(
        runner.step(&mut context),
        ProgramStep::Action(ExecutableAction::Rotate(180.0))
    ));

    context.action_result = Some(0.0);
    assert_eq!(
        runner.next_action(&mut context),
        Some(ExecutableAction::Move(1.0))
    );
}

#[test]
fn expression_rotate_zero_completes_immediately_without_pending() {
    let program = compile_executable_source("if (rotate(0) == 0) { mine(); } else { move(1); }")
        .expect("program should compile");
    let mut runner = program.runner();
    let mut context = test_context(5, None);

    assert_eq!(
        runner.next_action(&mut context),
        Some(ExecutableAction::Mine)
    );
    assert!(!runner.has_pending_physical());
}

#[test]
fn dynamic_statement_rotate_zero_advances_to_next_statement() {
    let program =
        compile_executable_source("int r = 0; rotate(r); mine();").expect("program should compile");
    let mut runner = program.runner();
    let mut context = test_context(5, None);

    loop {
        match runner.step(&mut context) {
            ProgramStep::Action(ExecutableAction::Rotate(0.0)) => break,
            ProgramStep::Cpu => {}
            other => panic!("unexpected step before rotate(0): {other:?}"),
        }
    }
    assert!(!runner.has_pending_physical());

    assert_eq!(
        runner.next_action(&mut context),
        Some(ExecutableAction::Mine)
    );
}
