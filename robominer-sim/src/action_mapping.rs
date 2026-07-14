//! Maps program [`ExecutableAction`] values to simulation [`RobotAction`] values.
//!
//! Shared by static program expansion (`RepeatingActions`) and the runtime program
//! bridge (`program_bridge`). See [`robominer_program::pending_action_protocol`].

use robominer_program::ExecutableAction;
use robominer_program::motion::{expand_motion_steps, motion_chunk, record_motion_step};

use crate::robot::{RobotAction, RobotSpec};

pub(crate) fn move_speed(spec: &RobotSpec, signed_distance: f64) -> f64 {
    if signed_distance >= 0.0 {
        spec.forward_speed
    } else {
        spec.backward_speed
    }
}

pub(crate) fn rotate_speed(spec: &RobotSpec) -> f64 {
    spec.rotate_speed as f64
}

fn chunk_motion(
    amount: f64,
    speed: f64,
    action: impl Fn(f64) -> RobotAction,
) -> Option<RobotAction> {
    motion_chunk(amount, speed).map(|(step, _)| action(step))
}

fn motion_chunk_robot_action(signed_amount: f64, spec: &RobotSpec, rotate: bool) -> RobotAction {
    if rotate {
        chunk_motion(signed_amount, rotate_speed(spec), RobotAction::Rotate)
    } else {
        chunk_motion(
            signed_amount,
            move_speed(spec, signed_amount),
            RobotAction::Move,
        )
    }
    .unwrap_or(RobotAction::Wait)
}

fn non_motion_robot_action(action: ExecutableAction) -> Option<RobotAction> {
    match action {
        ExecutableAction::Mine => Some(RobotAction::Mine),
        ExecutableAction::Dump(ore_type) if ore_type > 0 => {
            Some(RobotAction::DumpOre((ore_type - 1) as usize))
        }
        ExecutableAction::Dump(_) => Some(RobotAction::DumpAll),
        ExecutableAction::StartScan(_) | ExecutableAction::AwaitScanResult => {
            Some(RobotAction::Wait)
        }
        ExecutableAction::Move(_) | ExecutableAction::Rotate(_) => None,
    }
}

/// First speed-limited chunk for a single simulation cycle.
pub(crate) fn robot_action_from_executable(
    action: ExecutableAction,
    spec: &RobotSpec,
) -> RobotAction {
    match action {
        ExecutableAction::Move(distance) => {
            motion_chunk_robot_action(distance, spec, false)
        }
        ExecutableAction::Rotate(rotation) => motion_chunk_robot_action(rotation, spec, true),
        other => non_motion_robot_action(other).unwrap_or(RobotAction::Wait),
    }
}

fn extend_chunked_motion(
    result: &mut Vec<RobotAction>,
    amount: f64,
    speed: f64,
    action: impl Fn(f64) -> RobotAction,
) {
    let steps = expand_motion_steps(amount, speed);
    if steps.is_empty() {
        result.push(RobotAction::Wait);
        return;
    }

    for step in steps {
        result.push(action(step));
    }
}

/// Expand a static executable program into per-cycle robot actions.
pub(crate) fn expand_executable_actions(
    spec: &RobotSpec,
    actions: &[ExecutableAction],
) -> Vec<RobotAction> {
    let mut result = Vec::new();

    for action in actions {
        match *action {
            ExecutableAction::Move(distance) => {
                extend_chunked_motion(
                    &mut result,
                    distance,
                    move_speed(spec, distance),
                    RobotAction::Move,
                );
            }
            ExecutableAction::Rotate(rotation) => {
                extend_chunked_motion(
                    &mut result,
                    rotation,
                    rotate_speed(spec),
                    RobotAction::Rotate,
                );
            }
            ExecutableAction::Mine => result.push(RobotAction::Mine),
            ExecutableAction::Dump(ore_type) if ore_type > 0 => {
                result.push(RobotAction::DumpOre((ore_type - 1) as usize));
            }
            ExecutableAction::Dump(_) => result.push(RobotAction::DumpAll),
            ExecutableAction::StartScan(_) | ExecutableAction::AwaitScanResult => {}
        }
    }

    result
}

/// Per-cycle chunking state for move/rotate actions emitted by the program runner.
#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) enum PendingExpressionAction {
    Move { remaining: f64, accumulated: f64 },
    Rotate { remaining: f64, accumulated: f64 },
}

