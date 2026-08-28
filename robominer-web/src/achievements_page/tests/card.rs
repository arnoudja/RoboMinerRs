use crate::html::{assert_contains_all, assert_html_contains, assert_html_not_contains};

use super::super::AchievementsPageState;
use super::super::render::render_achievements_page;
use super::fixtures::{
    achievement_card_position, sample_achievement_record, sample_achievement_state,
};

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
        depot_total_requirements: Vec::new(),
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
            "Ore &lt;D&gt; dumped in depot",
            r#"class="sufficientbalance">(30)"#,
            r#"class="insufficientbalance">(10)"#,
            "Area &amp; D",
            "New robot",
            r#"class="sufficientbalance">(11)"#,
            r#"class="insufficientbalance">(5)"#,
            ">12.3<",
            r#"class="achievement-progress-meter" value="33.3" max="100""#,
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
fn achievements_score_requirement_uses_display_rounding() {
    let mut state = sample_achievement_state(None);
    state.score_requirements[0].minimum_score = 900.0;
    state.score_requirements[0].current_score = 899.96;

    let html = render_achievements_page("Player".to_string(), None, &state);

    assert_html_contains(
        &html,
        r#"class="achievement-requirement-target">900.0</span>"#,
    );
    assert_html_contains(&html, r#"class="sufficientbalance">(900.0)</span>"#);
    assert_html_not_contains(&html, r#"class="insufficientbalance">(900.0)"#);
}

#[test]
fn achievements_score_requirement_names_robot_when_player_has_two() {
    let mut state = sample_achievement_state(None);
    state.robot_count = 2;
    state.score_requirements[0].minimum_score = 250.0;
    state.score_requirements[0].current_score = 248.6;
    state.score_requirements[0].current_score_robot_name = Some("Robot_1".to_string());

    let html = render_achievements_page("Player".to_string(), None, &state);

    assert_html_contains(
        &html,
        r#"class="achievement-requirement-target">250.0</span>"#,
    );
    assert_html_contains(
        &html,
        r#"class="insufficientbalance">(Robot_1: 248.6)</span>"#,
    );
}

#[test]
fn achievements_score_requirement_omits_robot_name_for_single_robot() {
    let mut state = sample_achievement_state(None);
    state.robot_count = 1;
    state.score_requirements[0].minimum_score = 250.0;
    state.score_requirements[0].current_score = 248.6;
    state.score_requirements[0].current_score_robot_name = Some("Robot_1".to_string());

    let html = render_achievements_page("Player".to_string(), None, &state);

    assert_html_contains(&html, r#"class="insufficientbalance">(248.6)</span>"#);
    assert_html_not_contains(&html, "Robot_1");
}

#[test]
fn achievements_score_requirement_escapes_robot_name() {
    let mut state = sample_achievement_state(None);
    state.robot_count = 2;
    state.score_requirements[0].current_score_robot_name = Some("Bot <1>".to_string());

    let html = render_achievements_page("Player".to_string(), None, &state);

    assert_html_contains(&html, r#"(Bot &lt;1&gt;: 10.0)"#);
    assert_html_not_contains(&html, "(Bot <1>:");
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
