use crate::types::{
    ExecutableAction, ExecutableExpression, ExecutableExpressionKind, Operator, RobotProperty,
    SourceSpan, VariableOperator,
};

/// One CPU step of expression evaluation, tagged with the source it came from so the
/// rally replay can highlight the sub-expression being evaluated.
#[derive(Debug, Clone, PartialEq)]
pub(crate) struct ExpressionWorkItem {
    pub span: SourceSpan,
    pub kind: ExpressionWork,
}

#[derive(Debug, Clone, PartialEq)]
pub(crate) enum ExpressionWork {
    PushNumber(f64),
    PushBool(bool),
    PushVariable(String),
    PushVariableUpdate {
        name: String,
        operator: VariableOperator,
    },
    PushTime,
    PushOre,
    PushStartScan,
    PushOreDistance,
    PushOreType,
    PushRobotProperty(RobotProperty),
    PushDynamicMove,
    PushDynamicRotate,
    PushDynamicDump,
    PushAction(ExecutableAction),
    ApplyUnaryNot,
    ApplyUnaryMinus,
    ApplyBinary(Operator),
}

pub(crate) fn schedule_expression(
    work: &mut Vec<ExpressionWorkItem>,
    expression: &ExecutableExpression,
) {
    let span = expression.span;
    let push = |work: &mut Vec<ExpressionWorkItem>, kind| {
        work.push(ExpressionWorkItem { span, kind });
    };

    match &expression.kind {
        ExecutableExpressionKind::Number(value) => {
            push(work, ExpressionWork::PushNumber(*value));
        }
        ExecutableExpressionKind::Bool(value) => {
            push(work, ExpressionWork::PushBool(*value));
        }
        ExecutableExpressionKind::Variable(name) => {
            push(work, ExpressionWork::PushVariable(name.clone()));
        }
        ExecutableExpressionKind::VariableUpdate { name, operator } => {
            push(
                work,
                ExpressionWork::PushVariableUpdate {
                    name: name.clone(),
                    operator: *operator,
                },
            );
        }
        ExecutableExpressionKind::UnaryNot(value) => {
            schedule_expression(work, value);
            push(work, ExpressionWork::ApplyUnaryNot);
        }
        ExecutableExpressionKind::UnaryMinus(value) => {
            schedule_expression(work, value);
            push(work, ExpressionWork::ApplyUnaryMinus);
        }
        ExecutableExpressionKind::Binary {
            operator,
            left,
            right,
        } => {
            schedule_expression(work, left);
            schedule_expression(work, right);
            push(work, ExpressionWork::ApplyBinary(*operator));
        }
        ExecutableExpressionKind::Time => {
            push(work, ExpressionWork::PushTime);
        }
        // Deprecated: prefer RobotProperty::OreStored*.
        ExecutableExpressionKind::Ore(ore_type) => {
            schedule_expression(work, ore_type);
            push(work, ExpressionWork::PushOre);
        }
        ExecutableExpressionKind::Scan(direction) => {
            if let Some(direction) = direction {
                schedule_expression(work, direction);
            }
            push(work, ExpressionWork::PushStartScan);
        }
        ExecutableExpressionKind::OreDistance => {
            push(work, ExpressionWork::PushOreDistance);
        }
        ExecutableExpressionKind::OreType => {
            push(work, ExpressionWork::PushOreType);
        }
        ExecutableExpressionKind::RobotProperty(property) => {
            push(work, ExpressionWork::PushRobotProperty(*property));
        }
        ExecutableExpressionKind::Move(arg) => {
            schedule_expression(work, arg);
            push(work, ExpressionWork::PushDynamicMove);
        }
        ExecutableExpressionKind::Rotate(arg) => {
            schedule_expression(work, arg);
            push(work, ExpressionWork::PushDynamicRotate);
        }
        ExecutableExpressionKind::Dump(arg) => {
            schedule_expression(work, arg);
            push(work, ExpressionWork::PushDynamicDump);
        }
        ExecutableExpressionKind::Action(action) => {
            push(work, ExpressionWork::PushAction(*action));
        }
    }
}

pub(crate) trait Truthy {
    fn is_truthy(&self) -> bool;
}

impl Truthy for f64 {
    fn is_truthy(&self) -> bool {
        *self != 0.0
    }
}

pub(crate) fn evaluate_operator(operator: Operator, left: f64, right: f64) -> f64 {
    match operator {
        Operator::Addition => left + right,
        Operator::Subtraction => left - right,
        Operator::Multiply => left * right,
        Operator::Division => left / right,
        Operator::Mod => (left as i32 % right as i32) as f64,
        Operator::Larger => (left > right) as i32 as f64,
        Operator::Smaller => (left < right) as i32 as f64,
        Operator::LargerEqual => (left >= right) as i32 as f64,
        Operator::SmallerEqual => (left <= right) as i32 as f64,
        Operator::Equal => (left == right) as i32 as f64,
        Operator::NotEqual => (left != right) as i32 as f64,
        Operator::And => (left.is_truthy() && right.is_truthy()) as i32 as f64,
        Operator::Or => (left.is_truthy() || right.is_truthy()) as i32 as f64,
        Operator::Undefined => 0.0,
    }
}
