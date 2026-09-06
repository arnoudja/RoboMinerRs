use crate::types::{
    ExecutableAction, ExecutableActionExpression, ExecutableExpression, ExecutableExpressionKind,
};

impl ExecutableExpression {
    pub(crate) fn literal_number(&self) -> Option<f64> {
        match &self.kind {
            ExecutableExpressionKind::Int(value) => Some(*value as f64),
            ExecutableExpressionKind::Float(value) => Some(*value),
            _ => None,
        }
    }

    pub(crate) fn first_action(&self) -> Option<ExecutableAction> {
        match &self.kind {
            ExecutableExpressionKind::Action(action) => Some(*action),
            ExecutableExpressionKind::Move(value) => value
                .literal_number()
                .map(ExecutableAction::Move)
                .or_else(|| value.first_action()),
            ExecutableExpressionKind::Rotate(value) => value
                .literal_number()
                .map(ExecutableAction::Rotate)
                .or_else(|| value.first_action()),
            ExecutableExpressionKind::Dump(value) => value
                .literal_number()
                .map(|value| ExecutableAction::Dump(value as i32))
                .or_else(|| value.first_action()),
            ExecutableExpressionKind::UnaryNot(value)
            | ExecutableExpressionKind::UnaryMinus(value)
            | ExecutableExpressionKind::Abs(value)
            | ExecutableExpressionKind::Sqrt(value)
            | ExecutableExpressionKind::Sin(value)
            | ExecutableExpressionKind::Cos(value)
            | ExecutableExpressionKind::Tan(value) => value.first_action(),
            ExecutableExpressionKind::Min(left, right)
            | ExecutableExpressionKind::Max(left, right)
            | ExecutableExpressionKind::Binary { left, right, .. } => {
                left.first_action().or_else(|| right.first_action())
            }
            ExecutableExpressionKind::Ore(value) => value.first_action(),
            ExecutableExpressionKind::Scan(direction) => {
                direction.as_ref().and_then(|value| value.first_action())
            }
            ExecutableExpressionKind::OreDistance
            | ExecutableExpressionKind::OreType
            | ExecutableExpressionKind::RobotProperty(_)
            | ExecutableExpressionKind::AreaProperty(_) => None,
            ExecutableExpressionKind::Int(_)
            | ExecutableExpressionKind::Float(_)
            | ExecutableExpressionKind::Bool(_)
            | ExecutableExpressionKind::Variable(_)
            | ExecutableExpressionKind::VariableUpdate { .. }
            | ExecutableExpressionKind::Time => None,
            ExecutableExpressionKind::Call { .. } => None,
        }
    }
}

impl ExecutableActionExpression {
    pub(crate) fn static_action(&self) -> Option<ExecutableAction> {
        match self {
            ExecutableActionExpression::Move(value) => {
                value.literal_number().map(ExecutableAction::Move)
            }
            ExecutableActionExpression::Rotate(value) => {
                value.literal_number().map(ExecutableAction::Rotate)
            }
            ExecutableActionExpression::Dump(value) => value
                .literal_number()
                .map(|value| ExecutableAction::Dump(value as i32)),
        }
    }
}
