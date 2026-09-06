use crate::compile::reserved::is_reserved_name;
use crate::types::{
    CompileError, ExecutableExpression, ExecutableExpressionKind, ExecutableFunction,
    ExecutableStatement, ExecutableStatementKind, FunctionParam, Operator, ValueType,
};

use super::super::input::{CompileInput, ProgramGlobal, expect_char};
use super::expressions::parse_executable_expression;
use super::statements::parse_executable_sequence;

/// Phase 1a: register every top-level function signature (brace-skip bodies) so later
/// full parse can resolve forward calls and check arity.
///
/// Skip failures on malformed non-function code return `Ok` with whatever signatures were
/// collected so the full parse can report the real diagnostic. Definite function-header
/// problems (duplicates, reserved names, committed `fn` forms) still return `Err`.
pub(super) fn collect_function_signatures(input: &mut CompileInput) -> Result<(), CompileError> {
    if input.peek() != Some('{') {
        return Ok(());
    }
    let _ = input.eat_char('{', false);
    input.eat_char(';', true);

    while input.peek() != Some('}') && !input.eof() {
        match try_collect_one_signature(input) {
            Ok(true) => {
                input.eat_char(';', true);
            }
            Ok(false) => match try_collect_top_level_var(input) {
                Ok(true) => {
                    input.eat_char(';', true);
                }
                Ok(false) => {
                    if skip_top_level_item(input).is_err() {
                        return Ok(());
                    }
                    input.eat_char(';', true);
                }
                Err(error) => return Err(error),
            },
            Err(error) => return Err(error),
        }
    }

    let _ = input.eat_char('}', false);
    Ok(())
}

fn try_collect_one_signature(input: &mut CompileInput) -> Result<bool, CompileError> {
    let checkpoint = input.checkpoint();

    if input.use_next_word("fn") {
        let explicit_return = parse_optional_value_type(input);
        let name = input.use_next_word_any();
        if name.is_empty() {
            return Err(CompileError::new(format!(
                "Syntax error at line {}. Function name expected",
                input.current_line
            )));
        }
        if input.peek() != Some('(') {
            return Err(CompileError::new(format!(
                "Syntax error at line {}. '(' expected",
                input.current_line
            )));
        }
        register_signature_skipping_body(input, name, explicit_return)?;
        return Ok(true);
    }

    let Some(return_type) = parse_optional_value_type(input) else {
        input.restore(checkpoint);
        return Ok(false);
    };

    if input.get_next_word() == "fn" {
        // `T fn name` is rejected during the full parse; skip this item instead.
        input.restore(checkpoint);
        return Ok(false);
    }

    let name = input.use_next_word_any();
    if name.is_empty() || input.peek() != Some('(') {
        input.restore(checkpoint);
        return Ok(false);
    }

    register_signature_skipping_body(input, name, Some(return_type))?;
    Ok(true)
}

/// Register a top-level `const? T name ...` declaration during the signature scan so
/// function bodies can resolve globals declared later in source order.
fn try_collect_top_level_var(input: &mut CompileInput) -> Result<bool, CompileError> {
    let checkpoint = input.checkpoint();
    let is_const = input.use_next_word("const");
    let Some(value_type) = parse_optional_value_type(input) else {
        input.restore(checkpoint);
        return Ok(false);
    };
    if input.get_next_word() == "fn" {
        input.restore(checkpoint);
        return Ok(false);
    }
    let name = input.use_next_word_any();
    if name.is_empty() || input.peek() == Some('(') {
        input.restore(checkpoint);
        return Ok(false);
    }

    if input.program_globals.contains_key(&name) {
        return Err(CompileError::new(format!(
            "Duplicate variable declaration at line {}: {}",
            input.current_line, name
        )));
    }

    input.program_globals.insert(
        name,
        ProgramGlobal {
            value_type,
            is_const,
        },
    );

    input.restore(checkpoint);
    skip_top_level_item(input)?;
    Ok(true)
}

fn register_signature_skipping_body(
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

    expect_char(input, '(', "'(' expected")?;
    let params = parse_function_params(input)?;
    expect_char(input, ')', "')' expected")?;

    if input.peek() != Some('{') {
        return Err(CompileError::new(format!(
            "Syntax error at line {}. '{{' expected",
            input.current_line
        )));
    }
    skip_braced_block(input)?;

    input.functions.insert(
        name.clone(),
        ExecutableFunction {
            name: name.clone(),
            return_type: explicit_return.unwrap_or(ValueType::Int),
            params,
            body: Vec::new(),
        },
    );
    input.pending_function_bodies.insert(name);
    Ok(())
}

