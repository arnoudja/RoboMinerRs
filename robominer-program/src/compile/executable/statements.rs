use crate::types::{
    CompileError, ExecutableExpression, ExecutableExpressionKind, ExecutableStatement,
    ExecutableStatementKind, Operator, SourceSpan, VariableOperator,
};

use super::super::input::{CompileInput, SourceMark, expect_char};
use super::actions::parse_executable_action_statement;
use super::builtins::{BuiltinObject, parse_builtin_property_statement};
use super::expect_declared_variable;
use super::expressions::parse_executable_expression;
use super::functions::{
    parse_optional_value_type, parse_typed_name_function, try_parse_fn_keyword_definition,
};

pub(super) fn parse_executable_sequence(
    input: &mut CompileInput,
) -> Result<ExecutableStatement, CompileError> {
    let mark = input.mark_pos();
    expect_char(input, '{', "'{' expected")?;

    // Only the outermost program sequence may define functions; capture then clear so
    // nested blocks (and recursive sequence calls) reject nested defs.
    let allow_function_defs = input.allow_function_defs;
    input.allow_function_defs = false;

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

        input.allow_function_defs = allow_function_defs;
        if let Some(_terminated) = try_parse_fn_keyword_definition(input)? {
            input.allow_function_defs = false;
            input.eat_char(';', true);
            previous_terminated = true;
            continue;
        }

        if input.peek() == Some('{') {
            input.allow_function_defs = false;
            statements.push(parse_executable_sequence(input)?);
            input.eat_char(';', true);
            previous_terminated = true;
        } else {
            match parse_executable_statement(input)? {
                StatementParse::FunctionDef => {
                    input.allow_function_defs = false;
                    input.eat_char(';', true);
                    previous_terminated = true;
                }
                StatementParse::Statement {
                    statement,
                    terminated,
                } => {
                    input.allow_function_defs = false;
                    statements.push(statement);
                    previous_terminated = input.eat_char(';', true) || terminated;
                }
            }
        }
    }

    input.allow_function_defs = false;
    expect_char(input, '}', "'}' expected")?;
    input.variables.set_scope_depth(outer_scope);

    Ok(ExecutableStatement::at(
        input.span_from(mark),
        ExecutableStatementKind::Sequence(statements),
    ))
}

enum StatementParse {
    FunctionDef,
    Statement {
        statement: ExecutableStatement,
        terminated: bool,
    },
}

fn parse_executable_statement(input: &mut CompileInput) -> Result<StatementParse, CompileError> {
    let mark = input.mark_pos();

    if !input.allow_function_defs && input.get_next_word() == "fn" {
        return Err(CompileError::new(format!(
            "Syntax error at line {}. Nested function definitions are not allowed",
            input.current_line
        )));
    }

    if input.use_next_word("return") {
        if !input.in_function_body {
            return Err(CompileError::new(format!(
                "Error at line {}: 'return' is only allowed inside a function",
                input.current_line
            )));
        }
        let value = if matches!(input.peek(), Some(';' | '}')) {
            None
        } else {
            parse_executable_expression(input)?
        };
        return Ok(StatementParse::Statement {
            statement: ExecutableStatement::at(
                input.span_from(mark),
                ExecutableStatementKind::Return(value),
            ),
            terminated: false,
        });
    }

    match parse_executable_variable_statement(input, mark)? {
        VariableParse::FunctionDef => return Ok(StatementParse::FunctionDef),
        VariableParse::Statement(statement) => {
            return Ok(StatementParse::Statement {
                statement,
                terminated: false,
            });
        }
        VariableParse::NotVariable => {}
    }

    if input.use_next_word("while") {
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

        return Ok(StatementParse::Statement {
            statement: ExecutableStatement::at(
                input.span_from(mark),
                ExecutableStatementKind::While {
                    condition,
                    body,
                    is_do_while: false,
                },
            ),
            terminated: true,
        });
    }

    if input.use_next_word("do") {
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

        return Ok(StatementParse::Statement {
            statement: ExecutableStatement::at(
                input.span_from(mark),
                ExecutableStatementKind::While {
                    condition,
                    body,
                    is_do_while: true,
                },
            ),
            terminated,
        });
    }

    if input.use_next_word("if") {
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

        return Ok(StatementParse::Statement {
            statement: ExecutableStatement::at(
                input.span_from(mark),
                ExecutableStatementKind::If {
                    condition,
                    true_body,
                    false_body,
                },
            ),
            terminated: true,
        });
    }

    Ok(StatementParse::Statement {
        statement: parse_executable_expression_statement(input, mark)?,
        terminated: false,
    })
}

fn parse_executable_expression_statement(
    input: &mut CompileInput,
    mark: SourceMark,
) -> Result<ExecutableStatement, CompileError> {
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
        input.span_from(mark),
        ExecutableStatementKind::Expression(expression),
    ))
}

