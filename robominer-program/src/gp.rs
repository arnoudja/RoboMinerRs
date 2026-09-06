use crate::ast_visit::{count_statements, set_statement_at, take_statement_at};
use crate::compile::compile_executable_source;
use crate::types::{
    CompileError, ExecutableAction, ExecutableExpression, ExecutableExpressionKind,
    ExecutableProgram, ExecutableStatement, ExecutableStatementKind, SourceSpan,
};
use crate::unparse::unparse_program;

/// Recompile through source so GP edits never leave invalid ASTs in the population.
pub fn recompile_program(program: &ExecutableProgram) -> Result<ExecutableProgram, CompileError> {
    compile_executable_source(&unparse_program(program))
}

/// Swap random statement subtrees between two programs, then recompile both.
pub fn crossover_programs(
    left: &ExecutableProgram,
    right: &ExecutableProgram,
    rng: &mut impl RngLike,
) -> Option<(ExecutableProgram, ExecutableProgram)> {
    let mut left_child = left.clone();
    let mut right_child = right.clone();
    let left_count = count_statements(&left_child.statements);
    let right_count = count_statements(&right_child.statements);
    if left_count == 0 || right_count == 0 {
        return None;
    }

    let left_index = rng.gen_range(0, left_count);
    let right_index = rng.gen_range(0, right_count);
    let left_stmt = take_statement_at(&mut left_child.statements, left_index)?;
    let right_stmt = take_statement_at(&mut right_child.statements, right_index)?;
    set_statement_at(&mut left_child.statements, left_index, right_stmt)?;
    set_statement_at(&mut right_child.statements, right_index, left_stmt)?;

    let left_ok = recompile_program(&left_child).ok()?;
    let right_ok = recompile_program(&right_child).ok()?;
    Some((left_ok, right_ok))
}

/// Apply a small random mutation and recompile; returns the original on failure.
pub fn mutate_program(program: &ExecutableProgram, rng: &mut impl RngLike) -> ExecutableProgram {
    let mut candidate = program.clone();
    let count = count_statements(&candidate.statements);
    if count == 0 {
        return program.clone();
    }

    match rng.gen_range(0, 4) {
        0 => {
            jitter_a_number(&mut candidate.statements, rng);
        }
        1 => {
            let index = rng.gen_range(0, count);
            let replacement = random_leaf_statement(rng);
            let _ = set_statement_at(&mut candidate.statements, index, replacement);
        }
        2 => {
            let index = rng.gen_range(0, count);
            if let Some(stmt) = take_statement_at(&mut candidate.statements, index) {
                let mut wrapped = stmt;
                wrap_in_while_mine(&mut wrapped);
                let _ = set_statement_at(&mut candidate.statements, index, wrapped);
            }
        }
        _ => {
            candidate.statements.push(random_leaf_statement(rng));
        }
    }

    recompile_program(&candidate).unwrap_or_else(|_| program.clone())
}

/// Minimal RNG trait so callers can use `rand` or a test double without a hard dependency.
pub trait RngLike {
    fn gen_range(&mut self, low: usize, high: usize) -> usize;
    fn gen_f64(&mut self) -> f64;
}

fn count_numbers(statements: &[ExecutableStatement]) -> usize {
    statements.iter().map(count_numbers_in_statement).sum()
}

