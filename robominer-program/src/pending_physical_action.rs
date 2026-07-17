//! Unified runner-side state for multi-cycle move/rotate actions.
//!
//! All three initiation paths (statement, dynamic statement, and expression) call
//! [`PendingPhysicalAction::start`] with the appropriate [`PhysicalCompletion`]
//! and resume through [`PendingPhysicalAction::continue_action`].
//!
//! See also [`crate::pending_action_protocol`].

use crate::ExecutableAction;
use crate::motion::is_zero_motion;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum PhysicalCompletion {
    Statement,
    Expression,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub(crate) struct PendingPhysicalAction {
    pub action: ExecutableAction,
    pub completion: PhysicalCompletion,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub(crate) enum ContinuePhysicalAction {
    NotActive,
    Reemit,
    StatementComplete,
    ExpressionComplete(f64),
}

impl PendingPhysicalAction {
    pub(crate) fn is_chunked(action: ExecutableAction) -> bool {
        match action {
            ExecutableAction::Move(distance) => !is_zero_motion(distance),
            ExecutableAction::Rotate(rotation) => !is_zero_motion(rotation),
            _ => false,
        }
    }

    pub(crate) fn start(action: ExecutableAction, completion: PhysicalCompletion) -> Self {
        debug_assert!(
            Self::is_chunked(action),
            "pending physical requires chunked move/rotate, got {action:?}"
        );
        Self { action, completion }
    }

    pub(crate) fn continue_action(
        pending: &mut Option<Self>,
        action_result: &mut Option<f64>,
    ) -> ContinuePhysicalAction {
        let Some(_current) = pending.as_ref() else {
            return ContinuePhysicalAction::NotActive;
        };

        let Some(value) = action_result.take() else {
            return ContinuePhysicalAction::Reemit;
        };

        let completed = pending.take().expect("pending physical action");
        match completed.completion {
            PhysicalCompletion::Statement => {
                let _ = value;
                ContinuePhysicalAction::StatementComplete
            }
            PhysicalCompletion::Expression => ContinuePhysicalAction::ExpressionComplete(value),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn continue_action_is_not_active_without_pending() {
        let mut pending = None;
        let mut action_result = Some(1.0);

        assert_eq!(
            PendingPhysicalAction::continue_action(&mut pending, &mut action_result),
            ContinuePhysicalAction::NotActive
        );
        assert_eq!(action_result, Some(1.0));
    }

    #[test]
    fn continue_action_reemits_when_action_result_missing() {
        let mut pending = Some(PendingPhysicalAction::start(
            ExecutableAction::Move(2.0),
            PhysicalCompletion::Statement,
        ));
        let mut action_result = None;

        assert_eq!(
            PendingPhysicalAction::continue_action(&mut pending, &mut action_result),
            ContinuePhysicalAction::Reemit
        );
        assert!(pending.is_some());
        assert_eq!(pending.unwrap().action, ExecutableAction::Move(2.0));
    }

    #[test]
    fn continue_action_statement_complete_clears_pending_and_discards_value() {
        let mut pending = Some(PendingPhysicalAction::start(
            ExecutableAction::Move(2.0),
            PhysicalCompletion::Statement,
        ));
        let mut action_result = Some(0.0);

        assert_eq!(
            PendingPhysicalAction::continue_action(&mut pending, &mut action_result),
            ContinuePhysicalAction::StatementComplete
        );
        assert!(pending.is_none());
        assert_eq!(action_result, None);
    }

    #[test]
    fn continue_action_expression_complete_returns_accumulated_value() {
        let mut pending = Some(PendingPhysicalAction::start(
            ExecutableAction::Move(2.0),
            PhysicalCompletion::Expression,
        ));
        let mut action_result = Some(1.25);

        assert_eq!(
            PendingPhysicalAction::continue_action(&mut pending, &mut action_result),
            ContinuePhysicalAction::ExpressionComplete(1.25)
        );
        assert!(pending.is_none());
        assert_eq!(action_result, None);
    }

    #[test]
    fn is_chunked_only_for_non_zero_move_and_rotate() {
        assert!(PendingPhysicalAction::is_chunked(ExecutableAction::Move(
            1.0
        )));
        assert!(PendingPhysicalAction::is_chunked(ExecutableAction::Rotate(
            -90.0
        )));
        assert!(!PendingPhysicalAction::is_chunked(ExecutableAction::Move(
            0.0
        )));
        assert!(!PendingPhysicalAction::is_chunked(ExecutableAction::Move(
            crate::motion::MOTION_EPSILON
        )));
        assert!(!PendingPhysicalAction::is_chunked(
            ExecutableAction::Rotate(-crate::motion::MOTION_EPSILON / 2.0)
        ));
        assert!(!PendingPhysicalAction::is_chunked(ExecutableAction::Mine));
    }
}
