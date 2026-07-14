//! Shared move/rotate chunking used by the program runner and simulator.
//!
//! See [`crate::pending_action_protocol`] for how chunk completion feeds back
//! into multi-cycle execution.

/// Distance/angle comparisons and chunk remainders smaller than this are zero.
pub const MOTION_EPSILON: f64 = 0.000_001;

/// Result of applying one cycle's worth of travel to a pending move/rotate.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MotionStepOutcome {
    /// The chunk finished normally and more signed distance remains.
    Continue,
    /// The final chunk completed; no distance remains.
    Complete,
    /// Travel fell short of the requested chunk (for example a wall); end the action.
    Blocked,
}

impl MotionStepOutcome {
    pub fn is_finished(self) -> bool {
        matches!(self, Self::Complete | Self::Blocked)
    }
}

/// One speed-limited chunk from a signed distance/angle request.
///
/// Returns `(signed_step, signed_remaining)`.
pub fn motion_chunk(signed_amount: f64, speed: f64) -> Option<(f64, f64)> {
    if signed_amount == 0.0 || speed <= 0.0 {
        None
    } else {
        let step = signed_amount.signum() * signed_amount.abs().min(speed);
        Some((step, signed_amount - step))
    }
}

/// Expand a signed request into per-cycle chunks at the given speed.
pub fn expand_motion_steps(mut signed_amount: f64, speed: f64) -> Vec<f64> {
    let mut steps = Vec::new();

    while signed_amount.abs() > MOTION_EPSILON {
        let Some((step, remaining)) = motion_chunk(signed_amount, speed) else {
            break;
        };
        steps.push(step);
        signed_amount = remaining;
    }

    steps
}

/// Record actual travel against pending move/rotate state.
pub fn record_motion_step(
    remaining: &mut f64,
    accumulated: &mut f64,
    traveled: f64,
    speed: f64,
) -> MotionStepOutcome {
    let expected = motion_chunk(*remaining, speed)
        .map(|(step, _)| step)
        .unwrap_or(0.0);

    *accumulated += traveled;
    *remaining -= traveled;

    if traveled.abs() + MOTION_EPSILON < expected.abs() {
        MotionStepOutcome::Blocked
    } else if remaining.abs() <= MOTION_EPSILON {
        MotionStepOutcome::Complete
    } else {
        MotionStepOutcome::Continue
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn motion_chunk_splits_signed_distance_by_speed() {
        assert_eq!(motion_chunk(2.5, 1.0), Some((1.0, 1.5)));
        assert_eq!(motion_chunk(-2.5, 1.0), Some((-1.0, -1.5)));
        assert_eq!(motion_chunk(1.0, 1.5), Some((1.0, 0.0)));
        assert_eq!(motion_chunk(0.0, 1.0), None);
        assert_eq!(motion_chunk(1.0, 0.0), None);
    }

    #[test]
    fn expand_motion_steps_covers_full_request() {
        assert_eq!(expand_motion_steps(2.5, 1.0), vec![1.0, 1.0, 0.5]);
        assert_eq!(expand_motion_steps(-2.0, 1.0), vec![-1.0, -1.0]);
        assert!(expand_motion_steps(0.0, 1.0).is_empty());
    }

    #[test]
    fn record_motion_step_continues_until_remaining_is_zero() {
        let mut remaining = 2.0;
        let mut accumulated = 0.0;

        assert_eq!(
            record_motion_step(&mut remaining, &mut accumulated, 1.0, 1.0),
            MotionStepOutcome::Continue
        );
        assert_eq!(remaining, 1.0);
        assert_eq!(accumulated, 1.0);

        assert_eq!(
            record_motion_step(&mut remaining, &mut accumulated, 1.0, 1.0),
            MotionStepOutcome::Complete
        );
        assert_eq!(remaining, 0.0);
        assert_eq!(accumulated, 2.0);
    }

    #[test]
    fn record_motion_step_blocked_when_travel_is_zero() {
        let mut remaining = 2.0;
        let mut accumulated = 0.0;

        assert_eq!(
            record_motion_step(&mut remaining, &mut accumulated, 0.0, 1.0),
            MotionStepOutcome::Blocked
        );
        assert_eq!(accumulated, 0.0);
        assert_eq!(remaining, 2.0);
    }

    #[test]
    fn record_motion_step_blocked_when_travel_is_partial() {
        let mut remaining = 2.0;
        let mut accumulated = 0.0;

        assert_eq!(
            record_motion_step(&mut remaining, &mut accumulated, 0.4, 1.0),
            MotionStepOutcome::Blocked
        );
        assert_eq!(accumulated, 0.4);
        assert_eq!(remaining, 1.6);
    }
}
