use std::array;

use crate::MAX_ORE_TYPES;

/// Full-tier points for A / B / C scoring bands. Thresholds scale with the area
/// ore target; these totals stay fixed (legacy T=30 formula).
pub const SCORE_HIGH_TIER_POINTS: f64 = 900.0;
pub const SCORE_MID_TIER_POINTS: f64 = 90.0;
pub const SCORE_LOW_TIER_POINTS: f64 = 9.0;

/// First residual band after the C tier is full: this many points per leftover unit.
pub const SCORE_FIRST_RESIDUAL_FACTOR: f64 = 0.01;

/// Slots that actually contribute to rally score (A / B / C).
pub const SCORE_SLOT_COUNT: usize = 3;

const _: () = assert!(SCORE_SLOT_COUNT <= MAX_ORE_TYPES);

/// Score haul using area `score_ore_target` (T). Thresholds scale with T; full tier
/// points stay 900 / 90 / 9 / …. When T=30 this matches the legacy hardcoded formula.
/// Values of T below 1 are treated as 1.
pub fn calculate_score(ore: [i32; MAX_ORE_TYPES], score_ore_target: i32) -> f64 {
    calculate_score_breakdown(ore, score_ore_target).total
}

pub fn calculate_score_breakdown_from_slots(
    high: i32,
    mid: i32,
    low: i32,
    score_ore_target: i32,
) -> ScoreBreakdown {
    calculate_score_breakdown(
        ore_amounts(&[(0, high), (1, mid), (2, low)]),
        score_ore_target,
    )
}

/// Same formula as [`calculate_score`], with per-tier amounts for UI explanations.
pub fn calculate_score_breakdown(
    ore: [i32; MAX_ORE_TYPES],
    score_ore_target: i32,
) -> ScoreBreakdown {
    let t = score_ore_target.max(1);
    let mid_cap = 3 * t;
    let low_cap = 12 * t;
    let residual_band = 3 * t;
    let haul = array::from_fn(|index| ore[index].max(0));

    let high = score_tier(haul[0], 0, t, SCORE_HIGH_TIER_POINTS);
    let mid = score_tier(haul[1], high.converted_out, mid_cap, SCORE_MID_TIER_POINTS);
    let low = score_tier(haul[2], mid.converted_out, low_cap, SCORE_LOW_TIER_POINTS);

    let mut residuals = Vec::new();
    let mut remaining = low.overflow_units;
    let mut factor = SCORE_FIRST_RESIDUAL_FACTOR;
    while remaining > 0 {
        let scored_units = remaining.min(residual_band);
        residuals.push(ScoreResidualBand {
            scored_units,
            cap: residual_band,
            points_per_unit: factor,
            points: scored_units as f64 * factor,
        });
        remaining -= residual_band;
        factor /= 10.0;
    }

    let residual_points: f64 = residuals.iter().map(|band| band.points).sum();
    ScoreBreakdown {
        score_ore_target: t,
        haul,
        high,
        mid,
        low,
        residuals,
        total: high.points + mid.points + low.points + residual_points,
    }
}

fn score_tier(haul_units: i32, converted_in: i32, cap: i32, full_tier_points: f64) -> ScoreTier {
    let equivalent_units = haul_units.saturating_add(converted_in);
    let scored_units = equivalent_units.min(cap);
    let overflow_units = (equivalent_units - cap).max(0);
    let points_per_unit = full_tier_points / f64::from(cap);
    ScoreTier {
        haul_units,
        converted_in,
        equivalent_units,
        scored_units,
        cap,
        points_per_unit,
        points: scored_units as f64 * points_per_unit,
        overflow_units,
        converted_out: overflow_units.saturating_mul(2),
    }
}

/// Per-tier rally score explanation produced by [`calculate_score_breakdown`].
#[derive(Debug, Clone, PartialEq)]
pub struct ScoreBreakdown {
    pub score_ore_target: i32,
    pub haul: [i32; MAX_ORE_TYPES],
    pub high: ScoreTier,
    pub mid: ScoreTier,
    pub low: ScoreTier,
    pub residuals: Vec<ScoreResidualBand>,
    pub total: f64,
}

impl ScoreBreakdown {
    pub fn next_unfilled_tier(&self) -> Option<ScoreSlot> {
        if self.high.scored_units < self.high.cap {
            Some(ScoreSlot::High)
        } else if self.mid.scored_units < self.mid.cap {
            Some(ScoreSlot::Mid)
        } else if self.low.scored_units < self.low.cap {
            Some(ScoreSlot::Low)
        } else {
            None
        }
    }
}

/// A / B / C scoring slot.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ScoreSlot {
    High,
    Mid,
    Low,
}

/// One diminishing scoring tier (A, B-equivalent, or C-equivalent).
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ScoreTier {
    pub haul_units: i32,
    pub converted_in: i32,
    pub equivalent_units: i32,
    pub scored_units: i32,
    pub cap: i32,
    pub points_per_unit: f64,
    pub points: f64,
    pub overflow_units: i32,
    pub converted_out: i32,
}

/// Tiny leftover band after the C tier is full.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ScoreResidualBand {
    pub scored_units: i32,
    pub cap: i32,
    pub points_per_unit: f64,
    pub points: f64,
}

pub fn ore_amounts(amounts: &[(usize, i32)]) -> [i32; MAX_ORE_TYPES] {
    let mut ore = array::from_fn(|_| 0);

    for (ore_type, amount) in amounts {
        ore[*ore_type] = *amount;
    }

    ore
}
