use crate::*;

use super::helpers::*;

#[test]
fn expression_operator_precedence_multiplies_before_adding() {
    let program = compile_executable_source("dump(1 + 2 * 3);").expect("program should compile");
    let mut runner = program.runner();
    let mut context = test_context(5, None);

    assert_eq!(
        runner.next_action(&mut context),
        Some(ExecutableAction::Dump(7))
    );
}

#[test]
fn expression_logical_and_requires_both_operands() {
    let program = compile_executable_source("if (1 && 0) { mine(); } else { rotate(90); }")
        .expect("program should compile");
    let mut runner = program.runner();
    let mut context = test_context(5, None);

    assert_eq!(
        runner.next_action(&mut context),
        Some(ExecutableAction::Rotate(90.0))
    );
}

#[test]
fn expression_logical_or_short_circuits_to_true() {
    let program = compile_executable_source("if (0 || 1) { mine(); } else { rotate(90); }")
        .expect("program should compile");
    let mut runner = program.runner();
    let mut context = test_context(5, None);

    assert_eq!(
        runner.next_action(&mut context),
        Some(ExecutableAction::Mine)
    );
}

#[test]
fn expression_nested_if_resumes_after_move_in_condition() {
    let program =
        compile_executable_source("if (move(1) >= 1) { if (rotate(90) == 90) { mine(); } }")
            .expect("program should compile");
    let mut runner = program.runner();
    let mut context = test_context(5, None);

    assert_eq!(
        runner.next_action(&mut context),
        Some(ExecutableAction::Move(1.0))
    );

    let mut after_move = test_context(5, Some(1.0));
    assert_eq!(
        runner.next_action(&mut after_move),
        Some(ExecutableAction::Rotate(90.0))
    );

    let mut after_rotate = test_context(5, Some(90.0));
    assert_eq!(
        runner.next_action(&mut after_rotate),
        Some(ExecutableAction::Mine)
    );
}

#[test]
fn expression_while_condition_reevaluates_after_body() {
    let program =
        compile_executable_source("int count = 0; while (count < 2) { count++; mine(); }")
            .expect("program should compile");
    let mut runner = program.runner();
    let mut context = test_context(10, None);

    assert_eq!(
        runner.next_action(&mut context),
        Some(ExecutableAction::Mine)
    );
    context.action_result = Some(1.0);

    assert_eq!(
        runner.next_action(&mut context),
        Some(ExecutableAction::Mine)
    );
}

#[test]
fn expression_ore_reads_amount_after_scan_context() {
    let program =
        compile_executable_source("scan(); dump(ore(0));").expect("program should compile");
    let mut runner = program.runner();
    let mut context = scan_context(10, None, 6, true, true, 3.0, 1.0);
    context.ore[0] = 4;

    loop {
        match runner.step(&mut context) {
            ProgramStep::Action(ExecutableAction::StartScan(0.0)) => {
                context.action_result = Some(6.0);
            }
            ProgramStep::Action(ExecutableAction::Dump(4)) => break,
            ProgramStep::Cpu => {}
            other => panic!("unexpected step: {other:?}"),
        }
    }
}

#[test]
fn expression_ore_stored_properties_match_deprecated_ore_query() {
    let program = compile_executable_source(
        "dump(robot.oreStored); dump(robot.oreStoredA); dump(robot.oreStoredB); dump(robot.oreStoredC);",
    )
    .expect("program should compile");
    let mut runner = program.runner();
    let mut context = test_context(8, None);
    context.ore[0] = 5;
    context.ore[1] = 2;
    context.ore[2] = 1;

    assert_eq!(
        runner.next_action(&mut context),
        Some(ExecutableAction::Dump(8))
    );
    context.action_result = Some(8.0);
    assert_eq!(
        runner.next_action(&mut context),
        Some(ExecutableAction::Dump(5))
    );
    context.action_result = Some(5.0);
    assert_eq!(
        runner.next_action(&mut context),
        Some(ExecutableAction::Dump(2))
    );
    context.action_result = Some(2.0);
    assert_eq!(
        runner.next_action(&mut context),
        Some(ExecutableAction::Dump(1))
    );
}

#[test]
fn expression_unary_not_in_while_condition() {
    let program = compile_executable_source("int done = 0; while (!done) { done = 1; mine(); }")
        .expect("program should compile");
    let mut runner = program.runner();
    let mut context = test_context(5, None);

    assert_eq!(
        runner.next_action(&mut context),
        Some(ExecutableAction::Mine)
    );
    assert_eq!(runner.next_action(&mut context), None);
}
