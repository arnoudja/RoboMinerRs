use crate::types::{
    CompileError, ExecutableAction, ExecutableExpression, Operator, RobotProperty, VariableOperator,
};

use super::super::input::{
    CompileInput, expect_char, expect_empty_call, parse_operator_token,
    robot_property_mutation_error,
};
use super::actions::{
    parse_dump_expression, parse_move_expression, parse_rotate_expression, parse_scan_call,
};
use super::expect_declared_variable;

pub(super) fn parse_executable_expression(
    input: &mut CompileInput,
) -> Result<Option<ExecutableExpression>, CompileError> {
    let Some(first) = parse_executable_single_expression(input)? else {
        return Ok(None);
    };

    let mut values = vec![(Operator::Undefined, first)];

    loop {
        let operator = parse_operator_token(input);
        if operator == Operator::Undefined {
            break;
        }

        let next = parse_executable_single_expression(input)?.ok_or_else(|| {
            CompileError::new(format!(
                "Syntax error at line {}. Expression expected",
                input.current_line
            ))
        })?;
        values.push((operator, next));
    }

    while values.len() > 1 {
        let mut i = 1;
        while i + 1 < values.len() && values[i + 1].0.priority() > values[i].0.priority() {
            i += 1;
        }

        let combined = ExecutableExpression::Binary {
            operator: values[i].0,
            left: Box::new(values[i - 1].1.clone()),
            right: Box::new(values[i].1.clone()),
        };
        values[i - 1].1 = combined;
        values.remove(i);
    }

    Ok(Some(values.remove(0).1))
}

pub(super) fn parse_executable_single_expression(
    input: &mut CompileInput,
) -> Result<Option<ExecutableExpression>, CompileError> {
    if input.eat_char('(', false) {
        let value = parse_executable_expression(input)?;
        if value.is_none() || !input.eat_char(')', false) {
            return Err(CompileError::new(format!(
                "Syntax error at line {}. {} expected",
                input.current_line,
                if value.is_some() { ")" } else { "expression" }
            )));
        }

        return Ok(value);
    }

    if input.peek() != Some('=') && input.eat_char('!', false) {
        let value = parse_executable_single_expression(input)?.ok_or_else(|| {
            CompileError::new(format!(
                "Syntax error at line {}. expression expected",
                input.current_line
            ))
        })?;

        return Ok(Some(ExecutableExpression::UnaryNot(Box::new(value))));
    }

    if input.use_next_word("true") {
        return Ok(Some(ExecutableExpression::Number(1.0)));
    }

    if input.use_next_word("false") {
        return Ok(Some(ExecutableExpression::Number(0.0)));
    }

    if input.use_next_word("mine") {
        expect_empty_call(input)?;
        return Ok(Some(ExecutableExpression::Action(ExecutableAction::Mine)));
    }

    if input.use_next_word("move") {
        return Ok(Some(parse_move_expression(input)?));
    }

    if input.use_next_word("rotate") {
        return Ok(Some(parse_rotate_expression(input)?));
    }

    if input.use_next_word("dump") {
        return Ok(Some(parse_dump_expression(input)?));
    }

    if input.use_next_word("time") {
        expect_empty_call(input)?;
        return Ok(Some(ExecutableExpression::Time));
    }

    if input.use_next_word("scan") {
        return Ok(Some(parse_scan_call(input)?));
    }

    if input.use_next_word("oreDistance") {
        expect_empty_call(input)?;
        return Ok(Some(ExecutableExpression::OreDistance));
    }

    if input.use_next_word("oreType") {
        expect_empty_call(input)?;
        return Ok(Some(ExecutableExpression::OreType));
    }

    // Deprecated: prefer robot.oreStored / robot.oreStoredA|B|C. Kept for existing programs.
    if input.use_next_word("ore") {
        expect_char(input, '(', "'(' expected")?;
        let ore_type = parse_executable_expression(input)?.ok_or_else(|| {
            CompileError::new(format!(
                "Syntax error at line {}. value expected",
                input.current_line
            ))
        })?;
        expect_char(input, ')', "')' expected")?;
        return Ok(Some(ExecutableExpression::Ore(Box::new(ore_type))));
    }

    if let Some(expression) = parse_robot_property_expression(input)? {
        if input.eat_sequence("++") || input.eat_sequence("--") {
            return Err(robot_property_mutation_error(input.current_line));
        }
        return Ok(Some(expression));
    }

    let mut variable_operator = VariableOperator::None;
    if input.eat_sequence("++") {
        variable_operator = VariableOperator::PreIncrement;
    } else if input.eat_sequence("--") {
        variable_operator = VariableOperator::PreDecrement;
    }

    let name = input.use_next_word_any();
    if !name.is_empty() {
        if variable_operator == VariableOperator::None {
            if input.eat_sequence("++") {
                variable_operator = VariableOperator::PostIncrement;
            } else if input.eat_sequence("--") {
                variable_operator = VariableOperator::PostDecrement;
            }
        }

        expect_declared_variable(input, &name)?;

        return Ok(Some(if variable_operator == VariableOperator::None {
            ExecutableExpression::Variable(name)
        } else {
            ExecutableExpression::VariableUpdate {
                name,
                operator: variable_operator,
            }
        }));
    } else if variable_operator != VariableOperator::None {
        return Err(CompileError::new(format!(
            "Syntax error at line {}. Variable expected",
            input.current_line
        )));
    }

    Ok(input
        .extract_number_value()
        .map(ExecutableExpression::Number))
}

fn parse_robot_property_expression(
    input: &mut CompileInput,
) -> Result<Option<ExecutableExpression>, CompileError> {
    if !input.use_next_word("robot") {
        return Ok(None);
    }

    if !input.eat_char('.', false) {
        input.return_next_word("robot".to_string());
        return Ok(None);
    }

    let property_name = input.use_next_word_any();
    if property_name.is_empty() {
        return Err(CompileError::new(format!(
            "Syntax error at line {}. Robot property expected",
            input.current_line
        )));
    }

    let property = RobotProperty::from_name(&property_name, input.current_line)?;
    Ok(Some(ExecutableExpression::RobotProperty(property)))
}