fn skip_braced_block(input: &mut CompileInput) -> Result<(), CompileError> {
    expect_char(input, '{', "'{' expected")?;
    let mut depth = 1;
    while depth > 0 {
        match input.peek() {
            None => {
                return Err(CompileError::new(format!(
                    "Syntax error at line {}. '}}' expected",
                    input.current_line
                )));
            }
            Some('{') => {
                depth += 1;
                input.bump_token_or_char();
            }
            Some('}') => {
                depth -= 1;
                input.bump_token_or_char();
            }
            _ => input.bump_token_or_char(),
        }
    }
    Ok(())
}

fn skip_balanced_parens(input: &mut CompileInput) -> Result<(), CompileError> {
    expect_char(input, '(', "'(' expected")?;
    let mut depth = 1;
    while depth > 0 {
        match input.peek() {
            None => {
                return Err(CompileError::new(format!(
                    "Syntax error at line {}. ')' expected",
                    input.current_line
                )));
            }
            Some('(') => {
                depth += 1;
                input.bump_token_or_char();
            }
            Some(')') => {
                depth -= 1;
                input.bump_token_or_char();
            }
            Some('{') => skip_braced_block(input)?,
            _ => input.bump_token_or_char(),
        }
    }
    Ok(())
}

fn skip_top_level_item(input: &mut CompileInput) -> Result<(), CompileError> {
    if input.peek() == Some('{') {
        skip_braced_block(input)?;
        return Ok(());
    }

    let word = input.get_next_word().to_owned();
    match word.as_str() {
        "if" => {
            input.use_next_word_any();
            skip_balanced_parens(input)?;
            skip_embedded_statement(input)?;
            if input.use_next_word("else") {
                skip_embedded_statement(input)?;
            }
            Ok(())
        }
        "while" => {
            input.use_next_word_any();
            skip_balanced_parens(input)?;
            if input.eat_char(';', false) {
                return Ok(());
            }
            skip_embedded_statement(input)
        }
        "do" => {
            input.use_next_word_any();
            skip_embedded_statement(input)?;
            if !input.use_next_word("while") {
                return Err(CompileError::new(format!(
                    "Syntax error at line {}. 'while' expected",
                    input.current_line
                )));
            }
            skip_balanced_parens(input)?;
            Ok(())
        }
        _ => skip_simple_statement(input),
    }
}

fn skip_embedded_statement(input: &mut CompileInput) -> Result<(), CompileError> {
    if input.peek() == Some('{') {
        skip_braced_block(input)
    } else if input.eat_char(';', false) {
        Ok(())
    } else {
        skip_simple_statement(input)
    }
}

