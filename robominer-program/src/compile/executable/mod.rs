mod actions;
mod expressions;
mod statements;

use crate::types::{
    CompileError, ExecutableAction, ExecutableProgram, ExecutableStatement, ExecutableStatementKind,
};

use super::input::CompileInput;

use statements::parse_executable_sequence;

pub(super) fn expect_declared_variable(
    input: &CompileInput,
    name: &str,
) -> Result<(), CompileError> {
    if input.variables.contains(name) {
        Ok(())
    } else {
        Err(CompileError::new(format!(
            "Syntax error at line {}. Variable expected",
            input.current_line
        )))
    }
}

fn collect_static_actions(statements: &[ExecutableStatement], actions: &mut Vec<ExecutableAction>) {
    for statement in statements {
        match &statement.kind {
            ExecutableStatementKind::Action(action) => actions.push(*action),
            ExecutableStatementKind::DynamicAction(action) => {
                if let Some(action) = action.static_action() {
                    actions.push(action);
                }
            }
            ExecutableStatementKind::Sequence(statements) => {
                collect_static_actions(statements, actions)
            }
            ExecutableStatementKind::Declare { .. }
            | ExecutableStatementKind::Assign { .. }
            | ExecutableStatementKind::Expression(_) => {}
            ExecutableStatementKind::If { .. } | ExecutableStatementKind::While { .. } => {}
        }
    }
}

pub(super) fn parse_executable_program(source: &str) -> Result<ExecutableProgram, CompileError> {
    let mut input = CompileInput::new(source);
    let root = parse_executable_sequence(&mut input)?;
    let statements = match root.kind {
        ExecutableStatementKind::Sequence(statements) => statements,
        _ => vec![root],
    };
    let mut actions = Vec::new();
    collect_static_actions(&statements, &mut actions);
    let requires_runtime = statements.iter().any(ExecutableStatement::requires_runtime);

    Ok(ExecutableProgram {
        statements,
        actions,
        requires_runtime,
    })
}
