use crate::types::{
    ExecutableAction, ExecutableExpression, Operator, RobotProperty, VariableOperator,
};

#[derive(Debug, Clone, PartialEq)]
pub(crate) enum ExpressionWork {
    PushNumber(f64),
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
    ApplyBinary(Operator),
}

pub(crate) fn schedule_expression(
    work: &mut Vec<ExpressionWork>,
    expression: &ExecutableExpression,
) {
    match expression {
        ExecutableExpression::Number(value) => {
            work.push(ExpressionWork::PushNumber(*value));
        }
        ExecutableExpression::Variable(name) => {
            work.push(ExpressionWork::PushVariable(name.clone()));
        }
        ExecutableExpression::VariableUpdate { name, operator } => {
            work.push(ExpressionWork::PushVariableUpdate {
                name: name.clone(),
                operator: *operator,
            });
        }
        ExecutableExpression::UnaryNot(value) => {
            schedule_expression(work, value);
            work.push(ExpressionWork::ApplyUnaryNot);
        }
        ExecutableExpression::Binary {
            operator,
            left,
            right,
        } => {
            schedule_expression(work, left);
            schedule_expression(work, right);
            work.push(ExpressionWork::ApplyBinary(*operator));
        }
        ExecutableExpression::Time => {
            work.push(ExpressionWork::PushTime);
        }
        ExecutableExpression::Ore(ore_type) => {
            schedule_expression(work, ore_type);
            work.push(ExpressionWork::PushOre);
        }
        ExecutableExpression::Scan(direction) => {
            if let Some(direction) = direction {
                schedule_expression(work, direction);
            }
            work.push(ExpressionWork::PushStartScan);
        }
        ExecutableExpression::OreDistance => {
            work.push(ExpressionWork::PushOreDistance);
        }
        ExecutableExpression::OreType => {
            work.push(ExpressionWork::PushOreType);
        }
        ExecutableExpression::RobotProperty(property) => {
            work.push(ExpressionWork::PushRobotProperty(*property));
        }
        ExecutableExpression::Move(arg) => {
            schedule_expression(work, arg);
            work.push(ExpressionWork::PushDynamicMove);
        }
        ExecutableExpression::Rotate(arg) => {
            schedule_expression(work, arg);
            work.push(ExpressionWork::PushDynamicRotate);
        }
        ExecutableExpression::Dump(arg) => {
            schedule_expression(work, arg);
            work.push(ExpressionWork::PushDynamicDump);
        }
        ExecutableExpression::Action(action) => {
            work.push(ExpressionWork::PushAction(*action));
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
