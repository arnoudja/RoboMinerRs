use crate::*;

use super::helpers::*;

#[test]
fn statement_move_reemits_until_action_result_is_provided() {
    let program = compile_executable_source("move(2); mine();").expect("program should compile");
    let mut runner = program.runner();
    let mut context = test_context(5, None);

    assert_eq!(
        runner.step(&mut context),
        ProgramStep::Action(ExecutableAction::Move(2.0))
    );
    assert!(runner.awaits_action_result());

    assert_eq!(
        runner.step(&mut test_context(5, None)),
        ProgramStep::Action(ExecutableAction::Move(2.0))
    );

    let mut after_move = test_context(5, Some(2.0));
    assert_eq!(
        runner.next_action(&mut after_move),
        Some(ExecutableAction::Mine)
    );
}

#[test]
fn statement_move_blocked_still_advances_to_next_statement() {
    let program = compile_executable_source("move(1); mine();").expect("program should compile");
    let mut runner = program.runner();

    let mut start = test_context(5, None);
    assert_eq!(
        runner.step(&mut start),
        ProgramStep::Action(ExecutableAction::Move(1.0))
    );

    let mut blocked = test_context(5, Some(0.0));
    assert_eq!(
        runner.next_action(&mut blocked),
        Some(ExecutableAction::Mine)
    );
}

#[test]
fn expression_move_condition_receives_traveled_distance() {
    let program = compile_executable_source("if (move(2) >= 1) { mine(); } else { rotate(90); }")
        .expect("program should compile");
    let mut runner = program.runner();
    let mut context = test_context(5, None);

    assert!(matches!(
        runner.step(&mut context),
        ProgramStep::Action(ExecutableAction::Move(2.0))
    ));

    context.action_result = Some(2.0);
    assert_eq!(
        runner.next_action(&mut context),
        Some(ExecutableAction::Mine)
    );
}

#[test]
fn expression_move_blocked_selects_false_branch() {
    let program = compile_executable_source("if (move(2) >= 1) { mine(); } else { rotate(90); }")
        .expect("program should compile");
    let mut runner = program.runner();
    let mut context = test_context(5, None);

    assert!(matches!(
        runner.step(&mut context),
        ProgramStep::Action(ExecutableAction::Move(2.0))
    ));

    context.action_result = Some(0.0);
    assert_eq!(
        runner.next_action(&mut context),
        Some(ExecutableAction::Rotate(90.0))
    );
}

#[test]
fn while_move_statement_reemits_until_action_result_completes() {
    let program = compile_executable_source("while (move(1) >= 1) { mine(); }")
        .expect("program should compile");
    let mut runner = program.runner();
    let mut context = test_context(5, None);

    assert_eq!(
        runner.step(&mut context),
        ProgramStep::Action(ExecutableAction::Move(1.0))
    );

    context.action_result = Some(1.0);
    assert_eq!(
        runner.next_action(&mut context),
        Some(ExecutableAction::Mine)
    );
}

#[test]
fn runner_keeps_pending_program_motion_while_action_result_is_missing() {
    let program = compile_executable_source("move(2);").expect("program should compile");
    let mut runner = program.runner();

    let mut context = test_context(5, None);
    assert!(matches!(
        runner.step(&mut context),
        ProgramStep::Action(ExecutableAction::Move(2.0))
    ));
    assert!(runner.has_pending_program_motion());

    let mut still_pending = test_context(5, None);
    assert!(matches!(
        runner.step(&mut still_pending),
        ProgramStep::Action(ExecutableAction::Move(2.0))
    ));
    assert!(runner.has_pending_program_motion());
}

#[test]
fn statement_move_zero_advances_without_awaiting_result() {
    let program = compile_executable_source("move(0); mine();").expect("program should compile");
    let mut runner = program.runner();
    let mut context = test_context(5, None);

    assert_eq!(
        runner.step(&mut context),
        ProgramStep::Action(ExecutableAction::Move(0.0))
    );
    assert!(!runner.awaits_action_result());
    assert!(!runner.has_pending_program_motion());

    assert_eq!(
        runner.next_action(&mut context),
        Some(ExecutableAction::Mine)
    );
}

