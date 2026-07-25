use crate::*;

use super::helpers::*;

fn span(line: u16, start_col: u16, end_col: u16) -> SourceSpan {
    SourceSpan {
        line,
        start_col,
        end_col,
    }
}

/// Walk a program one CPU/action step at a time, recording each new source span.
fn spans_while_stepping(source: &str) -> Vec<SourceSpan> {
    let program = compile_executable_source(source).expect("program should compile");
    let mut runner = program.runner();
    let mut spans: Vec<SourceSpan> = Vec::new();
    let mut action_result = None;

    for _ in 0..64 {
        if let Some(span) = runner.current_source_span()
            && spans.last() != Some(&span)
        {
            spans.push(span);
        }

        let mut context = test_context(20, action_result);
        match runner.step(&mut context) {
            ProgramStep::Done => break,
            ProgramStep::Action(_) => action_result = Some(1.0),
            ProgramStep::Cpu => action_result = None,
        }
    }

    spans
}

#[test]
fn statements_sharing_a_line_get_distinct_column_spans() {
    let program =
        compile_executable_source("mine(); dumpA();").expect("two actions should compile");
    let statements = program.statements();

    assert_eq!(statements.len(), 2);
    // Columns are 1-based over the displayed source, start inclusive and end exclusive,
    // so `mine()` is 1..7 and `dumpA()` is 9..16.
    assert_eq!(statements[0].source_span, span(1, 1, 7));
    assert_eq!(statements[1].source_span, span(1, 9, 16));
    assert_eq!(statements[0].source_line(), 1);
    assert_eq!(statements[1].source_line(), 1);
}

#[test]
fn statement_spans_track_columns_past_the_first_line() {
    let program = compile_executable_source("mine();\n  dumpB(); dumpC();")
        .expect("multi-line actions should compile");
    let statements = program.statements();

    assert_eq!(statements.len(), 3);
    assert_eq!(statements[0].source_span, span(1, 1, 7));
    // Line 2 is not offset by the compiler's wrapping `{`, so the indent is preserved.
    assert_eq!(statements[1].source_span, span(2, 3, 10));
    assert_eq!(statements[2].source_span, span(2, 12, 19));
}

#[test]
fn source_span_line_matches_statement_source_line() {
    let program = compile_executable_source("while (mine())\n{\nmove(1);\n}")
        .expect("while loop should compile");

    for statement in program.statements() {
        assert_eq!(statement.source_span.line, statement.source_line());
    }
}

#[test]
fn expression_evaluation_advances_source_span_within_one_line() {
    // Columns: `if (mine() > 0) { dump(); }`
    //           1234567890...
    let spans = spans_while_stepping("if (mine() > 0) { dump(); }");

    assert!(
        spans.iter().all(|span| span.line == 1),
        "single-line program should only report line 1: {spans:?}"
    );
    assert!(
        spans.len() >= 3,
        "expression CPU steps should report several spans: {spans:?}"
    );

    // `mine()` and the literal `0` are evaluated in separate CPU cycles, so replay
    // highlighting can distinguish them.
    assert!(
        spans.contains(&span(1, 5, 11)),
        "expected a span for `mine()`: {spans:?}"
    );
    assert!(
        spans.contains(&span(1, 14, 15)),
        "expected a span for the literal `0`: {spans:?}"
    );
    // The comparison itself covers both operands.
    assert!(
        spans.contains(&span(1, 5, 15)),
        "expected a span for `mine() > 0`: {spans:?}"
    );
    // The taken branch narrows to the body statement.
    assert!(
        spans.contains(&span(1, 19, 25)),
        "expected a span for `dump();`: {spans:?}"
    );
}

#[test]
fn current_source_span_line_agrees_with_current_source_line() {
    let program =
        compile_executable_source("mine();\nmove(1);\ndumpA();").expect("program should compile");
    let mut runner = program.runner();
    let mut action_result = None;

    for _ in 0..32 {
        match (runner.current_source_span(), runner.current_source_line()) {
            (Some(span), Some(line)) => assert_eq!(span.line, line),
            (None, None) => {}
            (span, line) => panic!("span/line disagree: {span:?} vs {line:?}"),
        }

        let mut context = test_context(20, action_result);
        match runner.step(&mut context) {
            ProgramStep::Done => break,
            ProgramStep::Action(_) => action_result = Some(1.0),
            ProgramStep::Cpu => action_result = None,
        }
    }
}