fn parse_executable_item(input: &mut CompileInput) -> Result<ExecutableStatement, CompileError> {
    if input.peek() == Some('{') {
        parse_executable_sequence(input)
    } else {
        match parse_executable_statement(input)? {
            StatementParse::FunctionDef => Err(CompileError::new(format!(
                "Syntax error at line {}. Nested function definitions are not allowed",
                input.current_line
            ))),
            StatementParse::Statement {
                statement,
                terminated,
            } => {
                if !terminated {
                    expect_char(input, ';', "';' expected")?;
                }
                Ok(statement)
            }
        }
    }
}

enum VariableParse {
    NotVariable,
    FunctionDef,
    Statement(ExecutableStatement),
}

fn parse_executable_variable_statement(
    input: &mut CompileInput,
    mark: SourceMark,
) -> Result<VariableParse, CompileError> {
    let is_const = input.use_next_word("const");

    let value_type = parse_optional_value_type(input);

    if let Some(value_type) = value_type {
        if input.get_next_word() == "fn" {
            return Err(CompileError::new(format!(
                "Syntax error at line {}. Unexpected 'fn' after type; use 'fn T name' or 'T name'",
                input.current_line
            )));
        }

        let name = input.use_next_word_any();
        if name.is_empty() {
            return Err(CompileError::new(format!(
                "Syntax error at line {}. Identifier expected",
                input.current_line
            )));
        }

        if input.allow_function_defs && !is_const && input.peek() == Some('(') {
            parse_typed_name_function(input, name, value_type)?;
            return Ok(VariableParse::FunctionDef);
        }

        if input.functions.contains_key(&name) {
            return Err(CompileError::new(format!(
                "Error at line {}: variable name '{}' conflicts with a function",
                input.current_line, name
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

        return Ok(VariableParse::Statement(ExecutableStatement::at(
            input.span_from(mark),
            ExecutableStatementKind::Declare {
                name,
                value_type,
                value,
            },
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
        return Ok(VariableParse::NotVariable);
    }

    // Covers the leading `++`/`--` too, so it doubles as the span of the name reference.
    let name_span = input.span_from(mark);

    if variable_operator != VariableOperator::None {
        expect_declared_variable(input, &name)?;
        return Ok(VariableParse::Statement(ExecutableStatement::at(
            name_span,
            ExecutableStatementKind::Expression(ExecutableExpression::new(
                name_span,
                ExecutableExpressionKind::VariableUpdate {
                    name,
                    operator: variable_operator,
                },
            )),
        )));
    }

    if input.eat_sequence("+=") {
        return Ok(VariableParse::Statement(parse_compound_assignment(
            input,
            mark,
            name,
            name_span,
            Operator::Addition,
        )?));
    }

    if input.eat_sequence("-=") {
        return Ok(VariableParse::Statement(parse_compound_assignment(
            input,
            mark,
            name,
            name_span,
            Operator::Subtraction,
        )?));
    }

    if input.eat_char('=', false) {
        expect_declared_variable(input, &name)?;
        if input.variable_is_const(&name) {
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
        return Ok(VariableParse::Statement(ExecutableStatement::at(
            input.span_from(mark),
            ExecutableStatementKind::Assign { name, value },
        )));
    }

    if input.eat_sequence("++") {
        expect_declared_variable(input, &name)?;
        let span = input.span_from(mark);
        return Ok(VariableParse::Statement(ExecutableStatement::at(
            span,
            ExecutableStatementKind::Expression(ExecutableExpression::new(
                span,
                ExecutableExpressionKind::VariableUpdate {
                    name,
                    operator: VariableOperator::PostIncrement,
                },
            )),
        )));
    }

    if input.eat_sequence("--") {
        expect_declared_variable(input, &name)?;
        let span = input.span_from(mark);
        return Ok(VariableParse::Statement(ExecutableStatement::at(
            span,
            ExecutableStatementKind::Expression(ExecutableExpression::new(
                span,
                ExecutableExpressionKind::VariableUpdate {
                    name,
                    operator: VariableOperator::PostDecrement,
                },
            )),
        )));
    }

    if let Some(object) = BuiltinObject::from_word(&name)
        && input.peek() == Some('.')
    {
        return Ok(VariableParse::Statement(parse_builtin_property_statement(
            input, mark, object,
        )?));
    }

    input.return_next_word(name);
    Ok(VariableParse::NotVariable)
}

fn parse_compound_assignment(
    input: &mut CompileInput,
    mark: SourceMark,
    name: String,
    name_span: SourceSpan,
    operator: Operator,
) -> Result<ExecutableStatement, CompileError> {
    expect_declared_variable(input, &name)?;
    if input.variable_is_const(&name) {
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
    let span = input.span_from(mark);
    Ok(ExecutableStatement::at(
        span,
        ExecutableStatementKind::Assign {
            name: name.clone(),
            value: ExecutableExpression::new(
                span,
                ExecutableExpressionKind::Binary {
                    operator,
                    left: Box::new(ExecutableExpression::new(
                        name_span,
                        ExecutableExpressionKind::Variable(name),
                    )),
                    right: Box::new(rhs),
                },
            ),
        },
    ))
}
