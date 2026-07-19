use crate::types::{
    CompileError, ExecutableAction, ExecutableActionExpression, ExecutableExpression,
    ExecutableStatement, ExecutableStatementKind,
};

use super::super::input::{CompileInput, expect_char, expect_empty_call};
use super::expressions::parse_executable_expression;

pub(super) fn parse_executable_action_statement(
    input: &mut CompileInput,
) -> Result<ExecutableStatement, CompileError> {
    let source_line = input.current_line.min(u16::MAX as usize) as u16;

    if input.use_next_word("mine") {
        expect_empty_call(input)?;
        return Ok(ExecutableStatement::at(
            source_line,
            ExecutableStatementKind::Action(ExecutableAction::Mine),
        ));
    }

    if input.use_next_word("move") {
        let action = ExecutableActionExpression::Move(parse_executable_call_expression(input)?);
        return Ok(ExecutableStatement::at(
            source_line,
            action
                .static_action()
                .map(ExecutableStatementKind::Action)
                .unwrap_or(ExecutableStatementKind::DynamicAction(action)),
        ));
    }

    if input.use_next_word("rotate") {
        let action = ExecutableActionExpression::Rotate(parse_executable_call_expression(input)?);
        return Ok(ExecutableStatement::at(
            source_line,
            action
                .static_action()
                .map(ExecutableStatementKind::Action)
                .unwrap_or(ExecutableStatementKind::DynamicAction(action)),
        ));
    }

    if let Some(ore_type) = parse_named_dump_action(input)? {
        return Ok(ExecutableStatement::at(
            source_line,
            ExecutableStatementKind::Action(ExecutableAction::Dump(ore_type)),
        ));
    }

    if input.use_next_word("dump") {
        let action = match parse_dump_call_expression(input)? {
            DumpCall::All => ExecutableActionExpression::Dump(ExecutableExpression::Number(0.0)),
            DumpCall::Typed(expression) => ExecutableActionExpression::Dump(expression),
        };
        return Ok(ExecutableStatement::at(
            source_line,
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
) -> Result<ExecutableExpression, CompileError> {
    let expression = parse_executable_call_expression(input)?;
    if let Some(distance) = expression.literal_number() {
        Ok(ExecutableExpression::Action(ExecutableAction::Move(
            distance,
        )))
    } else {
        Ok(ExecutableExpression::Move(Box::new(expression)))
    }
}

pub(super) fn parse_rotate_expression(
    input: &mut CompileInput,
) -> Result<ExecutableExpression, CompileError> {
    let expression = parse_executable_call_expression(input)?;
    if let Some(rotation) = expression.literal_number() {
        Ok(ExecutableExpression::Action(ExecutableAction::Rotate(
            rotation,
        )))
    } else {
        Ok(ExecutableExpression::Rotate(Box::new(expression)))
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
) -> Result<ExecutableExpression, CompileError> {
    match parse_dump_call_expression(input)? {
        DumpCall::All => Ok(ExecutableExpression::Action(ExecutableAction::Dump(0))),
        DumpCall::Typed(expression) => {
            // Deprecated: prefer dump() / dumpA() / dumpB() / dumpC().
            if let Some(ore_type) = expression.literal_number() {
                Ok(ExecutableExpression::Action(ExecutableAction::Dump(
                    ore_type as i32,
                )))
            } else {
                Ok(ExecutableExpression::Dump(Box::new(expression)))
            }
        }
    }
}

pub(super) fn parse_scan_call(
    input: &mut CompileInput,
) -> Result<ExecutableExpression, CompileError> {
    expect_char(input, '(', "'(' expected")?;
    if input.eat_char(')', false) {
        return Ok(ExecutableExpression::Scan(None));
    }

    let direction = parse_executable_expression(input)?.ok_or_else(|| {
        CompileError::new(format!(
            "Syntax error at line {}. value expected",
            input.current_line
        ))
    })?;
    expect_char(input, ')', "')' expected")?;
    Ok(ExecutableExpression::Scan(Some(Box::new(direction))))
}
