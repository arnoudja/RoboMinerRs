use crate::types::{ExecutableAction, ExecutableActionExpression, ExecutableExpression};

impl ExecutableExpression {
    pub(crate) fn literal_number(&self) -> Option<f64> {
        match self {
            ExecutableExpression::Number(value) => Some(*value),
            _ => None,
        }
    }

    pub(crate) fn first_action(&self) -> Option<ExecutableAction> {
        match self {
            ExecutableExpression::Action(action) => Some(*action),
            ExecutableExpression::Move(value) => value
                .literal_number()
                .map(ExecutableAction::Move)
                .or_else(|| value.first_action()),
            ExecutableExpression::Rotate(value) => value
                .literal_number()
                .map(ExecutableAction::Rotate)
                .or_else(|| value.first_action()),
            ExecutableExpression::Dump(value) => value
                .literal_number()
                .map(|value| ExecutableAction::Dump(value as i32))
                .or_else(|| value.first_action()),
            ExecutableExpression::UnaryNot(value) => value.first_action(),
            ExecutableExpression::Binary { left, right, .. } => {
                left.first_action().or_else(|| right.first_action())
            }
            ExecutableExpression::Ore(value) => value.first_action(),
            ExecutableExpression::Scan(direction) => {
                direction.as_ref().and_then(|value| value.first_action())
            }
            ExecutableExpression::OreDistance
            | ExecutableExpression::OreType
            | ExecutableExpression::RobotProperty(_) => None,
            ExecutableExpression::Number(_)
            | ExecutableExpression::Variable(_)
            | ExecutableExpression::VariableUpdate { .. }
            | ExecutableExpression::Time => None,
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
