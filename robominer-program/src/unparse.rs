use crate::types::{
    ExecutableAction, ExecutableActionExpression, ExecutableExpression, ExecutableProgram,
    ExecutableStatement, ExecutableStatementKind, Operator, VariableOperator,
};

/// Emit Edit-code-legal source for an executable program AST.
pub fn unparse_program(program: &ExecutableProgram) -> String {
    let mut out = String::new();
    for (index, statement) in program.statements.iter().enumerate() {
        if index > 0 {
            out.push('\n');
        }
        unparse_statement(statement, &mut out);
    }
    out
}

fn unparse_statement(statement: &ExecutableStatement, out: &mut String) {
    match &statement.kind {
        ExecutableStatementKind::Action(action) => {
            unparse_action(action, out);
            out.push(';');
        }
        ExecutableStatementKind::DynamicAction(action) => {
            unparse_dynamic_action(action, out);
            out.push(';');
        }
        ExecutableStatementKind::Sequence(statements) => {
            out.push('{');
            if !statements.is_empty() {
                out.push(' ');
                for (index, child) in statements.iter().enumerate() {
                    if index > 0 {
                        out.push(' ');
                    }
                    unparse_statement(child, out);
                }
                out.push(' ');
            }
            out.push('}');
        }
        ExecutableStatementKind::Declare { name, value } => {
            out.push_str("int ");
            out.push_str(name);
            if let Some(value) = value {
                out.push_str(" = ");
                unparse_expression(value, out, 0);
            }
            out.push(';');
        }
        ExecutableStatementKind::Assign { name, value } => {
            out.push_str(name);
            out.push_str(" = ");
            unparse_expression(value, out, 0);
            out.push(';');
        }
        ExecutableStatementKind::Expression(expression) => {
            unparse_expression(expression, out, 0);
            out.push(';');
        }
        ExecutableStatementKind::If {
            condition,
            true_body,
            false_body,
        } => {
            out.push_str("if (");
            unparse_expression(condition, out, 0);
            out.push_str(") ");
            unparse_statement(true_body, out);
            if let Some(false_body) = false_body {
                out.push_str(" else ");
                unparse_statement(false_body, out);
            }
        }
        ExecutableStatementKind::While {
            condition,
            body,
            is_do_while,
        } => {
            if *is_do_while {
                out.push_str("do ");
                match body.as_deref() {
                    Some(body) => unparse_statement(body, out),
                    None => out.push_str("{}"),
                }
                out.push_str(" while (");
                unparse_expression(condition, out, 0);
                out.push(')');
                out.push(';');
            } else {
                out.push_str("while (");
                unparse_expression(condition, out, 0);
                out.push(')');
                match body.as_deref() {
                    Some(body) => {
                        out.push(' ');
                        unparse_statement(body, out);
                    }
                    None => out.push(';'),
                }
            }
        }
    }
}

fn unparse_action(action: &ExecutableAction, out: &mut String) {
    match action {
        ExecutableAction::Move(distance) => {
            out.push_str("move(");
            unparse_number(*distance, out);
            out.push(')');
        }
        ExecutableAction::Rotate(rotation) => {
            out.push_str("rotate(");
            unparse_number(*rotation, out);
            out.push(')');
        }
        ExecutableAction::Mine => out.push_str("mine()"),
        ExecutableAction::Dump(ore_type) => {
            out.push_str("dump(");
            out.push_str(&ore_type.to_string());
            out.push(')');
        }
        ExecutableAction::StartScan(direction) => {
            out.push_str("scan(");
            if *direction != 0.0 {
                unparse_number(*direction, out);
            }
            out.push(')');
        }
        ExecutableAction::AwaitScanResult => out.push_str("scan()"),
    }
}

fn unparse_dynamic_action(action: &ExecutableActionExpression, out: &mut String) {
    match action {
        ExecutableActionExpression::Move(expression) => {
            out.push_str("move(");
            unparse_expression(expression, out, 0);
            out.push(')');
        }
        ExecutableActionExpression::Rotate(expression) => {
            out.push_str("rotate(");
            unparse_expression(expression, out, 0);
            out.push(')');
        }
        ExecutableActionExpression::Dump(expression) => {
            out.push_str("dump(");
            unparse_expression(expression, out, 0);
            out.push(')');
        }
    }
}

fn unparse_expression(expression: &ExecutableExpression, out: &mut String, parent_priority: usize) {
    match expression {
        ExecutableExpression::Number(value) => unparse_number(*value, out),
        ExecutableExpression::Variable(name) => out.push_str(name),
        ExecutableExpression::VariableUpdate { name, operator } => match operator {
            VariableOperator::None => out.push_str(name),
            VariableOperator::PreIncrement => {
                out.push_str("++");
                out.push_str(name);
            }
            VariableOperator::PreDecrement => {
                out.push_str("--");
                out.push_str(name);
            }
            VariableOperator::PostIncrement => {
                out.push_str(name);
                out.push_str("++");
            }
            VariableOperator::PostDecrement => {
                out.push_str(name);
                out.push_str("--");
            }
        },
        ExecutableExpression::UnaryNot(inner) => {
            out.push('!');
            unparse_expression(inner, out, usize::MAX);
        }
        ExecutableExpression::Binary {
            operator,
            left,
            right,
        } => {
            let priority = operator.priority();
            let needs_parens = priority < parent_priority;
            if needs_parens {
                out.push('(');
            }
            unparse_expression(left, out, priority);
            out.push(' ');
            out.push_str(operator.as_token());
            out.push(' ');
            // Right side of left-associative ops needs stricter binding.
            let right_priority = if matches!(
                operator,
                Operator::Subtraction
                    | Operator::Division
                    | Operator::Mod
                    | Operator::Larger
                    | Operator::Smaller
                    | Operator::LargerEqual
                    | Operator::SmallerEqual
                    | Operator::Equal
                    | Operator::NotEqual
            ) {
                priority + 1
            } else {
                priority
            };
            unparse_expression(right, out, right_priority);
            if needs_parens {
                out.push(')');
            }
        }
        ExecutableExpression::Time => out.push_str("time()"),
        ExecutableExpression::Ore(inner) => {
            out.push_str("ore(");
            unparse_expression(inner, out, 0);
            out.push(')');
        }
        ExecutableExpression::Scan(direction) => {
            out.push_str("scan(");
            if let Some(direction) = direction {
                unparse_expression(direction, out, 0);
            }
            out.push(')');
        }
        ExecutableExpression::OreDistance => out.push_str("oreDistance()"),
        ExecutableExpression::OreType => out.push_str("oreType()"),
        ExecutableExpression::RobotProperty(property) => {
            out.push_str("robot.");
            out.push_str(property.as_name());
        }
        ExecutableExpression::Move(inner) => {
            out.push_str("move(");
            unparse_expression(inner, out, 0);
            out.push(')');
        }
        ExecutableExpression::Rotate(inner) => {
            out.push_str("rotate(");
            unparse_expression(inner, out, 0);
            out.push(')');
        }
        ExecutableExpression::Dump(inner) => {
            out.push_str("dump(");
            unparse_expression(inner, out, 0);
            out.push(')');
        }
        ExecutableExpression::Action(action) => unparse_action(action, out),
    }
}

fn unparse_number(value: f64, out: &mut String) {
    if value.is_finite() && value.fract() == 0.0 && value.abs() <= i64::MAX as f64 {
        out.push_str(&(value as i64).to_string());
    } else {
        let formatted = format!("{value}");
        out.push_str(&formatted);
    }
}
