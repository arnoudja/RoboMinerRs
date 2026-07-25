use crate::types::{
    CompileError, ExecutableAction, ExecutableActionExpression, ExecutableExpression,
    ExecutableExpressionKind, ExecutableStatement, ExecutableStatementKind,
};

use super::super::input::{CompileInput, expect_char, expect_empty_call};
use super::expressions::parse_executable_expression;

pub(super) fn parse_executable_action_statement(
    input: &mut CompileInput,
) -> Result<ExecutableStatement, CompileError> {
    let mark = input.mark_pos();

    if input.use_next_word("mine") {
        expect_empty_call(input)?;
        return Ok(ExecutableStatement::at(
            input.span_from(mark),
            ExecutableStatementKind::Action(ExecutableAction::Mine),
        ));
    }

    if input.use_next_word("move") {
        let action = ExecutableActionExpression::Move(parse_executable_call_expression(input)?);
        return Ok(ExecutableStatement::at(
            input.span_from(mark),
            action
                .static_action()
                .map(ExecutableStatementKind::Action)
                .unwrap_or(ExecutableStatementKind::DynamicAction(action)),
        ));
    }

    if input.use_next_word("rotate") {
        let action = ExecutableActionExpression::Rotate(parse_executable_call_expression(input)?);
        return Ok(ExecutableStatement::at(
            input.span_from(mark),
            action
                .static_action()
                .map(ExecutableStatementKind::Action)
                .unwrap_or(ExecutableStatementKind::DynamicAction(action)),
        ));
    }

    if let Some(ore_type) = parse_named_dump_action(input)? {
        return Ok(ExecutableStatement::at(
            input.span_from(mark),
            ExecutableStatementKind::Action(ExecutableAction::Dump(ore_type)),
        ));
    }

    if input.use_next_word("dump") {
        let call = parse_dump_call_expression(input)?;
        let span = input.span_from(mark);
        let action = match call {
            DumpCall::All => ExecutableActionExpression::Dump(ExecutableExpression::new(
                span,
                ExecutableExpressionKind::Number(0.0),
            )),
            DumpCall::Typed(expression) => ExecutableActionExpression::Dump(expression),
        };
        return Ok(ExecutableStatement::at(
            span,
            action
                .static_action()
                .map(ExecutableStatementKind::Action)
                .unwrap_or(ExecutableStatementKind::DynamicAction(action)),
        ));
    }

    Err(CompileError::new(format!(
        "Executable program support currently handles move, rotate, mine, dump, dumpA, dumpB, dumpC, if, while, and do-while at line {}",
        input.current_line
    )))
}

pub(super) fn parse_executable_call_expression(
    input: &mut CompileInput,
) -> Result<ExecutableExpression, CompileError> {
    expect_char(input, '(', "'(' expected")?;
    let expression = parse_executable_expression(input)?.ok_or_else(|| {
        CompileError::new(format!(
            "Executable program support currently requires numeric arguments at line {}",
            input.current_line
        ))
    })?;
    expect_char(input, ')', "')' expected")?;

    Ok(expression)
}

pub(super) fn parse_move_expression(
    input: &mut CompileInput,
) -> Result<ExecutableExpressionKind, CompileError> {
    let expression = parse_executable_call_expression(input)?;
    if let Some(distance) = expression.literal_number() {
        Ok(ExecutableExpressionKind::Action(ExecutableAction::Move(
            distance,
        )))
    } else {
        Ok(ExecutableExpressionKind::Move(Box::new(expression)))
    }
}

pub(super) fn parse_rotate_expression(
    input: &mut CompileInput,
) -> Result<ExecutableExpressionKind, CompileError> {
    let expression = parse_executable_call_expression(input)?;
    if let Some(rotation) = expression.literal_number() {
        Ok(ExecutableExpressionKind::Action(ExecutableAction::Rotate(
            rotation,
        )))
    } else {
        Ok(ExecutableExpressionKind::Rotate(Box::new(expression)))
    }
}

/// Named dump helpers aligned with `robot.oreStoredA|B|C` (1-based quality slots).
pub(super) fn parse_named_dump_action(
    input: &mut CompileInput,
) -> Result<Option<i32>, CompileError> {
    let ore_type = if input.use_next_word("dumpA") {
        1
    } else if input.use_next_word("dumpB") {
        2
    } else if input.use_next_word("dumpC") {
        3
    } else {
        return Ok(None);
    };
    expect_empty_call(input)?;
    Ok(Some(ore_type))
}

pub(super) enum DumpCall {
    All,
    Typed(ExecutableExpression),
}

/// Parse `dump()`, `dump(expr)`.
///
/// - `dump()` dumps all ore types.
/// - `dump(<value>)` is deprecated but kept for existing programs (0 = all, 1/2/3 = A/B/C).
pub(super) fn parse_dump_call_expression(
    input: &mut CompileInput,
) -> Result<DumpCall, CompileError> {
    expect_char(input, '(', "'(' expected")?;
    if input.eat_char(')', false) {
        return Ok(DumpCall::All);
    }

    let expression = parse_executable_expression(input)?.ok_or_else(|| {
        CompileError::new(format!(
            "Executable program support currently requires numeric arguments at line {}",
            input.current_line
        ))
    })?;
    expect_char(input, ')', "')' expected")?;
    Ok(DumpCall::Typed(expression))
}

pub(super) fn parse_dump_expression(
    input: &mut CompileInput,
) -> Result<ExecutableExpressionKind, CompileError> {
    match parse_dump_call_expression(input)? {
        DumpCall::All => Ok(ExecutableExpressionKind::Action(ExecutableAction::Dump(0))),
        DumpCall::Typed(expression) => {
            // Deprecated: prefer dump() / dumpA() / dumpB() / dumpC().
            if let Some(ore_type) = expression.literal_number() {
                Ok(ExecutableExpressionKind::Action(ExecutableAction::Dump(
                    ore_type as i32,
                )))
            } else {
                Ok(ExecutableExpressionKind::Dump(Box::new(expression)))
            }
        }
    }
}

pub(super) fn parse_scan_call(
    input: &mut CompileInput,
) -> Result<ExecutableExpressionKind, CompileError> {
    expect_char(input, '(', "'(' expected")?;
    if input.eat_char(')', false) {
        return Ok(ExecutableExpressionKind::Scan(None));
    }

    let direction = parse_executable_expression(input)?.ok_or_else(|| {
        CompileError::new(format!(
            "Syntax error at line {}. value expected",
            input.current_line
        ))
    })?;
    expect_char(input, ')', "')' expected")?;
    Ok(ExecutableExpressionKind::Scan(Some(Box::new(direction))))
}