fn count_numbers_in_statement(statement: &ExecutableStatement) -> usize {
    match &statement.kind {
        ExecutableStatementKind::Action(ExecutableAction::Move(_))
        | ExecutableStatementKind::Action(ExecutableAction::Rotate(_))
        | ExecutableStatementKind::Action(ExecutableAction::StartScan(_)) => 1,
        ExecutableStatementKind::Action(_) => 0,
        ExecutableStatementKind::DynamicAction(action) => match action {
            crate::types::ExecutableActionExpression::Move(expr)
            | crate::types::ExecutableActionExpression::Rotate(expr)
            | crate::types::ExecutableActionExpression::Dump(expr) => {
                count_numbers_in_expression(expr)
            }
        },
        ExecutableStatementKind::Sequence(statements) => count_numbers(statements),
        ExecutableStatementKind::Declare { value, .. } => {
            value.as_ref().map(count_numbers_in_expression).unwrap_or(0)
        }
        ExecutableStatementKind::Assign { value, .. }
        | ExecutableStatementKind::Expression(value) => count_numbers_in_expression(value),
        ExecutableStatementKind::If {
            condition,
            true_body,
            false_body,
        } => {
            count_numbers_in_expression(condition)
                + count_numbers_in_statement(true_body)
                + false_body
                    .as_ref()
                    .map(|body| count_numbers_in_statement(body))
                    .unwrap_or(0)
        }
        ExecutableStatementKind::While {
            condition, body, ..
        } => {
            count_numbers_in_expression(condition)
                + body
                    .as_ref()
                    .map(|body| count_numbers_in_statement(body))
                    .unwrap_or(0)
        }
        ExecutableStatementKind::Return(expr) => {
            expr.as_ref().map(count_numbers_in_expression).unwrap_or(0)
        }
    }
}

fn count_numbers_in_expression(expression: &ExecutableExpression) -> usize {
    match &expression.kind {
        ExecutableExpressionKind::Int(_) | ExecutableExpressionKind::Float(_) => 1,
        ExecutableExpressionKind::UnaryNot(inner)
        | ExecutableExpressionKind::UnaryMinus(inner)
        | ExecutableExpressionKind::Abs(inner)
        | ExecutableExpressionKind::Sqrt(inner)
        | ExecutableExpressionKind::Sin(inner)
        | ExecutableExpressionKind::Cos(inner)
        | ExecutableExpressionKind::Tan(inner)
        | ExecutableExpressionKind::Ore(inner)
        | ExecutableExpressionKind::Move(inner)
        | ExecutableExpressionKind::Rotate(inner)
        | ExecutableExpressionKind::Dump(inner) => count_numbers_in_expression(inner),
        ExecutableExpressionKind::Min(left, right)
        | ExecutableExpressionKind::Max(left, right)
        | ExecutableExpressionKind::Binary { left, right, .. } => {
            count_numbers_in_expression(left) + count_numbers_in_expression(right)
        }
        ExecutableExpressionKind::Scan(Some(inner)) => count_numbers_in_expression(inner),
        ExecutableExpressionKind::Action(ExecutableAction::Move(_))
        | ExecutableExpressionKind::Action(ExecutableAction::Rotate(_))
        | ExecutableExpressionKind::Action(ExecutableAction::StartScan(_)) => 1,
        ExecutableExpressionKind::Call { args, .. } => {
            args.iter().map(count_numbers_in_expression).sum()
        }
        _ => 0,
    }
}

fn jitter_a_number(statements: &mut [ExecutableStatement], rng: &mut impl RngLike) {
    let total = count_numbers(statements);
    if total == 0 {
        return;
    }
    let target = rng.gen_range(0, total);
    let mut counter = 0usize;
    apply_number_jitter(statements, &mut counter, target, rng);
}

fn apply_number_jitter(
    statements: &mut [ExecutableStatement],
    counter: &mut usize,
    target: usize,
    rng: &mut impl RngLike,
) -> bool {
    for statement in statements {
        if apply_number_jitter_in_statement(statement, counter, target, rng) {
            return true;
        }
    }
    false
}

