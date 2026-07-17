//! Classification of how an [`ExecutableAction`] awaits a simulation result.
//!
//! Prevents the runner from waiting on actions the sim maps to [`Wait`] (which never
//! produces an `action_result`). See [`crate::pending_action_protocol`].

use crate::ExecutableAction;
use crate::motion::is_zero_motion;

/// How the runner/sim should await completion for an emitted action.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ActionAwaitKind {
    /// No `action_result` is produced (zero motion, statement side effects, AwaitScanResult).
    None,
    /// One-cycle expression mine/dump; sim records a scalar result.
    Scalar,
    /// Chunked move/rotate; sim pending accumulates until finished.
    Motion,
    /// `StartScan`; sim writes scan_time in the CPU loop (not via physics Wait).
    ScanStart,
}

impl ActionAwaitKind {
    /// True when the program bridge should expect a numeric `action_result` from physics
    /// or from finishing pending motion (not scan-start, which is written synchronously).
    pub fn expects_physics_result(self) -> bool {
        matches!(self, Self::Scalar | Self::Motion)
    }
}

/// Classify how an action participates in the pending-result protocol.
pub fn await_kind(action: ExecutableAction) -> ActionAwaitKind {
    match action {
        ExecutableAction::Move(distance) if is_zero_motion(distance) => ActionAwaitKind::None,
        ExecutableAction::Rotate(rotation) if is_zero_motion(rotation) => ActionAwaitKind::None,
        ExecutableAction::Move(_) | ExecutableAction::Rotate(_) => ActionAwaitKind::Motion,
        ExecutableAction::Mine | ExecutableAction::Dump(_) => ActionAwaitKind::Scalar,
        ExecutableAction::StartScan(_) => ActionAwaitKind::ScanStart,
        ExecutableAction::AwaitScanResult => ActionAwaitKind::None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn await_kind_classifies_zero_motion_as_none() {
        assert_eq!(
            await_kind(ExecutableAction::Move(0.0)),
            ActionAwaitKind::None
        );
        assert_eq!(
            await_kind(ExecutableAction::Rotate(crate::motion::MOTION_EPSILON)),
            ActionAwaitKind::None
        );
        assert!(!ActionAwaitKind::None.expects_physics_result());
    }

    #[test]
    fn await_kind_classifies_chunked_motion_and_scalars() {
        assert_eq!(
            await_kind(ExecutableAction::Move(1.0)),
            ActionAwaitKind::Motion
        );
        assert_eq!(await_kind(ExecutableAction::Mine), ActionAwaitKind::Scalar);
        assert_eq!(
            await_kind(ExecutableAction::Dump(2)),
            ActionAwaitKind::Scalar
        );
        assert_eq!(
            await_kind(ExecutableAction::StartScan(0.0)),
            ActionAwaitKind::ScanStart
        );
        assert!(ActionAwaitKind::Motion.expects_physics_result());
        assert!(ActionAwaitKind::Scalar.expects_physics_result());
        assert!(!ActionAwaitKind::ScanStart.expects_physics_result());
    }
}
