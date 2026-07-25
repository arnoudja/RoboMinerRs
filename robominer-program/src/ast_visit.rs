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
