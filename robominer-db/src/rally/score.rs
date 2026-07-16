use crate::{SCORE_HISTORY_FACTOR, SCORE_START_FACTOR};

pub fn updated_robot_mining_area_score(previous_score: Option<f64>, score: f64) -> f64 {
    match previous_score {
        Some(previous_score) => {
            ((SCORE_HISTORY_FACTOR - 1.0) * previous_score + score) / SCORE_HISTORY_FACTOR
        }
        None => score / SCORE_START_FACTOR,
    }
}

#[cfg(test)]
mod tests {
    use super::updated_robot_mining_area_score;

    #[test]
    fn new_robot_mining_area_scores_start_below_raw_score() {
        assert_eq!(updated_robot_mining_area_score(None, 140.0), 100.0);
    }

    #[test]
    fn existing_robot_mining_area_scores_use_legacy_history_factor() {
        assert_eq!(updated_robot_mining_area_score(Some(100.0), 150.0), 110.0);
    }
}
