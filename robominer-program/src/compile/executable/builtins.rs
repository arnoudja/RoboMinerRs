use crate::types::{
    AreaProperty, CompileError, ExecutableExpression, ExecutableExpressionKind,
    ExecutableStatement, ExecutableStatementKind, RobotProperty,
};

use super::super::input::{
    CompileInput, SourceMark, area_property_mutation_error, robot_property_mutation_error,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum BuiltinObject {
    Robot,
    Area,
}

impl BuiltinObject {
    pub(super) fn from_word(name: &str) -> Option<Self> {
        match name {
            "robot" => Some(Self::Robot),
            "area" => Some(Self::Area),
            _ => None,
        }
    }

    fn word(self) -> &'static str {
        match self {
            Self::Robot => "robot",
            Self::Area => "area",
        }
    }

    fn property_expected_message(self) -> &'static str {
        match self {
            Self::Robot => "Robot property expected",
            Self::Area => "Area property expected",
        }
    }

    pub(super) fn mutation_error(self, line: usize) -> CompileError {
        match self {
            Self::Robot => robot_property_mutation_error(line),
            Self::Area => area_property_mutation_error(line),
        }
    }

    fn parse_property(
        self,
        property_name: &str,
        line: usize,
    ) -> Result<ExecutableExpressionKind, CompileError> {
        match self {
            Self::Robot => Ok(ExecutableExpressionKind::RobotProperty(
                RobotProperty::from_name(property_name, line)?,
            )),
            Self::Area => Ok(ExecutableExpressionKind::AreaProperty(
                AreaProperty::from_name(property_name, line)?,
            )),
        }
    }

    fn from_expression_kind(kind: &ExecutableExpressionKind) -> Option<Self> {
        match kind {
            ExecutableExpressionKind::RobotProperty(_) => Some(Self::Robot),
            ExecutableExpressionKind::AreaProperty(_) => Some(Self::Area),
            _ => None,
        }
    }
}

/// Parse `robot.name` / `area.name` when either word is next in the input.
pub(super) fn parse_builtin_property_expression(
    input: &mut CompileInput,
) -> Result<Option<ExecutableExpressionKind>, CompileError> {
    let object = if input.use_next_word("robot") {
        BuiltinObject::Robot
    } else if input.use_next_word("area") {
        BuiltinObject::Area
    } else {
        return Ok(None);
    };

    if !input.eat_char('.', false) {
        input.return_next_word(object.word().to_string());
        return Ok(None);
    }

    Ok(Some(parse_property_after_dot(input, object)?))
}

/// Reject `robot.x++` / `area.sizeX--` after a successful property parse.
pub(super) fn reject_builtin_property_mutation(
    input: &mut CompileInput,
    kind: &ExecutableExpressionKind,
) -> Result<(), CompileError> {
    if input.eat_sequence("++") || input.eat_sequence("--") {
        let Some(object) = BuiltinObject::from_expression_kind(kind) else {
            return Ok(());
        };
        return Err(object.mutation_error(input.current_line));
    }
    Ok(())
}

/// After consuming the object word as a statement name, parse `.property`.
pub(super) fn parse_builtin_property_statement(
    input: &mut CompileInput,
    mark: SourceMark,
    object: BuiltinObject,
) -> Result<ExecutableStatement, CompileError> {
    input.eat_char('.', false);
    let kind = parse_property_after_dot(input, object)?;
    if input.eat_char('=', false) || input.eat_sequence("++") || input.eat_sequence("--") {
        return Err(object.mutation_error(input.current_line));
    }
    let span = input.span_from(mark);
    Ok(ExecutableStatement::at(
        span,
        ExecutableStatementKind::Expression(ExecutableExpression::new(span, kind)),
    ))
}

fn parse_property_after_dot(
    input: &mut CompileInput,
    object: BuiltinObject,
) -> Result<ExecutableExpressionKind, CompileError> {
    let property_name = input.use_next_word_any();
    if property_name.is_empty() {
        return Err(CompileError::new(format!(
            "Syntax error at line {}. {}",
            input.current_line,
            object.property_expected_message()
        )));
    }
    object.parse_property(&property_name, input.current_line)
}
