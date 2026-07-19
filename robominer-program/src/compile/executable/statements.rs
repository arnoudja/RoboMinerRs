use crate::types::{
    CompileError, ExecutableExpression, ExecutableStatement, ExecutableStatementKind, Operator,
    RobotProperty, ValueType, VariableOperator,
};

use super::super::input::{CompileInput, expect_char, robot_property_mutation_error};
use super::actions::parse_executable_action_statement;
use super::expect_declared_variable;
use super::expressions::parse_executable_expression;

pub(super) fn parse_executable_sequence(
    input: &mut CompileInput,
) -> Result<ExecutableStatement, CompileError> {
    let source_line = clamp_line(input.current_line);
    expect_char(input, '{', "'{' expected")?;

    let outer_scope = input.variables.scope_depth;
    input.variables.set_scope_depth(outer_scope + 1);

    let mut statements = Vec::new();
    let mut previous_terminated = true;

    input.eat_char(';', true);

    while input.peek() != Some('}') && !input.eof() {
        if !previous_terminated {
            return Err(CompileError::new(format!(
                "Missing ';' at line {}.",
                input.current_line
            )));
        }

        if input.peek() == Some('{') {
            statements.push(parse_executable_sequence(input)?);
            input.eat_char(';', true);
            previous_terminated = true;
        } else {
            let (statement, terminated) = parse_executable_statement(input)?;
            statements.push(statement);
            previous_terminated = input.eat_char(';', true) || terminated;
        }
    }

    expect_char(input, '}', "'}' expected")?;
    input.variables.set_scope_depth(outer_scope);

    Ok(ExecutableStatement::at(
        source_line,
        ExecutableStatementKind::Sequence(statements),
    ))
}

fn clamp_line(line: usize) -> u16 {
    line.min(u16::MAX as usize) as u16
}

fn parse_executable_statement(
    input: &mut CompileInput,
) -> Result<(ExecutableStatement, bool), CompileError> {
    if let Some(statement) = parse_executable_variable_statement(input)? {
        return Ok((statement, false));
    }

    if input.use_next_word("while") {
        let source_line = clamp_line(input.current_line);
        expect_char(input, '(', "'(' expected")?;
        let condition = parse_executable_expression(input)?.ok_or_else(|| {
            CompileError::new(format!(
                "Executable program support requires a runtime expression at line {}",
                input.current_line
            ))
        })?;
        expect_char(input, ')', "')' expected")?;

        let body = if input.eat_char(';', false) {
            None
        } else {
            Some(Box::new(parse_executable_item(input)?))
        };

        return Ok((
            ExecutableStatement::at(
                source_line,
                ExecutableStatementKind::While {
                    condition,
                    body,
                    is_do_while: false,
                },
            ),
            true,
        ));
    }

    if input.use_next_word("do") {
        let source_line = clamp_line(input.current_line);
        if input.peek() != Some('{') {
            return Err(CompileError::new(format!(
                "Syntax error at line {}. '{{' expected",
                input.current_line
            )));
        }

        let body = Some(Box::new(parse_executable_sequence(input)?));

        if !input.use_next_word("while") {
            return Err(CompileError::new(format!(
                "Syntax error at line {}. 'while' expected",
                input.current_line
            )));
        }

        expect_char(input, '(', "'(' expected")?;
        let condition = parse_executable_expression(input)?.ok_or_else(|| {
            CompileError::new(format!(
                "Executable program support requires a runtime expression at line {}",
                input.current_line
            ))
        })?;
        expect_char(input, ')', "')' expected")?;
        let terminated = input.eat_char(';', false);

        return Ok((
            ExecutableStatement::at(
                source_line,
                ExecutableStatementKind::While {
                    condition,
                    body,
                    is_do_while: true,
                },
            ),
            terminated,
        ));
    }

    if input.use_next_word("if") {
        let source_line = clamp_line(input.current_line);
        expect_char(input, '(', "'(' expected")?;
        let condition = parse_executable_expression(input)?.ok_or_else(|| {
            CompileError::new(format!(
                "Executable program support requires a runtime expression at line {}",
                input.current_line
            ))
        })?;
        expect_char(input, ')', "')' expected")?;

        let true_body = Box::new(parse_executable_item(input)?);
        let mut false_body = None;

        if input.use_next_word("else") {
            false_body = Some(Box::new(parse_executable_item(input)?));
        }

        return Ok((
            ExecutableStatement::at(
                source_line,
                ExecutableStatementKind::If {
                    condition,
                    true_body,
                    false_body,
                },
            ),
            true,
        ));
    }

    Ok((parse_executable_expression_statement(input)?, false))
}

