use crate::*;

use super::helpers::*;

#[test]
fn executable_variables_drive_control_flow() {
    let program = compile_executable_source(
        "int count = 0; while (count < 3) { count++; }; if (count == 3) { mine(); }",
    )
    .expect("source should compile with executable variables");
    let mut runner = program.runner();
    let mut context = test_context(5, None);

    assert_eq!(
        runner.next_action(&mut context),
        Some(ExecutableAction::Mine)
    );
}

#[test]
fn executable_variables_can_be_action_arguments() {
    let program = compile_executable_source("int rot = 90; rotate(rot);")
        .expect("source should compile with variable action arguments");
    let mut runner = program.runner();
    let mut context = test_context(1, None);

    assert_eq!(
        runner.next_action(&mut context),
        Some(ExecutableAction::Rotate(90.0))
    );
}

#[test]
fn executable_variables_are_scoped_to_blocks() {
    let program = compile_executable_source(
        "{ int value = 1; }; int value = 2; if (value == 2) { mine(); }",
    )
    .expect("source should compile with reusable block-scoped variables");
    let mut runner = program.runner();
    let mut context = test_context(1, None);

    assert_eq!(
        runner.next_action(&mut context),
        Some(ExecutableAction::Mine)
    );
}

#[test]
fn binary_comparison_costs_one_cpu_instruction_per_expression_node() {
    let program =
        compile_executable_source("if (1 < 2) { mine(); }").expect("program should compile");
    let mut runner = program.runner();
    let mut cpu_count = 0;

    loop {
        let mut context = test_context(50, None);
        match runner.step(&mut context) {
            ProgramStep::Cpu => cpu_count += 1,
            ProgramStep::Action(ExecutableAction::Mine) => break,
            other => panic!("unexpected step {other:?} after {cpu_count} cpu instructions"),
        }
    }

    assert_eq!(
        cpu_count, 3,
        "1 < 2 should cost three expression instructions before mine()"
    );
}

#[test]
fn literal_initialization_costs_one_cpu_instruction() {
    let program =
        compile_executable_source("int x = 1; mine();").expect("program should compile");
    let mut runner = program.runner();
    let mut cpu_count = 0;

    loop {
        let mut context = test_context(50, None);
        match runner.step(&mut context) {
            ProgramStep::Cpu => cpu_count += 1,
            ProgramStep::Action(ExecutableAction::Mine) => break,
            other => panic!("unexpected step {other:?} after {cpu_count} cpu instructions"),
        }
    }

    assert_eq!(
        cpu_count, 1,
        "int x = 1 should cost one expression instruction before mine()"
    );
}

#[test]
fn robot_property_expression_evaluates_from_context() {
    let program = compile_executable_source("move(robot.cpuSpeed);")
        .expect("program should compile");
    let mut runner = program.runner();
    let mut context = robot_context(72.0);

    loop {
        match runner.step(&mut context) {
            ProgramStep::Action(ExecutableAction::Move(distance)) => {
                assert!((distance - 72.0).abs() < f64::EPSILON);
                break;
            }
            ProgramStep::Cpu => {}
            ProgramStep::Done => panic!("program finished without issuing move"),
            other => panic!("unexpected step {other:?}"),
        }
    }
}

#[test]
fn dynamic_move_in_expression_condition_compiles_and_runs() {
    assert_valid_any_size(
        "if (move(robot.forwardSpeed) < 1) { rotate(150); } else { rotate(0); }",
    );

    let program = compile_executable_source(
        "if (move(robot.forwardSpeed) < 1) { rotate(150); } else { rotate(0); }",
    )
    .expect("program should compile");
    let mut runner = program.runner();
    let mut context = robot_context(72.0);
    context.robot.forward_speed = 2.0;

    let mut moves = Vec::new();
    let mut rotates = Vec::new();
    loop {
        match runner.step(&mut context) {
            ProgramStep::Action(ExecutableAction::Move(distance)) => {
                moves.push(distance);
                context.action_result = Some(distance);
            }
            ProgramStep::Action(ExecutableAction::Rotate(angle)) => {
                rotates.push(angle);
                break;
            }
            ProgramStep::Action(_) => {}
            ProgramStep::Cpu => {}
            ProgramStep::Done => break,
        }
    }

    assert_eq!(moves, vec![2.0]);
    assert_eq!(rotates, vec![0.0]);
}
