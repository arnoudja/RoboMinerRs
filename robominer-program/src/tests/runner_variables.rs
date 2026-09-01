use crate::*;

use super::helpers::*;
use crate::program_value::ProgramValue;

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
    let program =
        compile_executable_source("{ int value = 1; }; int value = 2; if (value == 2) { mine(); }")
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
    let program = compile_executable_source("int x = 1; mine();").expect("program should compile");
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
    let program =
        compile_executable_source("move(robot.cpuSpeed);").expect("program should compile");
    let mut runner = program.runner();
    let mut context = robot_context(72.0);

    loop {
        match runner.step(&mut context) {
            ProgramStep::Action(ExecutableAction::Move(distance)) => {
                assert!((distance - 72.0).abs() < f64::EPSILON);
                break;
            }
            ProgramStep::Cpu => {}
            ProgramStep::Done | ProgramStep::Fault => {
                panic!("program finished without issuing move")
            }
            other => panic!("unexpected step {other:?}"),
        }
    }
}

#[test]
fn area_property_expression_evaluates_from_context() {
    let program =
        compile_executable_source("move(area.sizeX + area.miningTurns + area.startingOreA);")
            .expect("program should compile");
    let mut runner = program.runner();
    let mut context = robot_context(1.0);
    context.area.size_x = 9.0;
    context.area.mining_turns = 20;
    context.area.starting_ore_a = 42;

    loop {
        match runner.step(&mut context) {
            ProgramStep::Action(ExecutableAction::Move(distance)) => {
                assert!((distance - 71.0).abs() < f64::EPSILON);
                break;
            }
            ProgramStep::Cpu => {}
            ProgramStep::Done | ProgramStep::Fault => {
                panic!("program finished without issuing move")
            }
            other => panic!("unexpected step {other:?}"),
        }
    }
}

#[test]
fn dynamic_move_in_expression_condition_compiles_and_runs() {
    assert_valid_any_size("if (move(robot.forwardSpeed) < 1) { rotate(150); } else { rotate(0); }");

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
            ProgramStep::Done | ProgramStep::Fault => break,
        }
    }

    assert_eq!(moves, vec![2.0]);
    assert_eq!(rotates, vec![0.0]);
}

#[test]
fn runtime_variables_snapshot_flattens_scopes_with_types() {
    let program = compile_executable_source(
        "int outer = 1; bool flag = true; { int outer = 2; double speed = 1.5; };",
    )
    .expect("source should compile");
    let mut runner = program.runner();

    let mut saw_inner_shadow = false;
    loop {
        let mut context = test_context(50, None);
        match runner.step(&mut context) {
            ProgramStep::Cpu => {
                let snap = runner.runtime_variables_snapshot();
                if snap.get("outer").map(|v| v.value) == Some(ProgramValue::Int(2))
                    && snap.contains_key("speed")
                    && snap.contains_key("flag")
                {
                    assert_eq!(snap["outer"].kind(), CpuStepResultKind::Int);
                    assert_eq!(snap["flag"].kind(), CpuStepResultKind::Bool);
                    assert_eq!(snap["flag"].value, ProgramValue::Bool(true));
                    assert_eq!(snap["speed"].kind(), CpuStepResultKind::Float);
                    assert_eq!(snap["speed"].value, ProgramValue::Float(1.5));
                    saw_inner_shadow = true;
                }
            }
            ProgramStep::Done | ProgramStep::Fault => break,
            ProgramStep::Action(_) => {}
        }
    }

    assert!(
        saw_inner_shadow,
        "should observe shadowed outer=2 with typed locals"
    );

    let final_snap = runner.runtime_variables_snapshot();
    assert_eq!(
        final_snap.get("outer").map(|v| v.value),
        Some(ProgramValue::Int(1))
    );
    assert_eq!(final_snap["outer"].kind(), CpuStepResultKind::Int);
    assert!(!final_snap.contains_key("speed"));
    assert!(final_snap.contains_key("flag"));
    assert_eq!(final_snap["flag"].kind(), CpuStepResultKind::Bool);
}

#[test]
fn int_assign_from_float_rounds_half_away_from_zero() {
    let program = compile_executable_source("int v = 3.75; if (v == 4) { mine(); }")
        .expect("program should compile");
    let mut runner = program.runner();
    let mut context = test_context(5, None);

    assert_eq!(
        runner.next_action(&mut context),
        Some(ExecutableAction::Mine)
    );

    let snap = runner.runtime_variables_snapshot();
    assert_eq!(snap["v"].value, ProgramValue::Int(4));
}

#[test]
fn int_division_truncates_toward_zero() {
    let program =
        compile_executable_source("int a = 7; int b = 2; int c = a / b; if (c == 3) { mine(); }")
            .expect("program should compile");
    let mut runner = program.runner();
    let mut context = test_context(5, None);

    assert_eq!(
        runner.next_action(&mut context),
        Some(ExecutableAction::Mine)
    );
}

#[test]
fn mixed_int_float_compare_promotes_int_to_float() {
    let program = compile_executable_source(
        "int i = 4; double d = 3.75; if (d > i) { mine(); } else { rotate(90); }",
    )
    .expect("program should compile");
    let mut runner = program.runner();
    let mut context = test_context(5, None);

    assert_eq!(
        runner.next_action(&mut context),
        Some(ExecutableAction::Rotate(90.0))
    );
}

#[test]
fn int_int_compare_uses_integer_semantics() {
    let program = compile_executable_source(
        "int dist = 4; int travel = 3; if (travel < dist) { mine(); } else { rotate(90); }",
    )
    .expect("program should compile");
    let mut runner = program.runner();
    let mut context = test_context(5, None);

    assert_eq!(
        runner.next_action(&mut context),
        Some(ExecutableAction::Mine)
    );

    let program = compile_executable_source(
        "int dist = 4; int travel = 4; if (travel < dist) { mine(); } else { rotate(90); }",
    )
    .expect("program should compile");
    let mut runner = program.runner();

    assert_eq!(
        runner.next_action(&mut context),
        Some(ExecutableAction::Rotate(90.0))
    );
}
