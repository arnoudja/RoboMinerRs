//! Shared fixtures for `achievements_page` unit tests.

use std::collections::HashMap;

use crate::Request;
use crate::http::{first_form_values, split_form_field_values};
use crate::session::format_authenticated_cookie;

use super::super::AchievementsPageState;

pub(super) fn authenticated_request(path: &str) -> Request {
    Request {
        method: "GET".to_string(),
        path: path.to_string(),
        query: HashMap::new(),
        form: HashMap::new(),
        form_values: HashMap::new(),
        headers: HashMap::from([(
            "cookie".to_string(),
            format_authenticated_cookie(42, "Player"),
        )]),
    }
}

pub(super) fn form_request(path: &str, body: &str) -> Request {
    let mut request = authenticated_request(path);
    request.method = "POST".to_string();
    request.headers.insert(
        "content-type".to_string(),
        "application/x-www-form-urlencoded".to_string(),
    );
    request.form_values = split_form_field_values(body);
    request.form = first_form_values(&request.form_values);
    request
}

pub(super) fn sample_achievement_state(claim_message: Option<String>) -> AchievementsPageState {
    AchievementsPageState {
        viewed_username: None,
        player_not_found: false,
        overview_tracks: Vec::new(),
        robot_count: 1,
        claim_message,
        achievements: vec![sample_achievement_record(5, true, "Title <A>")],
        total_requirements: vec![
            robominer_db::AchievementPageTotalRequirementRecord {
                achievement_id: 5,
                ore_id: 1,
                ore_name: "Ore <C>".to_string(),
                amount: 10,
                current_amount: 11,
            },
            robominer_db::AchievementPageTotalRequirementRecord {
                achievement_id: 5,
                ore_id: 2,
                ore_name: "Ore E".to_string(),
                amount: 20,
                current_amount: 5,
            },
        ],
        score_requirements: vec![robominer_db::AchievementPageScoreRequirementRecord {
            achievement_id: 5,
            mining_area_id: 2,
            area_name: "Area & D".to_string(),
            minimum_score: 12.34,
            current_score: 10.0,
            current_score_robot_name: Some("Robot_1".to_string()),
        }],
        depot_total_requirements: vec![
            robominer_db::AchievementPageDepotTotalRequirementRecord {
                achievement_id: 5,
                ore_id: 3,
                ore_name: "Ore <D>".to_string(),
                amount: 25,
                current_amount: 30,
            },
            robominer_db::AchievementPageDepotTotalRequirementRecord {
                achievement_id: 5,
                ore_id: 1,
                ore_name: "Ore <C>".to_string(),
                amount: 15,
                current_amount: 10,
            },
        ],
        points_summary: robominer_db::AchievementPagePointsSummaryRecord {
            points_earned: 45,
            points_achievable: 150,
        },
    }
}

pub(super) fn sample_achievement_record(
    achievement_id: i64,
    claimable: bool,
    title: &str,
) -> robominer_db::AchievementPageStateRecord {
    robominer_db::AchievementPageStateRecord {
        achievement_id,
        title: title.to_string(),
        description: "Description & B".to_string(),
        steps_claimed: 1,
        number_of_steps: 2,
        achievement_points_earned: 10,
        total_achievement_points: 30,
        step: 2,
        next_achievement_points: 20,
        mining_queue_reward: 1,
        robot_reward: 2,
        ore_id: Some(1),
        ore_name: Some("Ore <C>".to_string()),
        current_ore_maximum: 50,
        max_ore_reward: 100,
        current_depot_maximum: 10,
        max_depot_reward: 25,
        mining_area_id: Some(2),
        mining_area_name: Some("Area & D".to_string()),
        claimable,
    }
}

pub(super) fn achievement_card_position(html: &str, achievement_id: i64) -> usize {
    html.find(&format!(r#"id="achievement{achievement_id}""#))
        .unwrap_or_else(|| panic!("achievement {achievement_id} card missing"))
}
