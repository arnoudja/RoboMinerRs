use crate::compile::reserved::is_reserved_name;
use crate::types::{
    CompileError, ExecutableExpression, ExecutableExpressionKind, ExecutableFunction,
    ExecutableStatement, ExecutableStatementKind, FunctionParam, Operator, ValueType,
};

use super::super::input::{CompileInput, VariableStorage, expect_char};
use super::expressions::parse_executable_expression;
use super::statements::parse_executable_sequence;

/// Try to parse a top-level `fn [T] name(...)` definition.
///
/// Returns `Ok(Some(terminated))` when a definition was consumed, `Ok(None)` when the
/// next token is not an `fn` definition (caller may still try `T name(` via variable stmt).
pub(super) fn try_parse_fn_keyword_definition(
    input: &mut CompileInput,
) -> Result<Option<bool>, CompileError> {
    if !input.allow_function_defs || !input.use_next_word("fn") {
        return Ok(None);
    }

    let explicit_return = parse_optional_value_type(input);
    let name = input.use_next_word_any();
    if name.is_empty() {
        return Err(CompileError::new(format!(
            "Syntax error at line {}. Function name expected",
            input.current_line
        )));
    }

    parse_function_after_name(input, name, explicit_return)?;
    Ok(Some(true))
}

/// Parse `T name(...)` after the type and name have already been consumed.
pub(super) fn parse_typed_name_function(
    input: &mut CompileInput,
    name: String,
    return_type: ValueType,
) -> Result<(), CompileError> {
    parse_function_after_name(input, name, Some(return_type))
}

fn parse_function_after_name(
    input: &mut CompileInput,
    name: String,
    explicit_return: Option<ValueType>,
) -> Result<(), CompileError> {
    if is_reserved_name(&name) {
        return Err(CompileError::new(format!(
            "Error at line {}: '{}' is a reserved name and cannot be used as a function",
            input.current_line, name
        )));
    }

    if input.functions.contains_key(&name) {
        return Err(CompileError::new(format!(
            "Duplicate function declaration at line {}: {}",
            input.current_line, name
        )));
    }

    if input.variables.contains(&name) {
        return Err(CompileError::new(format!(
            "Error at line {}: function name '{}' conflicts with a variable",
            input.current_line, name
        )));
    }

    expect_char(input, '(', "'(' expected")?;
    let params = parse_function_params(input)?;
    expect_char(input, ')', "')' expected")?;

    if input.peek() != Some('{') {
        return Err(CompileError::new(format!(
            "Syntax error at line {}. '{{' expected",
            input.current_line
        )));
    }

    // Register a stub so recursive / later calls in this body can resolve the name.
    input.functions.insert(
        name.clone(),
        ExecutableFunction {
            name: name.clone(),
            return_type: explicit_return.unwrap_or(ValueType::Int),
            params: params.clone(),
            body: Vec::new(),
        },
    );

    let outer_depth = input.variables.scope_depth;
    input.variables.set_scope_depth(outer_depth + 1);
    for param in &params {
        let value_type = param.value_type.unwrap_or(ValueType::Int);
        if input.variables.exists_at_current_level(&param.name) {
            input.variables.set_scope_depth(outer_depth);
            input.functions.remove(&name);
            return Err(CompileError::new(format!(
                "Duplicate parameter declaration at line {}: {}",
                input.current_line, param.name
            )));
        }
        input
            .variables
            .declare(param.name.clone(), value_type, false);
    }

    let was_in_function = input.in_function_body;
    let was_allow_defs = input.allow_function_defs;
    input.in_function_body = true;
    input.allow_function_defs = false;
    let body_statement = parse_executable_sequence(input);
    input.in_function_body = was_in_function;
    input.allow_function_defs = was_allow_defs;

    let body_statement = match body_statement {
        Ok(statement) => statement,
        Err(error) => {
            input.variables.set_scope_depth(outer_depth);
            input.functions.remove(&name);
            return Err(error);
        }
    };

    let body = match body_statement.kind {
        ExecutableStatementKind::Sequence(statements) => statements,
        kind => vec![ExecutableStatement::at(body_statement.source_span, kind)],
    };

    let return_type = match explicit_return {
        Some(value_type) => value_type,
        None => match infer_return_type(&body, &input.variables) {
            Ok(value_type) => value_type,
            Err(error) => {
                input.variables.set_scope_depth(outer_depth);
                input.functions.remove(&name);
                return Err(error);
            }
        },
    };

    input.variables.set_scope_depth(outer_depth);
    input.functions.insert(
        name.clone(),
        ExecutableFunction {
            name,
            return_type,
            params,
            body,
        },
    );
    Ok(())
}

fn parse_function_params(input: &mut CompileInput) -> Result<Vec<FunctionParam>, CompileError> {
    let mut params = Vec::new();
    if input.peek() == Some(')') {
        return Ok(params);
    }

    loop {
        let value_type = parse_optional_value_type(input);
        let name = input.use_next_word_any();
        if name.is_empty() {
            return Err(CompileError::new(format!(
                "Syntax error at line {}. Parameter name expected",
                input.current_line
            )));
        }
        params.push(FunctionParam { name, value_type });

        if input.eat_char(',', false) {
            continue;
        }
        break;
    }

    Ok(params)
}

pub(super) fn parse_optional_value_type(input: &mut CompileInput) -> Option<ValueType> {
    if input.use_next_word("int") {
        Some(ValueType::Int)
    } else if input.use_next_word("double") || input.use_next_word("float") {
        Some(ValueType::Double)
    } else if input.use_next_word("bool") {
        Some(ValueType::Bool)
    } else {
        None
    }
}

