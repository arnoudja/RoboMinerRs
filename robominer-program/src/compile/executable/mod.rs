mod actions;
mod expressions;
mod statements;

use crate::types::{CompileError, ExecutableAction, ExecutableProgram, ExecutableStatement};

use super::input::CompileInput;

use statements::parse_executable_sequence;

pub(super) fn expect_declared_variable(input: &CompileInput, name: &str) -> Result<(), CompileError> {
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
        match statement {
            ExecutableStatement::Action(action) => actions.push(*action),
            ExecutableStatement::DynamicAction(action) => {
                if let Some(action) = action.static_action() {
                    actions.push(action);
                }
            }
            ExecutableStatement::Sequence(statements) => {
                collect_static_actions(statements, actions)
            }
            ExecutableStatement::Declare { .. }
            | ExecutableStatement::Assign { .. }
            | ExecutableStatement::Expression(_) => {}
            ExecutableStatement::If { .. } | ExecutableStatement::While { .. } => {}
        }
    }
}

pub(super) fn parse_executable_program(source: &str) -> Result<ExecutableProgram, CompileError> {
    let mut input = CompileInput::new(source);
    let root = parse_executable_sequence(&mut input)?;
    let statements = match root {
        ExecutableStatement::Sequence(statements) => statements,
        statement => vec![statement],
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
