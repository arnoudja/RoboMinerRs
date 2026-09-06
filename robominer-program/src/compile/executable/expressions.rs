use crate::types::{
    CompileError, ExecutableAction, ExecutableExpression, ExecutableExpressionKind, Operator,
    VariableOperator,
};

use super::super::input::{CompileInput, expect_char, expect_empty_call, parse_operator_token};
use super::actions::{
    parse_dump_expression, parse_move_expression, parse_named_dump_action, parse_rotate_expression,
    parse_scan_call,
};
use super::builtins::{parse_builtin_property_expression, reject_builtin_property_mutation};
use super::expect_declared_variable;
use super::functions::parse_call_arguments;

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

        let left = values[i - 1].1.clone();
        let right = values[i].1.clone();
        let span = left.span.join(right.span);
        values[i - 1].1 = ExecutableExpression::new(
            span,
            ExecutableExpressionKind::Binary {
                operator: values[i].0,
                left: Box::new(left),
                right: Box::new(right),
            },
        );
        values.remove(i);
    }

    Ok(Some(values.remove(0).1))
}

pub(super) fn parse_executable_single_expression(
    input: &mut CompileInput,
) -> Result<Option<ExecutableExpression>, CompileError> {
    let mark = input.mark_pos();

    // A parenthesised group keeps the span of the inner expression, so highlighting
    // points at the operands rather than the punctuation.
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

    let Some(kind) = parse_single_expression_kind(input)? else {
        return Ok(None);
    };

    Ok(Some(ExecutableExpression::new(input.span_from(mark), kind)))
}

