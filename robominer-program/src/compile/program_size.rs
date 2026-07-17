use crate::types::{
    ExecutableAction, ExecutableActionExpression, ExecutableExpression, ExecutableProgram,
    ExecutableStatement, ExecutableStatementKind, VariableOperator,
};

pub(super) fn program_instruction_size(program: &ExecutableProgram) -> usize {
    sequence_size(&program.statements)
}

fn sequence_size(statements: &[ExecutableStatement]) -> usize {
    match statements.len() {
        0 => 0,
        1 => statement_size(&statements[0]),
        _ => 1 + statements.iter().map(statement_size).sum::<usize>(),
    }
}

fn statement_size(statement: &ExecutableStatement) -> usize {
    match &statement.kind {
        ExecutableStatementKind::Action(action) => action_expression_size(action),
        ExecutableStatementKind::DynamicAction(action) => dynamic_action_size(action),
        ExecutableStatementKind::Sequence(statements) => sequence_size(statements),
        ExecutableStatementKind::Declare { value, .. } => {
            1 + value.as_ref().map(expression_size).unwrap_or(0)
        }
        ExecutableStatementKind::Assign { value, .. } => 1 + expression_size(value),
        ExecutableStatementKind::Expression(expression) => expression_size(expression),
        ExecutableStatementKind::If {
            condition,
            true_body,
            false_body,
        } => {
            1 + expression_size(condition)
                + statement_size(true_body)
                + false_body
                    .as_ref()
                    .map(|body| statement_size(body))
                    .unwrap_or(0)
        }
        ExecutableStatementKind::While {
            condition, body, ..
        } => {
            1 + expression_size(condition)
                + body.as_ref().map(|body| statement_size(body)).unwrap_or(0)
        }
    }
}

fn dynamic_action_size(action: &ExecutableActionExpression) -> usize {
    match action {
        ExecutableActionExpression::Move(expression)
        | ExecutableActionExpression::Rotate(expression)
        | ExecutableActionExpression::Dump(expression) => 1 + expression_size(expression),
    }
}

fn expression_size(expression: &ExecutableExpression) -> usize {
    match expression {
        ExecutableExpression::Number(_) => 1,
        ExecutableExpression::Variable(_) => 1,
        ExecutableExpression::VariableUpdate { operator, .. } => {
            if *operator == VariableOperator::None {
                1
            } else {
                2
            }
        }
        ExecutableExpression::UnaryNot(expression) => 1 + expression_size(expression),
        ExecutableExpression::Binary { left, right, .. } => {
            1 + expression_size(left) + expression_size(right)
        }
        ExecutableExpression::Time
        | ExecutableExpression::OreDistance
        | ExecutableExpression::OreType => 1,
        ExecutableExpression::Ore(expression) => 1 + expression_size(expression),
        ExecutableExpression::Scan(direction) => {
            1 + direction
                .as_ref()
                .map(|expression| expression_size(expression))
                .unwrap_or(0)
        }
        ExecutableExpression::RobotProperty(_) => 1,
        ExecutableExpression::Move(expression)
        | ExecutableExpression::Rotate(expression)
        | ExecutableExpression::Dump(expression) => 1 + expression_size(expression),
        ExecutableExpression::Action(action) => action_expression_size(action),
    }
}

fn action_expression_size(action: &ExecutableAction) -> usize {
    match action {
        ExecutableAction::Mine
        | ExecutableAction::StartScan(_)
        | ExecutableAction::AwaitScanResult => 1,
        ExecutableAction::Move(_) | ExecutableAction::Rotate(_) | ExecutableAction::Dump(_) => 2,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::compile::fixtures::compatibility_fixtures_with_expected_size;
    use crate::compile_executable_source;

    #[test]
    fn ast_size_matches_known_program_sizes() {
        for fixture in compatibility_fixtures_with_expected_size() {
            let program = compile_executable_source(fixture.source)
                .unwrap_or_else(|err| panic!("{} should compile: {err}", fixture.name));
            assert_eq!(
                program_instruction_size(&program),
                fixture.expected_size.unwrap() as usize,
                "{}",
                fixture.name
            );
        }
    }
}
