use super::helpers::assert_close;
use crate::*;

#[test]
fn breakdown_total_matches_calculate_score() {
    let cases = [
        ore_amounts(&[]),
        ore_amounts(&[(0, 30)]),
        ore_amounts(&[(0, 15)]),
        ore_amounts(&[(0, 35), (1, 100), (2, 500)]),
        ore_amounts(&[(1, 40)]),
        ore_amounts(&[(2, 12)]),
        ore_amounts(&[(0, 0), (1, 0), (2, 0)]),
        ore_amounts(&[(0, 90), (1, 90), (2, 360)]),
    ];

    for ore in cases {
        for target in [1, 15, 30, 60] {
            assert_close(
                calculate_score_breakdown(ore, target).total,
                calculate_score(ore, target),
            );
        }
    }
}

#[test]
fn breakdown_explains_legacy_t30_tiers() {
    let ore = ore_amounts(&[(0, 35), (1, 100), (2, 500)]);
    let breakdown = calculate_score_breakdown(ore, 30);

    assert_eq!(breakdown.score_ore_target, 30);
    assert_eq!(breakdown.high.cap, 30);
    assert_eq!(breakdown.mid.cap, 90);
    assert_eq!(breakdown.low.cap, 360);
    assert_eq!(breakdown.high.scored_units, 30);
    assert_eq!(breakdown.high.overflow_units, 5);
    assert_eq!(breakdown.high.converted_out, 10);
    assert_close(breakdown.high.points, SCORE_HIGH_TIER_POINTS);
    assert_eq!(breakdown.mid.converted_in, 10);
    assert_eq!(breakdown.mid.equivalent_units, 110);
    assert_eq!(breakdown.mid.scored_units, 90);
    assert_eq!(breakdown.mid.converted_out, 40);
    assert_close(breakdown.mid.points, SCORE_MID_TIER_POINTS);
    assert_eq!(breakdown.low.converted_in, 40);
    assert_eq!(breakdown.low.equivalent_units, 540);
    assert_eq!(breakdown.low.scored_units, 360);
    assert_eq!(breakdown.low.overflow_units, 180);
    assert_close(breakdown.low.points, SCORE_LOW_TIER_POINTS);
    assert!(!breakdown.residuals.is_empty());
    assert_close(breakdown.total, 999.99);
    assert_eq!(breakdown.next_unfilled_tier(), None);
}

#[test]
fn breakdown_points_at_unfilled_a_tier() {
    let ore = ore_amounts(&[(0, 12), (1, 5)]);
    let breakdown = calculate_score_breakdown(ore, 30);

    assert_eq!(breakdown.high.scored_units, 12);
    assert_eq!(breakdown.high.overflow_units, 0);
    assert_close(breakdown.high.points, 360.0);
    assert_eq!(breakdown.mid.scored_units, 5);
    assert_close(breakdown.mid.points, 5.0);
    assert_eq!(breakdown.next_unfilled_tier(), Some(ScoreSlot::High));
}

#[test]
fn negative_haul_is_treated_as_zero() {
    let mut ore = [0; MAX_ORE_TYPES];
    ore[0] = -8;
    let breakdown = calculate_score_breakdown(ore, 30);
    assert_eq!(breakdown.high.haul_units, 0);
    assert_close(breakdown.total, 0.0);
}