fn skip_simple_statement(input: &mut CompileInput) -> Result<(), CompileError> {
    let mut paren_depth = 0;
    loop {
        match input.peek() {
            None => {
                return Err(CompileError::new(format!(
                    "Syntax error at line {}. ';' expected",
                    input.current_line
                )));
            }
            Some(';') if paren_depth == 0 => {
                input.eat_char(';', false);
                return Ok(());
            }
            Some('}') if paren_depth == 0 => return Ok(()),
            Some('(') => {
                paren_depth += 1;
                input.bump_token_or_char();
            }
            Some(')') => {
                paren_depth -= 1;
                input.bump_token_or_char();
            }
            Some('{') if paren_depth == 0 => {
                // End of a brace-terminated construct (`T fn name() { ... }` skip path).
                skip_braced_block(input)?;
                return Ok(());
            }
            Some('{') => skip_braced_block(input)?,
            _ => input.bump_token_or_char(),
        }
    }
}

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

    let replacing_stub = input.pending_function_bodies.contains(&name);
    if input.functions.contains_key(&name) && !replacing_stub {
        return Err(CompileError::new(format!(
            "Duplicate function declaration at line {}: {}",
            input.current_line, name
        )));
    }

    if input.variables.contains(&name) || input.program_globals.contains_key(&name) {
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

    // Register / refresh a stub so recursive calls in this body can resolve the name.
    input.functions.insert(
        name.clone(),
        ExecutableFunction {
            name: name.clone(),
            return_type: explicit_return.unwrap_or(ValueType::Int),
            params: params.clone(),
            body: Vec::new(),
        },
    );
    input.pending_function_bodies.insert(name.clone());

    let outer_depth = input.variables.scope_depth;
    input.variables.set_scope_depth(outer_depth + 1);
    for param in &params {
        if input.variables.exists_at_current_level(&param.name) {
            input.variables.set_scope_depth(outer_depth);
            if !replacing_stub {
                input.functions.remove(&name);
                input.pending_function_bodies.remove(&name);
            }
            return Err(CompileError::new(format!(
                "Duplicate parameter declaration at line {}: {}",
                input.current_line, param.name
            )));
        }
        // Untyped params are known names but not a concrete type for inference.
        match param.value_type {
            Some(value_type) => {
                input
                    .variables
                    .declare(param.name.clone(), value_type, false);
            }
            None => input.variables.declare_untyped(param.name.clone()),
        }
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
            if !replacing_stub {
                input.functions.remove(&name);
                input.pending_function_bodies.remove(&name);
            }
            return Err(error);
        }
    };

    let body = match body_statement.kind {
        ExecutableStatementKind::Sequence(statements) => statements,
        kind => vec![ExecutableStatement::at(body_statement.source_span, kind)],
    };

    let return_type = match explicit_return {
        Some(value_type) => value_type,
        None => match infer_return_type(&body, input) {
            Ok(value_type) => value_type,
            Err(error) => {
                input.variables.set_scope_depth(outer_depth);
                if !replacing_stub {
                    input.functions.remove(&name);
                    input.pending_function_bodies.remove(&name);
                }
                return Err(error);
            }
        },
    };

    input.variables.set_scope_depth(outer_depth);
    input.pending_function_bodies.remove(&name);
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
    input: &CompileInput,
) -> Result<ValueType, CompileError> {
    let mut inferred: Option<ValueType> = None;
    collect_valued_return_types(body, input, &mut inferred)?;
    Ok(inferred.unwrap_or(ValueType::Int))
}

fn collect_valued_return_types(
    statements: &[ExecutableStatement],
    input: &CompileInput,
    inferred: &mut Option<ValueType>,
) -> Result<(), CompileError> {
    for statement in statements {
        match &statement.kind {
            ExecutableStatementKind::Return(Some(expr)) => {
                let Some(value_type) = expression_value_type(expr, input) else {
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
                collect_valued_return_types(inner, input, inferred)?;
            }
            ExecutableStatementKind::If {
                true_body,
                false_body,
                ..
            } => {
                collect_valued_return_types(std::slice::from_ref(true_body), input, inferred)?;
                if let Some(false_body) = false_body {
                    collect_valued_return_types(std::slice::from_ref(false_body), input, inferred)?;
                }
            }
            ExecutableStatementKind::While { body, .. } => {
                if let Some(body) = body {
                    collect_valued_return_types(std::slice::from_ref(body), input, inferred)?;
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
    input: &CompileInput,
) -> Option<ValueType> {
    match &expression.kind {
        ExecutableExpressionKind::Int(_) => Some(ValueType::Int),
        ExecutableExpressionKind::Float(_) => Some(ValueType::Double),
        ExecutableExpressionKind::Bool(_) => Some(ValueType::Bool),
        ExecutableExpressionKind::Variable(name)
        | ExecutableExpressionKind::VariableUpdate { name, .. } => {
            input.ast_variable_value_type(name)
        }
        ExecutableExpressionKind::UnaryNot(_) => Some(ValueType::Bool),
        ExecutableExpressionKind::UnaryMinus(inner)
        | ExecutableExpressionKind::Abs(inner)
        | ExecutableExpressionKind::Sqrt(inner)
        | ExecutableExpressionKind::Sin(inner)
        | ExecutableExpressionKind::Cos(inner)
        | ExecutableExpressionKind::Tan(inner) => expression_value_type(inner, input),
        ExecutableExpressionKind::Min(left, right) | ExecutableExpressionKind::Max(left, right) => {
            let left_type = expression_value_type(left, input)?;
            let right_type = expression_value_type(right, input)?;
            Some(promote_numeric(left_type, right_type))
        }
        ExecutableExpressionKind::Binary {
            operator,
            left,
            right,
        } => binary_result_type(*operator, left, right, input),
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
    input: &CompileInput,
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
            let left_type = expression_value_type(left, input)?;
            let right_type = expression_value_type(right, input)?;
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
