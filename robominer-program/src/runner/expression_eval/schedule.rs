use crate::types::{
    AreaProperty, ExecutableAction, ExecutableExpression, ExecutableExpressionKind, Operator,
    RobotProperty, SourceSpan, VariableOperator,
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
    PushInt(i64),
    PushFloat(f64),
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
    PushAreaProperty(AreaProperty),
    PushDynamicMove,
    PushDynamicRotate,
    PushDynamicDump,
    PushAction(ExecutableAction),
    ApplyUnaryNot,
    ApplyUnaryMinus,
    ApplyAbs,
    ApplySqrt,
    ApplySin,
    ApplyCos,
    ApplyTan,
    ApplyMin,
    ApplyMax,
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
        ExecutableExpressionKind::Int(value) => {
            push(work, ExpressionWork::PushInt(*value));
        }
        ExecutableExpressionKind::Float(value) => {
            push(work, ExpressionWork::PushFloat(*value));
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
        ExecutableExpressionKind::Abs(value) => {
            schedule_expression(work, value);
            push(work, ExpressionWork::ApplyAbs);
        }
        ExecutableExpressionKind::Sqrt(value) => {
            schedule_expression(work, value);
            push(work, ExpressionWork::ApplySqrt);
        }
        ExecutableExpressionKind::Sin(value) => {
            schedule_expression(work, value);
            push(work, ExpressionWork::ApplySin);
        }
        ExecutableExpressionKind::Cos(value) => {
            schedule_expression(work, value);
            push(work, ExpressionWork::ApplyCos);
        }
        ExecutableExpressionKind::Tan(value) => {
            schedule_expression(work, value);
            push(work, ExpressionWork::ApplyTan);
        }
        ExecutableExpressionKind::Min(left, right) => {
            schedule_expression(work, left);
            schedule_expression(work, right);
            push(work, ExpressionWork::ApplyMin);
        }
        ExecutableExpressionKind::Max(left, right) => {
            schedule_expression(work, left);
            schedule_expression(work, right);
            push(work, ExpressionWork::ApplyMax);
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
        ExecutableExpressionKind::AreaProperty(property) => {
            push(work, ExpressionWork::PushAreaProperty(*property));
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
        ExecutableExpressionKind::Call { args, .. } => {
            for arg in args {
                schedule_expression(work, arg);
            }
            push(work, ExpressionWork::PushInt(0));
        }
    }
}
