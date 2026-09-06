use super::helpers::*;
use crate::*;

#[test]
fn function_return_value_used_by_move() {
    let program =
        compile_executable_source("fn int f() { return 2; } move(f());").expect("compile");
    let mut runner = program.runner();
    let mut ctx = test_context(20, None);
    assert!(matches!(
        runner.next_action(&mut ctx),
        Some(ExecutableAction::Move(d)) if (d - 2.0).abs() < 1e-9
    ));
}

#[test]
fn untyped_param_is_by_value_and_dynamic() {
    let program = compile_executable_source("fn int id(x) { return x; } move(id(3)); move(id(4));")
        .expect("compile");
    let mut runner = program.runner();
    let mut ctx = test_context(30, None);
    assert!(
        matches!(runner.next_action(&mut ctx), Some(ExecutableAction::Move(d)) if (d-3.0).abs()<1e-9)
    );
    let mut ctx = test_context(30, Some(3.0));
    assert!(
        matches!(runner.next_action(&mut ctx), Some(ExecutableAction::Move(d)) if (d-4.0).abs()<1e-9)
    );
}

#[test]
fn explicit_double_return_coerces_untyped_param() {
    let program =
        compile_executable_source("fn double id(x) { return x; } move(id(3.7));").expect("compile");
    let mut runner = program.runner();
    let mut ctx = test_context(20, None);
    assert!(matches!(
        runner.next_action(&mut ctx),
        Some(ExecutableAction::Move(d)) if (d - 3.7).abs() < 1e-9
    ));
}

#[test]
fn function_reads_and_writes_top_level_var() {
    let program = compile_executable_source("fn bump() { x = x + 1; } int x = 0; bump(); move(x);")
        .expect("compile");
    let mut runner = program.runner();
    let mut ctx = test_context(40, None);
    // Drain CPU until Move
    let action = runner.next_action(&mut ctx);
    assert!(matches!(action, Some(ExecutableAction::Move(d)) if (d-1.0).abs()<1e-9));
}

#[test]
fn recursion_and_depth_fault() {
    let ok = compile_executable_source(
        "fn int sum(int n) { if (n <= 0) { return 0; } return n + sum(n - 1); } move(sum(3));",
    )
    .expect("compile");
    let mut runner = ok.runner();
    let mut ctx = test_context(200, None);
    assert!(
        matches!(runner.next_action(&mut ctx), Some(ExecutableAction::Move(d)) if (d-6.0).abs()<1e-9)
    );

    let deep = compile_executable_source("fn int rec(int n) { return rec(n); } move(rec(1));")
        .expect("compile");
    let mut runner = deep.runner();
    let mut ctx = test_context(10_000, None);
    let mut saw_fault = false;
    for _ in 0..10_000 {
        match runner.step(&mut ctx) {
            ProgramStep::Fault => {
                saw_fault = true;
                break;
            }
            ProgramStep::Done => break,
            ProgramStep::Action(_) => {
                // should not need actions
            }
            ProgramStep::Cpu => {}
        }
    }
    assert!(saw_fault, "infinite recursion must Fault at depth 256");
}

#[test]
fn fallthrough_returns_zero() {
    let program = compile_executable_source("fn int f() { } move(f());").expect("compile");
    let mut runner = program.runner();
    let mut ctx = test_context(20, None);
    assert!(
        matches!(runner.next_action(&mut ctx), Some(ExecutableAction::Move(d)) if d.abs()<1e-9)
    );
}
