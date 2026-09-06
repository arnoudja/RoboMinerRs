//! Shared statement-tree walks used by GP (and similar AST consumers).

use crate::types::{ExecutableProgram, ExecutableStatement, ExecutableStatementKind};

pub(crate) fn count_statements(statements: &[ExecutableStatement]) -> usize {
    statements.iter().map(count_statement).sum()
}

/// Count statements in main and every function body (BTreeMap name order does not matter).
pub(crate) fn count_program_statements(program: &ExecutableProgram) -> usize {
    let main = count_statements(&program.statements);
    let functions = program
        .functions
        .values()
        .map(|function| count_statements(&function.body))
        .sum::<usize>();
    main + functions
}

pub(crate) fn count_statement(statement: &ExecutableStatement) -> usize {
    1 + match &statement.kind {
        ExecutableStatementKind::Sequence(statements) => count_statements(statements),
        ExecutableStatementKind::If {
            true_body,
            false_body,
            ..
        } => {
            count_statement(true_body)
                + false_body
                    .as_ref()
                    .map(|body| count_statement(body))
                    .unwrap_or(0)
        }
        ExecutableStatementKind::While { body, .. } => {
            body.as_ref().map(|body| count_statement(body)).unwrap_or(0)
        }
        _ => 0,
    }
}

/// Pre-order index of the statement at `index`, calling `f` with a mutable reference.
pub(crate) fn with_statement_at_mut<R>(
    statements: &mut [ExecutableStatement],
    index: usize,
    f: &mut impl FnMut(&mut ExecutableStatement) -> R,
) -> Option<R> {
    let mut remaining = index;
    for statement in statements.iter_mut() {
        if remaining == 0 {
            return Some(f(statement));
        }
        remaining -= 1;
        if let Some(result) = with_nested_mut(statement, &mut remaining, f) {
            return Some(result);
        }
    }
    None
}

/// Pre-order index across main statements, then each function body in name order.
pub(crate) fn with_program_statement_at_mut<R>(
    program: &mut ExecutableProgram,
    index: usize,
    mut f: impl FnMut(&mut ExecutableStatement) -> R,
) -> Option<R> {
    let main_count = count_statements(&program.statements);
    if index < main_count {
        return with_statement_at_mut(&mut program.statements, index, &mut f);
    }
    let mut remaining = index - main_count;
    for function in program.functions.values_mut() {
        let body_count = count_statements(&function.body);
        if remaining < body_count {
            return with_statement_at_mut(&mut function.body, remaining, &mut f);
        }
        remaining -= body_count;
    }
    None
}

pub(crate) fn take_program_statement_at(
    program: &mut ExecutableProgram,
    index: usize,
) -> Option<ExecutableStatement> {
    with_program_statement_at_mut(program, index, |statement| statement.clone())
}

pub(crate) fn set_program_statement_at(
    program: &mut ExecutableProgram,
    index: usize,
    replacement: ExecutableStatement,
) -> Option<()> {
    with_program_statement_at_mut(program, index, |statement| {
        *statement = replacement.clone();
    })
}

fn with_nested_mut<R>(
    statement: &mut ExecutableStatement,
    remaining: &mut usize,
    f: &mut impl FnMut(&mut ExecutableStatement) -> R,
) -> Option<R> {
    match &mut statement.kind {
        ExecutableStatementKind::Sequence(statements) => {
            for child in statements.iter_mut() {
                if *remaining == 0 {
                    return Some(f(child));
                }
                *remaining -= 1;
                if let Some(result) = with_nested_mut(child, remaining, f) {
                    return Some(result);
                }
            }
            None
        }
        ExecutableStatementKind::If {
            true_body,
            false_body,
            ..
        } => {
            if *remaining == 0 {
                return Some(f(true_body));
            }
            *remaining -= 1;
            if let Some(result) = with_nested_mut(true_body, remaining, f) {
                return Some(result);
            }
            if let Some(false_body) = false_body {
                if *remaining == 0 {
                    return Some(f(false_body));
                }
                *remaining -= 1;
                with_nested_mut(false_body, remaining, f)
            } else {
                None
            }
        }
        ExecutableStatementKind::While { body, .. } => {
            let Some(body) = body else {
                return None;
            };
            if *remaining == 0 {
                return Some(f(body));
            }
            *remaining -= 1;
            with_nested_mut(body, remaining, f)
        }
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::count_program_statements;
    use crate::compile_executable_source;

    #[test]
    fn count_program_statements_includes_function_bodies() {
        let main_only = compile_executable_source("move(1);").expect("compile main");
        let with_function =
            compile_executable_source("fn int f() { return 1; } move(f());").expect("compile fn");
        assert_eq!(count_program_statements(&main_only), 1);
        assert!(
            count_program_statements(&with_function) > count_program_statements(&main_only),
            "function body statements must be visited"
        );
    }
}