fn apply_number_jitter_in_statement(
    statement: &mut ExecutableStatement,
    counter: &mut usize,
    target: usize,
    rng: &mut impl RngLike,
) -> bool {
    match &mut statement.kind {
        ExecutableStatementKind::Action(ExecutableAction::Move(v))
        | ExecutableStatementKind::Action(ExecutableAction::Rotate(v))
        | ExecutableStatementKind::Action(ExecutableAction::StartScan(v)) => {
            if *counter == target {
                *v = jitter_number(*v, rng);
                return true;
            }
            *counter += 1;
            false
        }
        ExecutableStatementKind::Action(_) => false,
        ExecutableStatementKind::DynamicAction(action) => match action {
            crate::types::ExecutableActionExpression::Move(expr)
            | crate::types::ExecutableActionExpression::Rotate(expr)
            | crate::types::ExecutableActionExpression::Dump(expr) => {
                apply_number_jitter_in_expression(expr, counter, target, rng)
            }
        },
        ExecutableStatementKind::Sequence(statements) => {
            apply_number_jitter(statements, counter, target, rng)
        }
        ExecutableStatementKind::Declare { value, .. } => value
            .as_mut()
            .is_some_and(|value| apply_number_jitter_in_expression(value, counter, target, rng)),
        ExecutableStatementKind::Assign { value, .. }
        | ExecutableStatementKind::Expression(value) => {
            apply_number_jitter_in_expression(value, counter, target, rng)
        }
        ExecutableStatementKind::If {
            condition,
            true_body,
            false_body,
        } => {
            apply_number_jitter_in_expression(condition, counter, target, rng)
                || apply_number_jitter_in_statement(true_body, counter, target, rng)
                || false_body.as_mut().is_some_and(|body| {
                    apply_number_jitter_in_statement(body, counter, target, rng)
                })
        }
        ExecutableStatementKind::While {
            condition, body, ..
        } => {
            apply_number_jitter_in_expression(condition, counter, target, rng)
                || body.as_mut().is_some_and(|body| {
                    apply_number_jitter_in_statement(body, counter, target, rng)
                })
        }
        ExecutableStatementKind::Return(expr) => expr
            .as_mut()
            .is_some_and(|expr| apply_number_jitter_in_expression(expr, counter, target, rng)),
    }
}

fn apply_number_jitter_in_expression(
    expression: &mut ExecutableExpression,
    counter: &mut usize,
    target: usize,
    rng: &mut impl RngLike,
) -> bool {
    match &mut expression.kind {
        ExecutableExpressionKind::Int(value) => {
            if *counter == target {
                *value = jitter_number(*value as f64, rng).trunc() as i64;
                true
            } else {
                *counter += 1;
                false
            }
        }
        ExecutableExpressionKind::Float(value) => {
            if *counter == target {
                *value = jitter_number(*value, rng);
                true
            } else {
                *counter += 1;
                false
            }
        }
        ExecutableExpressionKind::UnaryNot(inner)
        | ExecutableExpressionKind::UnaryMinus(inner)
        | ExecutableExpressionKind::Abs(inner)
        | ExecutableExpressionKind::Sqrt(inner)
        | ExecutableExpressionKind::Sin(inner)
        | ExecutableExpressionKind::Cos(inner)
        | ExecutableExpressionKind::Tan(inner)
        | ExecutableExpressionKind::Ore(inner)
        | ExecutableExpressionKind::Move(inner)
        | ExecutableExpressionKind::Rotate(inner)
        | ExecutableExpressionKind::Dump(inner) => {
            apply_number_jitter_in_expression(inner, counter, target, rng)
        }
        ExecutableExpressionKind::Min(left, right)
        | ExecutableExpressionKind::Max(left, right)
        | ExecutableExpressionKind::Binary { left, right, .. } => {
            apply_number_jitter_in_expression(left, counter, target, rng)
                || apply_number_jitter_in_expression(right, counter, target, rng)
        }
        ExecutableExpressionKind::Scan(Some(inner)) => {
            apply_number_jitter_in_expression(inner, counter, target, rng)
        }
        ExecutableExpressionKind::Action(ExecutableAction::Move(v))
        | ExecutableExpressionKind::Action(ExecutableAction::Rotate(v))
        | ExecutableExpressionKind::Action(ExecutableAction::StartScan(v)) => {
            if *counter == target {
                *v = jitter_number(*v, rng);
                true
            } else {
                *counter += 1;
                false
            }
        }
        ExecutableExpressionKind::Call { args, .. } => {
            for arg in args {
                if apply_number_jitter_in_expression(arg, counter, target, rng) {
                    return true;
                }
            }
            false
        }
        _ => false,
    }
}