fn parse_single_expression_kind(
    input: &mut CompileInput,
) -> Result<Option<ExecutableExpressionKind>, CompileError> {
    if input.peek() != Some('=') && input.eat_char('!', false) {
        let value = parse_executable_single_expression(input)?.ok_or_else(|| {
            CompileError::new(format!(
                "Syntax error at line {}. expression expected",
                input.current_line
            ))
        })?;

        return Ok(Some(ExecutableExpressionKind::UnaryNot(Box::new(value))));
    }

    // Unary minus: not `--` (pre-decrement) and not a signed number literal like `-45`.
    if input.peek() == Some('-') && input.peek_nth(1) != Some('-') {
        let next = input.peek_nth(1);
        let is_number_literal = next.is_some_and(|c| c.is_ascii_digit() || c == '.');
        if !is_number_literal {
            input.eat_char('-', false);
            let value = parse_executable_single_expression(input)?.ok_or_else(|| {
                CompileError::new(format!(
                    "Syntax error at line {}. expression expected",
                    input.current_line
                ))
            })?;
            return Ok(Some(ExecutableExpressionKind::UnaryMinus(Box::new(value))));
        }
    }

    if input.use_next_word("true") {
        return Ok(Some(ExecutableExpressionKind::Bool(true)));
    }

    if input.use_next_word("false") {
        return Ok(Some(ExecutableExpressionKind::Bool(false)));
    }

    if input.use_next_word("mine") {
        expect_empty_call(input)?;
        return Ok(Some(ExecutableExpressionKind::Action(
            ExecutableAction::Mine,
        )));
    }

    if input.use_next_word("move") {
        return Ok(Some(parse_move_expression(input)?));
    }

    if input.use_next_word("rotate") {
        return Ok(Some(parse_rotate_expression(input)?));
    }

    if let Some(ore_type) = parse_named_dump_action(input)? {
        return Ok(Some(ExecutableExpressionKind::Action(
            ExecutableAction::Dump(ore_type),
        )));
    }

    if input.use_next_word("dump") {
        return Ok(Some(parse_dump_expression(input)?));
    }

    if input.use_next_word("time") {
        expect_empty_call(input)?;
        return Ok(Some(ExecutableExpressionKind::Time));
    }

    if input.use_next_word("abs") {
        return Ok(Some(ExecutableExpressionKind::Abs(Box::new(
            parse_call_argument(input)?,
        ))));
    }

    if input.use_next_word("sqrt") {
        return Ok(Some(ExecutableExpressionKind::Sqrt(Box::new(
            parse_call_argument(input)?,
        ))));
    }

    if input.use_next_word("sin") {
        return Ok(Some(ExecutableExpressionKind::Sin(Box::new(
            parse_call_argument(input)?,
        ))));
    }

    if input.use_next_word("cos") {
        return Ok(Some(ExecutableExpressionKind::Cos(Box::new(
            parse_call_argument(input)?,
        ))));
    }

    if input.use_next_word("tan") {
        return Ok(Some(ExecutableExpressionKind::Tan(Box::new(
            parse_call_argument(input)?,
        ))));
    }

    if input.use_next_word("min") {
        let (left, right) = parse_two_call_arguments(input)?;
        return Ok(Some(ExecutableExpressionKind::Min(
            Box::new(left),
            Box::new(right),
        )));
    }

    if input.use_next_word("max") {
        let (left, right) = parse_two_call_arguments(input)?;
        return Ok(Some(ExecutableExpressionKind::Max(
            Box::new(left),
            Box::new(right),
        )));
    }

    if input.use_next_word("scan") {
        return Ok(Some(parse_scan_call(input)?));
    }

    if input.use_next_word("oreDistance") {
        expect_empty_call(input)?;
        return Ok(Some(ExecutableExpressionKind::OreDistance));
    }

    if input.use_next_word("oreType") {
        expect_empty_call(input)?;
        return Ok(Some(ExecutableExpressionKind::OreType));
    }

    // Deprecated: prefer robot.oreStored / robot.oreStoredA|B|C. Kept for existing programs.
    if input.use_next_word("ore") {
        return Ok(Some(ExecutableExpressionKind::Ore(Box::new(
            parse_call_argument(input)?,
        ))));
    }

    if let Some(kind) = parse_builtin_property_expression(input)? {
        reject_builtin_property_mutation(input, &kind)?;
        return Ok(Some(kind));
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
            // Task 2: recognize calls to already-registered functions. Full call
            // expression parsing / forward refs land in Task 3.
            if input.peek() == Some('(') && input.functions.contains_key(&name) {
                let args = parse_call_arguments(input)?;
                return Ok(Some(ExecutableExpressionKind::Call { name, args }));
            }

            if input.eat_sequence("++") {
                variable_operator = VariableOperator::PostIncrement;
            } else if input.eat_sequence("--") {
                variable_operator = VariableOperator::PostDecrement;
            }
        }

        expect_declared_variable(input, &name)?;

        return Ok(Some(if variable_operator == VariableOperator::None {
            ExecutableExpressionKind::Variable(name)
        } else {
            ExecutableExpressionKind::VariableUpdate {
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

    Ok(input.extract_number_literal())
}

fn parse_call_argument(input: &mut CompileInput) -> Result<ExecutableExpression, CompileError> {
    expect_char(input, '(', "'(' expected")?;
    let value = parse_executable_expression(input)?.ok_or_else(|| {
        CompileError::new(format!(
            "Syntax error at line {}. value expected",
            input.current_line
        ))
    })?;
    expect_char(input, ')', "')' expected")?;
    Ok(value)
}

fn parse_two_call_arguments(
    input: &mut CompileInput,
) -> Result<(ExecutableExpression, ExecutableExpression), CompileError> {
    expect_char(input, '(', "'(' expected")?;
    let left = parse_executable_expression(input)?.ok_or_else(|| {
        CompileError::new(format!(
            "Syntax error at line {}. value expected",
            input.current_line
        ))
    })?;
    expect_char(input, ',', "',' expected")?;
    let right = parse_executable_expression(input)?.ok_or_else(|| {
        CompileError::new(format!(
            "Syntax error at line {}. value expected",
            input.current_line
        ))
    })?;
    expect_char(input, ')', "')' expected")?;
    Ok((left, right))
}
