//! Mining-area score requirement comparisons for achievements.
//!
//! The achievements UI shows scores with one decimal place (`{:.1}`). Requirement
//! checks use the same rounding so a displayed tie (e.g. both `900.0`) counts as met.

/// Round a mining score the same way the achievements UI displays it (`{:.1}`).
pub fn round_achievement_score(score: f64) -> f64 {
    (score * 10.0).round() / 10.0
}

/// True when the player's score meets the requirement after display rounding.
pub fn achievement_score_meets_requirement(current_score: f64, minimum_score: f64) -> bool {
    round_achievement_score(current_score) >= round_achievement_score(minimum_score)
}

#[cfg(test)]
mod tests {
    use super::{achievement_score_meets_requirement, round_achievement_score};

    #[test]
    fn round_achievement_score_matches_one_decimal_display() {
        assert_eq!(round_achievement_score(899.96), 900.0);
        assert_eq!(round_achievement_score(900.04), 900.0);
        assert_eq!(round_achievement_score(900.0), 900.0);
        assert_eq!(round_achievement_score(12.34), 12.3);
        assert_eq!(round_achievement_score(12.35), 12.4);
    }

    #[test]
    fn requirement_met_when_rounded_scores_are_equal() {
        assert!(achievement_score_meets_requirement(899.96, 900.0));
        assert!(achievement_score_meets_requirement(900.0, 900.0));
        assert!(achievement_score_meets_requirement(900.04, 900.0));
        assert!(!achievement_score_meets_requirement(899.94, 900.0));
    }
}
