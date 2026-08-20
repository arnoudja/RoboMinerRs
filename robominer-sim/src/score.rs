use std::array;

use crate::MAX_ORE_TYPES;

/// Number of A/B/C scoring slots. Extra cargo types beyond this do not score.
pub const SCORE_TIER_COUNT: usize = 3;

/// One diminishing score tier (A / B / C).
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ScoreTierBreakdown {
    pub mined: i32,
    pub overflow_in: i32,
    pub counted: i32,
    pub cap: i32,
    pub overflow_out: i32,
    pub overflow_converted: i32,
    pub points: f64,
    pub max_points: f64,
}

/// How a haul produced its rally score.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ScoreBreakdown {
    pub ore_target: i32,
    pub high: ScoreTierBreakdown,
    pub mid: ScoreTierBreakdown,
    pub low: ScoreTierBreakdown,
    pub residual_ore: i32,
    pub residual_points: f64,
    pub total: f64,
}

/// Score haul using area `score_ore_target` (T). Thresholds scale with T; full tier
/// points stay 900 / 90 / 9 / …. When T=30 this matches the legacy hardcoded formula.
/// Values of T below 1 are treated as 1.
pub fn calculate_score(ore: [i32; MAX_ORE_TYPES], score_ore_target: i32) -> f64 {
    score_breakdown(ore, score_ore_target).total
}

pub fn score_breakdown(ore: [i32; MAX_ORE_TYPES], score_ore_target: i32) -> ScoreBreakdown {
    let t = score_ore_target.max(1);
    let mid_cap = 3 * t;
    let low_cap = 12 * t;
    let band = 3 * t;

    let high_mined = ore[0];
    let mid_mined = ore[1];
    let low_mined = ore[2];

    let high_counted = high_mined.min(t);
    let high_overflow_out = (high_mined - t).max(0);
    let high_overflow_converted = high_overflow_out * 2;
    let high_points = high_counted as f64 * (900.0 / f64::from(t));

    let mid_equivalent = mid_mined + high_overflow_converted;
    let mid_counted = mid_equivalent.min(mid_cap);
    let mid_overflow_out = (mid_equivalent - mid_cap).max(0);
    let mid_overflow_converted = mid_overflow_out * 2;
    let mid_points = mid_counted as f64 * (90.0 / f64::from(mid_cap));

    let low_equivalent = low_mined + mid_overflow_converted;
    let low_counted = low_equivalent.min(low_cap);
    let low_overflow_out = (low_equivalent - low_cap).max(0);
    let low_points = low_counted as f64 * (9.0 / f64::from(low_cap));

    let mut residual_left = low_overflow_out;
    let mut residual_points = 0.0;
    let mut factor = 0.01;
    while residual_left > 0 {
        residual_points += residual_left.min(band) as f64 * factor;
        residual_left -= band;
        factor /= 10.0;
    }

    ScoreBreakdown {
        ore_target: t,
        high: ScoreTierBreakdown {
            mined: high_mined,
            overflow_in: 0,
            counted: high_counted,
            cap: t,
            overflow_out: high_overflow_out,
            overflow_converted: high_overflow_converted,
            points: high_points,
            max_points: 900.0,
        },
        mid: ScoreTierBreakdown {
            mined: mid_mined,
            overflow_in: high_overflow_converted,
            counted: mid_counted,
            cap: mid_cap,
            overflow_out: mid_overflow_out,
            overflow_converted: mid_overflow_converted,
            points: mid_points,
            max_points: 90.0,
        },
        low: ScoreTierBreakdown {
            mined: low_mined,
            overflow_in: mid_overflow_converted,
            counted: low_counted,
            cap: low_cap,
            overflow_out: low_overflow_out,
            overflow_converted: 0,
            points: low_points,
            max_points: 9.0,
        },
        residual_ore: low_overflow_out,
        residual_points,
        total: high_points + mid_points + low_points + residual_points,
    }
}

pub fn ore_amounts(amounts: &[(usize, i32)]) -> [i32; MAX_ORE_TYPES] {
    let mut ore = array::from_fn(|_| 0);

    for (ore_type, amount) in amounts {
        ore[*ore_type] = *amount;
    }

    ore
}

#[cfg(test)]
mod tests {
    use super::{calculate_score, ore_amounts, score_breakdown};

    fn assert_close(actual: f64, expected: f64) {
        assert!(
            (actual - expected).abs() < 0.000_001,
            "expected {expected}, got {actual}"
        );
    }

    #[test]
    fn breakdown_matches_legacy_ore_tiers() {
        let ore = ore_amounts(&[(0, 35), (1, 100), (2, 500)]);
        let breakdown = score_breakdown(ore, 30);

        assert_eq!(breakdown.ore_target, 30);
        assert_eq!(breakdown.high.counted, 30);
        assert_eq!(breakdown.high.overflow_converted, 10);
        assert_close(breakdown.high.points, 900.0);
        assert_eq!(breakdown.mid.counted, 90);
        assert_eq!(breakdown.mid.overflow_converted, 40);
        assert_close(breakdown.mid.points, 90.0);
        assert_eq!(breakdown.low.counted, 360);
        assert_eq!(breakdown.residual_ore, 180);
        assert_close(breakdown.low.points, 9.0);
        assert_close(breakdown.residual_points, 0.99);
        assert_close(breakdown.total, 999.99);
        assert_close(calculate_score(ore, 30), breakdown.total);
    }

    #[test]
    fn breakdown_scales_with_ore_target() {
        let high_only = ore_amounts(&[(0, 15)]);
        let breakdown = score_breakdown(high_only, 15);

        assert_eq!(breakdown.high.counted, 15);
        assert_eq!(breakdown.high.overflow_out, 0);
        assert_close(breakdown.high.points, 900.0);
        assert_close(breakdown.total, 900.0);
        assert_close(calculate_score(high_only, 30), 450.0);
    }
}
