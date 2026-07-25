use crate::html::{assert_contains_all, assert_html_not_contains};

use super::super::AchievementsPageState;
use super::super::render::render_achievements_page;
use super::fixtures::achievement_card_position;

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