fn infer_return_type(
    body: &[ExecutableStatement],
    variables: &VariableStorage,
) -> Result<ValueType, CompileError> {
    let mut inferred: Option<ValueType> = None;
    collect_valued_return_types(body, variables, &mut inferred)?;
    Ok(inferred.unwrap_or(ValueType::Int))
}

fn collect_valued_return_types(
    statements: &[ExecutableStatement],
    variables: &VariableStorage,
    inferred: &mut Option<ValueType>,
) -> Result<(), CompileError> {
    for statement in statements {
        match &statement.kind {
            ExecutableStatementKind::Return(Some(expr)) => {
                let Some(value_type) = expression_value_type(expr, variables) else {
                    return Err(CompileError::new(
                        "cannot infer return type; give an explicit return type".to_string(),
                    ));
                };
                match *inferred {
                    None => *inferred = Some(value_type),
                    Some(existing) if existing != value_type => {
                        return Err(CompileError::new("conflicting return types".to_string()));
                    }
                    Some(_) => {}
                }
            }
            ExecutableStatementKind::Sequence(inner) => {
                collect_valued_return_types(inner, variables, inferred)?;
            }
            ExecutableStatementKind::If {
                true_body,
                false_body,
                ..
            } => {
                collect_valued_return_types(std::slice::from_ref(true_body), variables, inferred)?;
                if let Some(false_body) = false_body {
                    collect_valued_return_types(
                        std::slice::from_ref(false_body),
                        variables,
                        inferred,
                    )?;
                }
            }
            ExecutableStatementKind::While { body, .. } => {
                if let Some(body) = body {
                    collect_valued_return_types(std::slice::from_ref(body), variables, inferred)?;
                }
            }
            ExecutableStatementKind::Return(None)
            | ExecutableStatementKind::Action(_)
            | ExecutableStatementKind::DynamicAction(_)
            | ExecutableStatementKind::Declare { .. }
            | ExecutableStatementKind::Assign { .. }
            | ExecutableStatementKind::Expression(_) => {}
        }
    }
    Ok(())
}

fn expression_value_type(
    expression: &ExecutableExpression,
    variables: &VariableStorage,
) -> Option<ValueType> {
    match &expression.kind {
        ExecutableExpressionKind::Int(_) => Some(ValueType::Int),
        ExecutableExpressionKind::Float(_) => Some(ValueType::Double),
        ExecutableExpressionKind::Bool(_) => Some(ValueType::Bool),
        ExecutableExpressionKind::Variable(name)
        | ExecutableExpressionKind::VariableUpdate { name, .. } => variables.value_type(name),
        ExecutableExpressionKind::UnaryNot(_) => Some(ValueType::Bool),
        ExecutableExpressionKind::UnaryMinus(inner)
        | ExecutableExpressionKind::Abs(inner)
        | ExecutableExpressionKind::Sqrt(inner)
        | ExecutableExpressionKind::Sin(inner)
        | ExecutableExpressionKind::Cos(inner)
        | ExecutableExpressionKind::Tan(inner) => expression_value_type(inner, variables),
        ExecutableExpressionKind::Min(left, right) | ExecutableExpressionKind::Max(left, right) => {
            let left_type = expression_value_type(left, variables)?;
            let right_type = expression_value_type(right, variables)?;
            Some(promote_numeric(left_type, right_type))
        }
        ExecutableExpressionKind::Binary {
            operator,
            left,
            right,
        } => binary_result_type(*operator, left, right, variables),
        ExecutableExpressionKind::Time
        | ExecutableExpressionKind::OreDistance
        | ExecutableExpressionKind::Scan(_)
        | ExecutableExpressionKind::Move(_)
        | ExecutableExpressionKind::RobotProperty(_) => Some(ValueType::Double),
        ExecutableExpressionKind::OreType
        | ExecutableExpressionKind::Ore(_)
        | ExecutableExpressionKind::AreaProperty(_)
        | ExecutableExpressionKind::Rotate(_)
        | ExecutableExpressionKind::Dump(_)
        | ExecutableExpressionKind::Action(_) => Some(ValueType::Int),
        ExecutableExpressionKind::Call { .. } => None,
    }
}

fn binary_result_type(
    operator: Operator,
    left: &ExecutableExpression,
    right: &ExecutableExpression,
    variables: &VariableStorage,
) -> Option<ValueType> {
    match operator {
        Operator::Larger
        | Operator::Smaller
        | Operator::LargerEqual
        | Operator::SmallerEqual
        | Operator::Equal
        | Operator::NotEqual
        | Operator::And
        | Operator::Or => Some(ValueType::Bool),
        Operator::Mod => Some(ValueType::Int),
        Operator::Addition | Operator::Subtraction | Operator::Multiply | Operator::Division => {
            let left_type = expression_value_type(left, variables)?;
            let right_type = expression_value_type(right, variables)?;
            Some(promote_numeric(left_type, right_type))
        }
        Operator::Undefined => None,
    }
}

fn promote_numeric(left: ValueType, right: ValueType) -> ValueType {
    if left == ValueType::Double || right == ValueType::Double {
        ValueType::Double
    } else {
        ValueType::Int
    }
}

pub(super) fn parse_call_arguments(
    input: &mut CompileInput,
) -> Result<Vec<ExecutableExpression>, CompileError> {
    expect_char(input, '(', "'(' expected")?;
    let mut args = Vec::new();
    if input.peek() != Some(')') {
        loop {
            let arg = parse_executable_expression(input)?.ok_or_else(|| {
                CompileError::new(format!(
                    "Syntax error at line {}. Expression expected",
                    input.current_line
                ))
            })?;
            args.push(arg);
            if input.eat_char(',', false) {
                continue;
            }
            break;
        }
    }
    expect_char(input, ')', "')' expected")?;
    Ok(args)
}