fn jitter_number(value: f64, rng: &mut impl RngLike) -> f64 {
    let delta = (rng.gen_f64() - 0.5) * 2.0;
    let next = value + delta;
    if next.is_finite() {
        (next * 100.0).round() / 100.0
    } else {
        value
    }
}

fn random_leaf_statement(rng: &mut impl RngLike) -> ExecutableStatement {
    let kind = match rng.gen_range(0, 4) {
        0 => ExecutableStatementKind::Action(ExecutableAction::Mine),
        1 => ExecutableStatementKind::Action(ExecutableAction::Move(1.0)),
        2 => ExecutableStatementKind::Action(ExecutableAction::Rotate(90.0)),
        _ => ExecutableStatementKind::Action(ExecutableAction::Dump(0)),
    };
    ExecutableStatement::at(SourceSpan::line_only(1), kind)
}

fn wrap_in_while_mine(statement: &mut ExecutableStatement) {
    let body = statement.clone();
    *statement = ExecutableStatement::at(
        statement.source_span,
        ExecutableStatementKind::While {
            condition: ExecutableExpression::unspanned(ExecutableExpressionKind::Action(
                ExecutableAction::Mine,
            )),
            body: Some(Box::new(body)),
            is_do_while: false,
        },
    );
}

/// Seed templates used when building the initial GA population.
pub fn seed_program_sources() -> Vec<&'static str> {
    vec![
        "move(1); mine();",
        "move(1.5); while (mine());",
        "if (move(1.5) >= 1) { while (mine()); } else { move(-1); rotate(20); }",
        "scan(); while (oreType() == 0) { move(1); scan(); } while (mine());",
        "while (true) { if (mine()) { while (mine()); } else if (move(1.42) < 0.1) { rotate(160); } else { move(1.42); } }",
        "while (true) { if (robot.oreStored >= robot.oreCap) { dump(); } else if (mine()) { while (mine()); } else { scan(); move(1); } }",
    ]
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::compatibility_fixture_source;
    use crate::unparse::unparse_program;

    struct StepRng(u64);

    impl RngLike for StepRng {
        fn gen_range(&mut self, low: usize, high: usize) -> usize {
            if high <= low {
                return low;
            }
            let value = self.0 as usize;
            self.0 = self.0.wrapping_add(1);
            low + value % (high - low)
        }

        fn gen_f64(&mut self) -> f64 {
            let value = (self.0 % 1000) as f64 / 1000.0;
            self.0 = self.0.wrapping_add(1);
            value
        }
    }

    #[test]
    fn seed_program_sources_all_compile() {
        for source in seed_program_sources() {
            compile_executable_source(source).unwrap_or_else(|error| {
                panic!("seed program failed to compile ({error}): {source}")
            });
        }
    }

    #[test]
    fn mutate_program_returns_compiling_program() {
        let source = compatibility_fixture_source("seed_ai_2");
        let program = compile_executable_source(source).expect("fixture compiles");
        let mut rng = StepRng(7);
        let mutated = mutate_program(&program, &mut rng);
        let again = unparse_program(&mutated);
        compile_executable_source(&again).expect("mutated program should compile");
    }

    #[test]
    fn crossover_programs_returns_compiling_children() {
        let left = compile_executable_source(compatibility_fixture_source("seed_ai_1"))
            .expect("left compiles");
        let right = compile_executable_source(compatibility_fixture_source("seed_ai_2"))
            .expect("right compiles");
        let mut rng = StepRng(3);
        let (a, b) = crossover_programs(&left, &right, &mut rng).expect("crossover");
        compile_executable_source(&unparse_program(&a)).expect("child a");
        compile_executable_source(&unparse_program(&b)).expect("child b");
    }
}
