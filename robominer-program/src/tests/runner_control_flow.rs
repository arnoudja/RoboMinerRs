use crate::*;

use super::helpers::*;

#[test]
fn while_condition_recheck_uses_while_source_line() {
    // Line 1: while (move(1) >= 1)
    // Line 2: { mine(); }
    let program = compile_executable_source("while (move(1) >= 1)\n{\nmine();\n}")
        .expect("while with move condition should compile");
    let mut runner = program.runner();

    let mut first = test_context(10, None);
    assert!(matches!(
        runner.next_action(&mut first),
        Some(ExecutableAction::Move(1.0))
    ));
    assert_eq!(runner.current_source_line(), Some(1));

    let mut after_move = test_context(10, Some(1.0));
    assert_eq!(
        runner.next_action(&mut after_move),
        Some(ExecutableAction::Mine)
    );
    assert_eq!(runner.current_source_line(), Some(3));

    // After the body finishes, the next move belongs to the while condition again.
    let mut recheck = test_context(10, Some(4.0));
    assert!(matches!(
        runner.next_action(&mut recheck),
        Some(ExecutableAction::Move(1.0))
    ));
    assert_eq!(
        runner.current_source_line(),
        Some(1),
        "while re-check must attribute to the while line, not the last body statement"
    );
}

#[test]
fn do_while_condition_recheck_uses_do_source_line() {
    // Line 1: do {
    // Line 2: mine();
    // Line 3: } while (move(1) >= 1);
    let program = compile_executable_source("do {\nmine();\n} while (move(1) >= 1);")
        .expect("do-while with move condition should compile");
    let mut runner = program.runner();

    let mut start = test_context(10, None);
    assert_eq!(runner.next_action(&mut start), Some(ExecutableAction::Mine));
    assert_eq!(runner.current_source_line(), Some(2));

    let mut after_mine = test_context(10, Some(4.0));
    assert!(matches!(
        runner.next_action(&mut after_mine),
        Some(ExecutableAction::Move(1.0))
    ));
    assert_eq!(
        runner.current_source_line(),
        Some(1),
        "do-while condition must attribute to the do line"
    );
}

#[test]
fn executable_program_suspends_for_mine_condition_result() {
    let program = compile_executable_source("while (mine());")
        .expect("mine action return should compile in executable control flow");
    let mut runner = program.runner();

    let mut first_context = test_context(3, None);
    assert_eq!(
        runner.next_action(&mut first_context),
        Some(ExecutableAction::Mine)
    );

    let mut mined_context = test_context(3, Some(4.0));
    assert_eq!(
        runner.next_action(&mut mined_context),
        Some(ExecutableAction::Mine)
    );

    let mut depleted_context = test_context(3, Some(0.0));
    assert_eq!(runner.next_action(&mut depleted_context), None);
}

#[test]
fn executable_while_block_allows_following_statement_without_semicolon() {
    let source = r#"while (!mine())
{
move(3);
}

while (true)
{
while (mine());
if (move(1) < 0.9)
{
    move(-1);
    rotate(135);
    rotate(135);
    rotate(135);
    rotate(135);
    move(1);
}
}"#;

    let program = compile_executable_source(source)
        .expect("while block followed by another while should compile");
    let mut runner = program.runner();
    let mut empty_context = test_context(20, None);

    assert_eq!(
        runner.next_action(&mut empty_context),
        Some(ExecutableAction::Mine)
    );
    let mut after_mine_context = test_context(20, Some(0.0));
    assert_eq!(
        runner.next_action(&mut after_mine_context),
        Some(ExecutableAction::Move(3.0))
    );
}

#[test]
fn executable_do_while_compiles_and_runs_body_before_condition() {
    let program = compile_executable_source("do { mine(); } while (false);")
        .expect("do-while should compile to an executable program");
    let mut runner = program.runner();

    let mut start = test_context(10, None);
    assert_eq!(runner.next_action(&mut start), Some(ExecutableAction::Mine));

    let mut after_mine = test_context(10, Some(5.0));
    assert_eq!(runner.next_action(&mut after_mine), None);
}

#[test]
fn executable_do_while_repeats_while_condition_is_true() {
    let program = compile_executable_source("int count = 0; do { count++; } while (count < 3);")
        .expect("counting do-while should compile");
    let mut runner = program.runner();

    for _ in 0..20 {
        let mut context = test_context(10, None);
        match runner.step(&mut context) {
            ProgramStep::Done => break,
            ProgramStep::Action(_) => panic!("counting do-while should not emit actions"),
            ProgramStep::Cpu => {}
        }
    }

    assert_eq!(runner.runtime_variable("count"), 3.0);
}
