use std::collections::HashMap;
use std::path::PathBuf;

use crate::http::{first_form_values, split_form_field_values};
use crate::session::format_authenticated_cookie;
use crate::{Request, ServerConfig, mutation_i64};

use super::render::render_achievements_page;
use super::{AchievementsPageState, achievements_page, claim_achievement_step_rejection_message};

fn authenticated_request(path: &str) -> Request {
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

fn form_request(path: &str, body: &str) -> Request {
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

fn sample_achievement_state(claim_message: Option<String>) -> AchievementsPageState {
    AchievementsPageState {
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
        }],
        points_summary: robominer_db::AchievementPagePointsSummaryRecord {
            points_earned: 45,
            points_achievable: 150,
        },
    }
}

fn sample_achievement_record(
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
        mining_area_id: Some(2),
        mining_area_name: Some("Area & D".to_string()),
        claimable,
    }
}

fn achievement_card_position(html: &str, achievement_id: i64) -> usize {
    html.find(&format!(r#"id="achievement{achievement_id}""#))
        .unwrap_or_else(|| panic!("achievement {achievement_id} card missing"))
}

#[test]
fn achievements_sort_non_claimable_by_descending_id() {
    let state = AchievementsPageState {
        robot_count: 1,
        claim_message: None,
        achievements: vec![
            sample_achievement_record(2, false, "Low id"),
            sample_achievement_record(5, true, "Ready"),
            sample_achievement_record(99, false, "High id"),
            sample_achievement_record(3, false, "Middle id"),
        ],
        total_requirements: Vec::new(),
        score_requirements: Vec::new(),
        points_summary: robominer_db::AchievementPagePointsSummaryRecord {
            points_earned: 0,
            points_achievable: 0,
        },
    };

    let html = render_achievements_page("Player".to_string(), None, &state);

    let claimable = achievement_card_position(&html, 5);
    let high_id = achievement_card_position(&html, 99);
    let middle_id = achievement_card_position(&html, 3);
    let low_id = achievement_card_position(&html, 2);

    assert!(claimable < high_id);
    assert!(high_id < middle_id);
    assert!(middle_id < low_id);
}

#[tokio::test(flavor = "current_thread")]
async fn achievements_requires_database_configuration() {
    let config = ServerConfig {
        static_root: PathBuf::from("robominer-web/static"),
        database_pool: None,
        allow_signup: true,
        trust_proxy: false,
    };

    let response = achievements_page(&authenticated_request("/achievements"), &config).await;
    let body = String::from_utf8(response.body).expect("message should be utf-8");

    assert_eq!(response.status, 503);
    assert!(body.contains("ROBOMINER_DATABASE_URL"));
}

#[test]
fn form_fields_are_available_to_mutation_parser() {
    let request = form_request("/achievements", "achievementId=42&name=Robo+Miner");

    assert_eq!(mutation_i64(&request, "achievementId"), Some(42));
    assert_eq!(request.form.get("name"), Some(&"Robo Miner".to_string()));

    let mut get_request = request;
    get_request.method = "GET".to_string();
    assert_eq!(mutation_i64(&get_request, "achievementId"), None);
}

#[test]
fn achievements_rendering_groups_requirements_and_escapes_fields() {
    let html = render_achievements_page(
        "Player".to_string(),
        None,
        &sample_achievement_state(Some("Unable to claim <x>".to_string())),
    );

    assert!(html.contains(r#"class="achievements-page""#));
    assert!(html.contains(r#"class="achievements-summary""#));
    assert!(html.contains(r#"class="achievement-card achievement-card-claimable""#));
    assert!(html.contains(r#"class="achievements-banner achievements-banner-error""#));
    assert!(html.contains("Unable to claim &lt;x&gt;"));
    assert!(html.contains("Title &lt;A&gt;"));
    assert!(html.contains("Description &amp; B"));
    assert!(html.contains("Ore &lt;C&gt; ore maximum"));
    assert!(html.contains("50 → 100"));
    assert!(html.contains("Area &amp; D"));
    assert!(html.contains("New robot"));
    assert!(html.contains(r#"class="sufficientbalance">(11)"#));
    assert!(html.contains(r#"class="insufficientbalance">(5)"#));
    assert!(html.contains(">12.3<"));
    assert!(html.contains(r#"class="achievement-progress-bar" style="width: 33.3%"#));
    assert!(html.contains(r#"name="achievementId" value="5""#));
    assert!(html.contains(r#"achievement-claim-badge">Claim</button>"#));
    assert!(!html.contains(r#">Claim step</button>"#));
    assert!(!html.contains("confirmAchievementClaim"));
    assert!(!html.contains("Claim next step for "));
    assert!(html.contains(
        r#">Points earned</span><span class="achievements-summary-value">45/150</span>"#
    ));
    assert!(
        html.contains(r#">Ready to claim</span><span class="achievements-summary-value">1</span>"#)
    );
}

#[test]
fn achievement_rejection_messages_match_engine_output() {
    assert_eq!(
        claim_achievement_step_rejection_message(
            robominer_db::ClaimAchievementStepRejection::RequirementsNotMet
        ),
        "achievement requirements are not met"
    );
    assert_eq!(
        claim_achievement_step_rejection_message(
            robominer_db::ClaimAchievementStepRejection::InvalidDefaultRobotConfiguration
        ),
        "invalid default robot configuration"
    );
}