fn parse_executable_expression_statement(
    input: &mut CompileInput,
) -> Result<ExecutableStatement, CompileError> {
    let source_line = clamp_line(input.current_line);
    let upcoming = input.get_next_word();
    if matches!(
        upcoming,
        "mine" | "move" | "rotate" | "dump" | "dumpA" | "dumpB" | "dumpC"
    ) {
        return parse_executable_action_statement(input);
    }

    let expression = parse_executable_expression(input)?.ok_or_else(|| {
        CompileError::new(format!(
            "Syntax error at line {}. Statement expected",
            input.current_line
        ))
    })?;

    Ok(ExecutableStatement::at(
        source_line,
        ExecutableStatementKind::Expression(expression),
    ))
}

fn parse_executable_item(input: &mut CompileInput) -> Result<ExecutableStatement, CompileError> {
    if input.peek() == Some('{') {
        parse_executable_sequence(input)
    } else {
        let (statement, terminated) = parse_executable_statement(input)?;
        if !terminated {
            expect_char(input, ';', "';' expected")?;
        }
        Ok(statement)
    }
}

fn parse_executable_variable_statement(
    input: &mut CompileInput,
) -> Result<Option<ExecutableStatement>, CompileError> {
    let source_line = clamp_line(input.current_line);
    let is_const = input.use_next_word("const");

    let value_type = if input.use_next_word("int") {
        Some(ValueType::Int)
    } else if input.use_next_word("double") || input.use_next_word("float") {
        Some(ValueType::Double)
    } else if input.use_next_word("bool") {
        Some(ValueType::Bool)
    } else {
        None
    };

    if let Some(value_type) = value_type {
        let name = input.use_next_word_any();
        if name.is_empty() {
            return Err(CompileError::new(format!(
                "Syntax error at line {}. Identifier expected",
                input.current_line
            )));
        }

        if input.variables.exists_at_current_level(&name) {
            return Err(CompileError::new(format!(
                "Duplicate variable declaration at line {}: {}",
                input.current_line, name
            )));
        }

        let value = if input.eat_char('=', false) {
            Some(parse_executable_expression(input)?.ok_or_else(|| {
                CompileError::new(format!(
                    "Syntax error at line {}. Expression expected",
                    input.current_line
                ))
            })?)
        } else if is_const {
            return Err(CompileError::new(format!(
                "Error at line {}: const variables must be assigned a value on declaration",
                input.current_line
            )));
        } else {
            None
        };

        input.variables.declare(name.clone(), value_type, is_const);

        return Ok(Some(ExecutableStatement::at(
            source_line,
            ExecutableStatementKind::Declare { name, value },
        )));
    }

    if is_const {
        return Err(CompileError::new(format!(
            "Syntax error at line {}. Variable type expected",
            input.current_line
        )));
    }

    let mut variable_operator = VariableOperator::None;
    if input.eat_sequence("++") {
        variable_operator = VariableOperator::PreIncrement;
    } else if input.eat_sequence("--") {
        variable_operator = VariableOperator::PreDecrement;
    }

    let name = input.use_next_word_any();
    if name.is_empty() {
        if variable_operator != VariableOperator::None {
            return Err(CompileError::new(format!(
                "Syntax error at line {}. Variable expected",
                input.current_line
            )));
        }
        return Ok(None);
    }

    if variable_operator != VariableOperator::None {
        expect_declared_variable(input, &name)?;
        return Ok(Some(ExecutableStatement::at(
            source_line,
            ExecutableStatementKind::Expression(ExecutableExpression::VariableUpdate {
                name,
                operator: variable_operator,
            }),
        )));
    }

    if input.eat_sequence("+=") {
        expect_declared_variable(input, &name)?;
        if input.variables.is_const(&name) {
            return Err(CompileError::new(format!(
                "Error at line {}: The value of a const variable cannot be changed.",
                input.current_line
            )));
        }
        let rhs = parse_executable_expression(input)?.ok_or_else(|| {
            CompileError::new(format!(
                "Syntax error at line {}. Expression expected",
                input.current_line
            ))
        })?;
        return Ok(Some(ExecutableStatement::at(
            source_line,
            ExecutableStatementKind::Assign {
                name: name.clone(),
                value: ExecutableExpression::Binary {
                    operator: Operator::Addition,
                    left: Box::new(ExecutableExpression::Variable(name)),
                    right: Box::new(rhs),
                },
            },
        )));
    }

    if input.eat_sequence("-=") {
        expect_declared_variable(input, &name)?;
        if input.variables.is_const(&name) {
            return Err(CompileError::new(format!(
                "Error at line {}: The value of a const variable cannot be changed.",
                input.current_line
            )));
        }
        let rhs = parse_executable_expression(input)?.ok_or_else(|| {
            CompileError::new(format!(
                "Syntax error at line {}. Expression expected",
                input.current_line
            ))
        })?;
        return Ok(Some(ExecutableStatement::at(
            source_line,
            ExecutableStatementKind::Assign {
                name: name.clone(),
                value: ExecutableExpression::Binary {
                    operator: Operator::Subtraction,
                    left: Box::new(ExecutableExpression::Variable(name)),
                    right: Box::new(rhs),
                },
            },
        )));
    }

    if input.eat_char('=', false) {
        expect_declared_variable(input, &name)?;
        if input.variables.is_const(&name) {
            return Err(CompileError::new(format!(
                "Error at line {}: The value of a const variable cannot be changed.",
                input.current_line
            )));
        }
        let value = parse_executable_expression(input)?.ok_or_else(|| {
            CompileError::new(format!(
                "Syntax error at line {}. Expression expected",
                input.current_line
            ))
        })?;
        return Ok(Some(ExecutableStatement::at(
            source_line,
            ExecutableStatementKind::Assign { name, value },
        )));
    }

    if input.eat_sequence("++") {
        expect_declared_variable(input, &name)?;
        return Ok(Some(ExecutableStatement::at(
            source_line,
            ExecutableStatementKind::Expression(ExecutableExpression::VariableUpdate {
                name,
                operator: VariableOperator::PostIncrement,
            }),
        )));
    }

    if input.eat_sequence("--") {
        expect_declared_variable(input, &name)?;
        return Ok(Some(ExecutableStatement::at(
            source_line,
            ExecutableStatementKind::Expression(ExecutableExpression::VariableUpdate {
                name,
                operator: VariableOperator::PostDecrement,
            }),
        )));
    }

    if name == "robot" && input.peek() == Some('.') {
        input.eat_char('.', false);
        let property_name = input.use_next_word_any();
        if property_name.is_empty() {
            return Err(CompileError::new(format!(
                "Syntax error at line {}. Robot property expected",
                input.current_line
            )));
        }
        let property = RobotProperty::from_name(&property_name, input.current_line)?;
        if input.eat_char('=', false) || input.eat_sequence("++") || input.eat_sequence("--") {
            return Err(robot_property_mutation_error(input.current_line));
        }
        return Ok(Some(ExecutableStatement::at(
            source_line,
            ExecutableStatementKind::Expression(ExecutableExpression::RobotProperty(property)),
        )));
    }

    input.return_next_word(name);
    Ok(None)
}
