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
fn expression_depot_properties_read_capacity_and_stored() {
    let program = compile_executable_source(
        "dump(robot.depotSizeA); dump(robot.depotSizeB); dump(robot.depotSizeC); dump(robot.depotStoredA); dump(robot.depotStoredB); dump(robot.depotStoredC);",
    )
    .expect("program should compile");
    let mut runner = program.runner();
    let mut context = test_context(8, None);
    context.depot_capacity[0] = 10;
    context.depot_capacity[1] = 4;
    context.depot_capacity[2] = 2;
    context.depot[0] = 7;
    context.depot[1] = 1;
    context.depot[2] = 0;

    assert_eq!(
        runner.next_action(&mut context),
        Some(ExecutableAction::Dump(10))
    );
    context.action_result = Some(10.0);
    assert_eq!(
        runner.next_action(&mut context),
        Some(ExecutableAction::Dump(4))
    );
    context.action_result = Some(4.0);
    assert_eq!(
        runner.next_action(&mut context),
        Some(ExecutableAction::Dump(2))
    );
    context.action_result = Some(2.0);
    assert_eq!(
        runner.next_action(&mut context),
        Some(ExecutableAction::Dump(7))
    );
    context.action_result = Some(7.0);
    assert_eq!(
        runner.next_action(&mut context),
        Some(ExecutableAction::Dump(1))
    );
    context.action_result = Some(1.0);
    assert_eq!(
        runner.next_action(&mut context),
        Some(ExecutableAction::Dump(0))
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

#[test]
fn expression_dynamic_dump_awaits_sim_result_then_continues() {
    let program = compile_executable_source("int slot = 1; if (dump(slot) >= 0) { mine(); }")
        .expect("program should compile");
    let mut runner = program.runner();
    let mut context = test_context(10, None);

    assert_eq!(
        runner.next_action(&mut context),
        Some(ExecutableAction::Dump(1))
    );

    // Pending dump re-emits until the sim supplies an action_result.
    assert_eq!(
        runner.next_action(&mut context),
        Some(ExecutableAction::Dump(1))
    );

    let mut after_dump = test_context(10, Some(2.0));
    assert_eq!(
        runner.next_action(&mut after_dump),
        Some(ExecutableAction::Mine)
    );
}

#[test]
fn expression_unary_minus_negates_variable() {
    let program =
        compile_executable_source("int x = 5; dump(-x);").expect("program should compile");
    let mut runner = program.runner();
    let mut context = test_context(5, None);

    assert_eq!(
        runner.next_action(&mut context),
        Some(ExecutableAction::Dump(-5))
    );
}

#[test]
fn expression_unary_minus_negates_parenthesized_sum() {
    let program = compile_executable_source("dump(-(1+2));").expect("program should compile");
    let mut runner = program.runner();
    let mut context = test_context(5, None);

    assert_eq!(
        runner.next_action(&mut context),
        Some(ExecutableAction::Dump(-3))
    );
}

#[test]
fn expression_binary_minus_then_unary_minus() {
    let program = compile_executable_source("int a = 10; int b = 3; dump(a - -b);")
        .expect("program should compile");
    let mut runner = program.runner();
    let mut context = test_context(5, None);

    assert_eq!(
        runner.next_action(&mut context),
        Some(ExecutableAction::Dump(13))
    );
}

#[test]
fn expression_pre_decrement_is_not_double_unary_minus() {
    let program =
        compile_executable_source("int x = 5; dump(--x);").expect("program should compile");
    let mut runner = program.runner();
    let mut context = test_context(5, None);

    assert_eq!(
        runner.next_action(&mut context),
        Some(ExecutableAction::Dump(4))
    );
}

#[test]
fn expression_negative_number_literal_is_not_unary_minus() {
    let program =
        compile_executable_source("int x = -45; rotate(-45);").expect("program should compile");

    match &program.statements()[0].kind {
        ExecutableStatementKind::Declare {
            value: Some(expr), ..
        } => {
            assert_eq!(expr.kind, ExecutableExpressionKind::Number(-45.0));
        }
        other => panic!("expected declare with Number(-45), got {other:?}"),
    }

    assert_eq!(program.actions(), &[ExecutableAction::Rotate(-45.0)]);
}

#[test]
fn expression_abs_returns_absolute_value() {
    let program = compile_executable_source("dump(abs(-7));").expect("program should compile");
    let mut runner = program.runner();
    let mut context = test_context(5, None);

    assert_eq!(
        runner.next_action(&mut context),
        Some(ExecutableAction::Dump(7))
    );
}

#[test]
fn expression_min_returns_smaller_value() {
    let program = compile_executable_source("dump(min(3, -1));").expect("program should compile");
    let mut runner = program.runner();
    let mut context = test_context(5, None);

    assert_eq!(
        runner.next_action(&mut context),
        Some(ExecutableAction::Dump(-1))
    );
}

#[test]
fn expression_max_returns_larger_value() {
    let program = compile_executable_source("int a = 4; int b = 9; dump(max(a, b));")
        .expect("program should compile");
    let mut runner = program.runner();
    let mut context = test_context(5, None);

    assert_eq!(
        runner.next_action(&mut context),
        Some(ExecutableAction::Dump(9))
    );
}
