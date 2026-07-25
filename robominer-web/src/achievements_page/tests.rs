use std::collections::HashMap;
use std::path::PathBuf;

use crate::html::{assert_contains_all, assert_html_contains, assert_html_not_contains};
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
        current_depot_maximum: 10,
        max_depot_reward: 25,
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
        viewed_username: None,
        player_not_found: false,
        overview_tracks: Vec::new(),
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
    assert_html_contains(&body, "ROBOMINER_DATABASE_URL");
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

    assert_contains_all(
        &html,
        &[
            r#"class="achievements-page""#,
            r#"class="achievements-summary""#,
            r#"class="achievement-card achievement-card-claimable""#,
            r#"class="achievements-banner achievements-banner-error""#,
            "Unable to claim &lt;x&gt;",
            "Title &lt;A&gt;",
            "Description &amp; B",
            "Ore &lt;C&gt; ore maximum",
            "50 → 100",
            "Ore &lt;C&gt; depot maximum",
            "10 → 25",
            "Area &amp; D",
            "New robot",
            r#"class="sufficientbalance">(11)"#,
            r#"class="insufficientbalance">(5)"#,
            ">12.3<",
            r#"class="achievement-progress-bar" style="width: 33.3%"#,
            r#"name="achievementId" value="5""#,
            r#"achievement-claim-badge">Claim</button>"#,
            r#">Points earned</span><span class="achievements-summary-value">45/150</span>"#,
            r#">Ready to claim</span><span class="achievements-summary-value">1</span>"#,
        ],
    );
    for absent in [
        ">Claim step</button>",
        "confirmAchievementClaim",
        "Claim next step for ",
    ] {
        assert_html_not_contains(&html, absent);
    }
}

#[test]
fn achievements_hide_ore_and_depot_maximum_when_reward_does_not_increase() {
    let mut state = sample_achievement_state(None);
    state.achievements[0].current_ore_maximum = 100;
    state.achievements[0].max_ore_reward = 50;
    state.achievements[0].current_depot_maximum = 40;
    state.achievements[0].max_depot_reward = 25;

    let html = render_achievements_page("Player".to_string(), None, &state);

    for absent in ["ore maximum", "depot maximum", "100 → 100", "40 → 40"] {
        assert_html_not_contains(&html, absent);
    }
    assert_html_contains(&html, "Queue increase");
}

#[test]
fn achievements_overview_renders_other_player_tracks_without_claim_ui() {
    let state = AchievementsPageState {
        viewed_username: Some("Champion <X>".to_string()),
        player_not_found: false,
        overview_tracks: vec![
            robominer_db::AchievementOverviewTrackRecord {
                achievement_id: 2,
                title: "Track <Done>".to_string(),
                description: "Finished & sealed".to_string(),
                steps_claimed: 2,
                number_of_steps: 2,
                points_earned: 30,
                total_points: 30,
            },
            robominer_db::AchievementOverviewTrackRecord {
                achievement_id: 5,
                title: "Track <Open>".to_string(),
                description: "Still going".to_string(),
                steps_claimed: 1,
                number_of_steps: 3,
                points_earned: 10,
                total_points: 40,
            },
        ],
        robot_count: 0,
        achievements: Vec::new(),
        total_requirements: Vec::new(),
        score_requirements: Vec::new(),
        points_summary: robominer_db::AchievementPagePointsSummaryRecord {
            points_earned: 40,
            points_achievable: 150,
        },
        claim_message: None,
    };

    let html = render_achievements_page("Player".to_string(), None, &state);

    assert_contains_all(
        &html,
        &[
            r#"class="achievements-page achievements-page-overview""#,
            "Champion &lt;X&gt;&#39;s achievements",
            "Track &lt;Done&gt;",
            "Finished &amp; sealed",
            "Track &lt;Open&gt;",
            r#"achievement-status-complete">Completed</span>"#,
            r#"achievement-status-progress">In progress</span>"#,
            r#">Points earned</span><span class="achievements-summary-value">40/150</span>"#,
            r#">Tracks</span><span class="achievements-summary-value">2</span>"#,
            r#"href="leaderboard">Back to Top players</a>"#,
        ],
    );
    for absent in ["Ready to claim", "Claim", "Next reward", "Requirements"] {
        assert_html_not_contains(&html, absent);
    }
    let done = achievement_card_position(&html, 2);
    let open = achievement_card_position(&html, 5);
    assert!(
        open < done,
        "overview tracks should sort by achievement id descending"
    );
}

#[test]
fn achievements_overview_shows_not_found_for_missing_player() {
    let state = AchievementsPageState {
        viewed_username: Some("Missing <Player>".to_string()),
        player_not_found: true,
        overview_tracks: Vec::new(),
        robot_count: 0,
        achievements: Vec::new(),
        total_requirements: Vec::new(),
        score_requirements: Vec::new(),
        points_summary: robominer_db::AchievementPagePointsSummaryRecord {
            points_earned: 0,
            points_achievable: 0,
        },
        claim_message: None,
    };

    let html = render_achievements_page("Player".to_string(), None, &state);

    assert_contains_all(
        &html,
        &[
            "Missing &lt;Player&gt;",
            "Player not found.",
            r#"href="leaderboard">Back to Top players</a>"#,
        ],
    );
    assert_html_not_contains(&html, "Claim");
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
