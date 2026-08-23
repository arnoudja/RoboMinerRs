//! Unified runner-side state for multi-cycle move/rotate actions.
//!
//! All three initiation paths (statement, dynamic statement, and expression) call
//! [`PendingProgramMotion::start`] with the appropriate [`ProgramMotionCompletion`]
//! and resume through [`PendingProgramMotion::continue_action`].
//!
//! See also [`crate::pending_action_protocol`].

use crate::ExecutableAction;
use crate::motion::is_zero_motion;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum ProgramMotionCompletion {
    Statement,
    Expression,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub(crate) struct PendingProgramMotion {
    pub action: ExecutableAction,
    pub completion: ProgramMotionCompletion,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub(crate) enum ContinueProgramMotion {
    NotActive,
    Reemit,
    /// Statement move/rotate finished; value is travel distance / rotation amount for debug `r`.
    StatementComplete(f64),
    ExpressionComplete(f64),
}

impl PendingProgramMotion {
    pub(crate) fn is_chunked(action: ExecutableAction) -> bool {
        match action {
            ExecutableAction::Move(distance) => !is_zero_motion(distance),
            ExecutableAction::Rotate(rotation) => !is_zero_motion(rotation),
            _ => false,
        }
    }

    pub(crate) fn start(action: ExecutableAction, completion: ProgramMotionCompletion) -> Self {
        debug_assert!(
            Self::is_chunked(action),
            "pending_program_motion requires chunked move/rotate, got {action:?}"
        );
        Self { action, completion }
    }

    pub(crate) fn continue_action(
        pending: &mut Option<Self>,
        action_result: &mut Option<f64>,
    ) -> ContinueProgramMotion {
        let Some(_current) = pending.as_ref() else {
            return ContinueProgramMotion::NotActive;
        };

        let Some(value) = action_result.take() else {
            return ContinueProgramMotion::Reemit;
        };

        let completed = match pending.take() {
            Some(completed) => completed,
            None => return ContinueProgramMotion::NotActive,
        };
        match completed.completion {
            ProgramMotionCompletion::Statement => ContinueProgramMotion::StatementComplete(value),
            ProgramMotionCompletion::Expression => ContinueProgramMotion::ExpressionComplete(value),
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
            PendingProgramMotion::continue_action(&mut pending, &mut action_result),
            ContinueProgramMotion::NotActive
        );
        assert_eq!(action_result, Some(1.0));
    }

    #[test]
    fn continue_action_reemits_when_action_result_missing() {
        let mut pending = Some(PendingProgramMotion::start(
            ExecutableAction::Move(2.0),
            ProgramMotionCompletion::Statement,
        ));
        let mut action_result = None;

        assert_eq!(
            PendingProgramMotion::continue_action(&mut pending, &mut action_result),
            ContinueProgramMotion::Reemit
        );
        assert!(pending.is_some());
        assert_eq!(pending.unwrap().action, ExecutableAction::Move(2.0));
    }

    #[test]
    fn continue_action_statement_complete_clears_pending_and_returns_value() {
        let mut pending = Some(PendingProgramMotion::start(
            ExecutableAction::Move(2.0),
            ProgramMotionCompletion::Statement,
        ));
        let mut action_result = Some(1.5);

        assert_eq!(
            PendingProgramMotion::continue_action(&mut pending, &mut action_result),
            ContinueProgramMotion::StatementComplete(1.5)
        );
        assert!(pending.is_none());
        assert_eq!(action_result, None);
    }

    #[test]
    fn continue_action_expression_complete_returns_accumulated_value() {
        let mut pending = Some(PendingProgramMotion::start(
            ExecutableAction::Move(2.0),
            ProgramMotionCompletion::Expression,
        ));
        let mut action_result = Some(1.25);

        assert_eq!(
            PendingProgramMotion::continue_action(&mut pending, &mut action_result),
            ContinueProgramMotion::ExpressionComplete(1.25)
        );
        assert!(pending.is_none());
        assert_eq!(action_result, None);
    }

    #[test]
    fn is_chunked_only_for_non_zero_move_and_rotate() {
        assert!(PendingProgramMotion::is_chunked(ExecutableAction::Move(
            1.0
        )));
        assert!(PendingProgramMotion::is_chunked(ExecutableAction::Rotate(
            -90.0
        )));
        assert!(!PendingProgramMotion::is_chunked(ExecutableAction::Move(
            0.0
        )));
        assert!(!PendingProgramMotion::is_chunked(ExecutableAction::Move(
            crate::motion::MOTION_EPSILON
        )));
        assert!(!PendingProgramMotion::is_chunked(ExecutableAction::Rotate(
            -crate::motion::MOTION_EPSILON / 2.0
        )));
        assert!(!PendingProgramMotion::is_chunked(ExecutableAction::Mine));
    }
}