impl PendingExpressionAction {
    pub(crate) fn next_robot_action(&self, spec: &RobotSpec) -> RobotAction {
        match *self {
            PendingExpressionAction::Move { remaining, .. } => {
                motion_chunk_robot_action(remaining, spec, false)
            }
            PendingExpressionAction::Rotate { remaining, .. } => {
                motion_chunk_robot_action(remaining, spec, true)
            }
        }
    }

    pub(crate) fn record_step(&mut self, value: f64, spec: &RobotSpec) -> bool {
        match self {
            PendingExpressionAction::Move {
                remaining,
                accumulated,
            } => record_motion_step(
                remaining,
                accumulated,
                value,
                move_speed(spec, *remaining),
            )
            .is_finished(),
            PendingExpressionAction::Rotate {
                remaining,
                accumulated,
            } => record_motion_step(
                remaining,
                accumulated,
                value,
                rotate_speed(spec),
            )
            .is_finished(),
        }
    }

    pub(crate) fn accumulated(&self) -> f64 {
        match *self {
            PendingExpressionAction::Move { accumulated, .. }
            | PendingExpressionAction::Rotate { accumulated, .. } => accumulated,
        }
    }
}

/// Map a runner action that awaits a multi-cycle result into pending state when needed.
pub(crate) fn map_awaiting_executable(
    action: ExecutableAction,
    spec: &RobotSpec,
) -> (Option<PendingExpressionAction>, RobotAction) {
    match action {
        ExecutableAction::Move(amount) if amount != 0.0 => {
            let pending = PendingExpressionAction::Move {
                remaining: amount,
                accumulated: 0.0,
            };
            (Some(pending), pending.next_robot_action(spec))
        }
        ExecutableAction::Rotate(amount) if amount != 0.0 => {
            let pending = PendingExpressionAction::Rotate {
                remaining: amount,
                accumulated: 0.0,
            };
            (Some(pending), pending.next_robot_action(spec))
        }
        _ => (None, robot_action_from_executable(action, spec)),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn test_spec() -> RobotSpec {
        RobotSpec::test_robot()
    }

    #[test]
    fn expand_executable_actions_matches_first_chunk_mapping() {
        let spec = test_spec();
        let actions = [
            ExecutableAction::Move(2.5),
            ExecutableAction::Rotate(180.0),
            ExecutableAction::Mine,
        ];

        let expanded = expand_executable_actions(&spec, &actions);
        assert_eq!(
            expanded[0],
            robot_action_from_executable(ExecutableAction::Move(2.5), &spec)
        );
        assert_eq!(expanded.len(), 6);
        assert_eq!(
            expanded[3],
            robot_action_from_executable(ExecutableAction::Rotate(180.0), &spec)
        );
        assert_eq!(expanded.last().copied(), Some(RobotAction::Mine));
    }

    #[test]
    fn map_awaiting_executable_starts_pending_move() {
        let spec = test_spec();
        let (pending, robot_action) =
            map_awaiting_executable(ExecutableAction::Move(2.0), &spec);

        assert!(pending.is_some());
        assert_eq!(robot_action, RobotAction::Move(1.0));
    }

    #[test]
    fn map_awaiting_executable_passes_through_immediate_actions() {
        let spec = test_spec();
        let (pending, robot_action) =
            map_awaiting_executable(ExecutableAction::Mine, &spec);

        assert!(pending.is_none());
        assert_eq!(robot_action, RobotAction::Mine);
    }

    #[test]
    fn record_step_continues_multi_chunk_move() {
        let mut pending = PendingExpressionAction::Move {
            remaining: 2.0,
            accumulated: 0.0,
        };
        let spec = test_spec();

        assert!(!pending.record_step(1.0, &spec));
        assert_eq!(pending.accumulated(), 1.0);

        assert!(pending.record_step(1.0, &spec));
        assert_eq!(pending.accumulated(), 2.0);
    }

    #[test]
    fn record_step_blocked_on_zero_travel() {
        let mut pending = PendingExpressionAction::Move {
            remaining: 2.0,
            accumulated: 0.0,
        };

        assert!(pending.record_step(0.0, &test_spec()));
        assert_eq!(pending.accumulated(), 0.0);
    }
}