#[test]
fn expression_move_zero_completes_immediately_without_pending() {
    let program = compile_executable_source("if (move(0) == 0) { mine(); } else { rotate(90); }")
        .expect("program should compile");
    let mut runner = program.runner();
    let mut context = test_context(5, None);

    assert_eq!(
        runner.next_action(&mut context),
        Some(ExecutableAction::Mine)
    );
    assert!(!runner.has_pending_program_motion());
}

#[test]
fn dynamic_statement_move_zero_advances_to_next_statement() {
    let program =
        compile_executable_source("int d = 0; move(d); mine();").expect("program should compile");
    let mut runner = program.runner();
    let mut context = test_context(5, None);

    loop {
        match runner.step(&mut context) {
            ProgramStep::Action(ExecutableAction::Move(0.0)) => break,
            ProgramStep::Cpu => {}
            other => panic!("unexpected step before move(0): {other:?}"),
        }
    }
    assert!(!runner.has_pending_program_motion());

    assert_eq!(
        runner.next_action(&mut context),
        Some(ExecutableAction::Mine)
    );
}

#[test]
fn dynamic_expression_move_zero_from_variable_selects_true_branch() {
    let program =
        compile_executable_source("int d = 0; if (move(d) == 0) { mine(); } else { rotate(90); }")
            .expect("program should compile");
    let mut runner = program.runner();
    let mut context = test_context(5, None);

    assert_eq!(
        runner.next_action(&mut context),
        Some(ExecutableAction::Mine)
    );
    assert!(!runner.has_pending_program_motion());
}

#[test]
fn dynamic_expression_move_below_epsilon_completes_without_pending() {
    // MOTION_EPSILON is 1e-6; values at or below it must not start pending motion.
    let program = compile_executable_source(
        "int d = 0.000001; if (move(d) == 0) { mine(); } else { rotate(90); }",
    )
    .expect("program should compile");
    let mut runner = program.runner();
    let mut context = test_context(5, None);

    assert_eq!(
        runner.next_action(&mut context),
        Some(ExecutableAction::Mine)
    );
    assert!(!runner.has_pending_program_motion());
}

#[test]
fn statement_move_completion_records_travel_step_result() {
    let program = compile_executable_source("move(2);").expect("program should compile");
    let mut runner = program.runner();
    assert_eq!(
        runner.step(&mut test_context(5, None)),
        ProgramStep::Action(ExecutableAction::Move(2.0))
    );
    assert!(runner.take_last_step_result().is_none());

    assert_eq!(
        runner.step(&mut test_context(5, Some(2.0))),
        ProgramStep::Cpu
    );
    let result = runner
        .take_last_step_result()
        .expect("statement move completion should expose travel as r");
    assert!((result.wire_value() - 2.0).abs() < 1e-9);
}

#[test]
fn dynamic_move_action_issue_omits_step_result() {
    let program =
        compile_executable_source("double d = 1.5; move(d);").expect("program should compile");
    let mut runner = program.runner();
    let mut context = test_context(5, None);
    for _ in 0..32 {
        let step = runner.step(&mut context);
        let result = runner.take_last_step_result();
        if let ProgramStep::Action(ExecutableAction::Move(distance)) = step {
            assert!((distance - 1.5).abs() < 1e-9);
            assert!(
                result.is_none(),
                "Action issue must not expose the move argument as r"
            );
            let span = runner
                .take_last_step_span()
                .expect("dynamic move issue should expose the call span");
            assert!(
                span.has_columns() && span.end_col - span.start_col > 1,
                "call span should cover move(d), not only the identifier: {span:?}"
            );
            return;
        }
    }
    panic!("expected dynamic move Action issue");
}
