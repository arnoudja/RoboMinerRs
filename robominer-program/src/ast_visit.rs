//! Shared statement-tree walks used by GP (and similar AST consumers).

use crate::types::{ExecutableStatement, ExecutableStatementKind};

pub(crate) fn count_statements(statements: &[ExecutableStatement]) -> usize {
    statements.iter().map(count_statement).sum()
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
    mut f: impl FnMut(&mut ExecutableStatement) -> R,
) -> Option<R> {
    let mut remaining = index;
    for statement in statements.iter_mut() {
        if remaining == 0 {
            return Some(f(statement));
        }
        remaining -= 1;
        if let Some(result) = with_nested_mut(statement, &mut remaining, &mut f) {
            return Some(result);
        }
    }
    None
}

pub(crate) fn take_statement_at(
    statements: &mut [ExecutableStatement],
    index: usize,
) -> Option<ExecutableStatement> {
    with_statement_at_mut(statements, index, |statement| statement.clone())
}

pub(crate) fn set_statement_at(
    statements: &mut [ExecutableStatement],
    index: usize,
    replacement: ExecutableStatement,
) -> Option<()> {
    let mut remaining = index;
    for statement in statements.iter_mut() {
        if remaining == 0 {
            *statement = replacement;
            return Some(());
        }
        remaining -= 1;
        if set_nested(statement, &mut remaining, replacement.clone()).is_some() {
            return Some(());
        }
    }
    None
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

fn set_nested(
    statement: &mut ExecutableStatement,
    remaining: &mut usize,
    replacement: ExecutableStatement,
) -> Option<()> {
    match &mut statement.kind {
        ExecutableStatementKind::Sequence(statements) => {
            for child in statements.iter_mut() {
                if *remaining == 0 {
                    *child = replacement;
                    return Some(());
                }
                *remaining -= 1;
                if set_nested(child, remaining, replacement.clone()).is_some() {
                    return Some(());
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
                **true_body = replacement;
                return Some(());
            }
            *remaining -= 1;
            if set_nested(true_body, remaining, replacement.clone()).is_some() {
                return Some(());
            }
            if let Some(false_body) = false_body {
                if *remaining == 0 {
                    **false_body = replacement;
                    return Some(());
                }
                *remaining -= 1;
                set_nested(false_body, remaining, replacement)
            } else {
                None
            }
        }
        ExecutableStatementKind::While { body, .. } => {
            let Some(body) = body else {
                return None;
            };
            if *remaining == 0 {
                **body = replacement;
                return Some(());
            }
            *remaining -= 1;
            set_nested(body, remaining, replacement)
        }
        _ => None,
    }
}
