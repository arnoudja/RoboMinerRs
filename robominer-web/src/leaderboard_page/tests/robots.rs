use crate::html::{assert_contains_all, assert_html_not_contains};

use super::super::render::render_leaderboard_page;
use super::super::{LEADERBOARD_PAGE_SIZE, LeaderboardQuery, LeaderboardTab};
use super::fixtures::sample_leaderboard_state;

#[test]
fn leaderboard_rendering_shows_robots_tab_and_viewer_highlights() {
    let html = render_leaderboard_page(
        "Player".to_string(),
        None,
        LeaderboardQuery {
            tab: LeaderboardTab::Robots,
            area_id: None,
            limit: LEADERBOARD_PAGE_SIZE,
        },
        &sample_leaderboard_state(
            vec![robominer_db::LeaderboardMiningAreaRecord {
                id: 1,
                area_name: "Crystal Cave".to_string(),
            }],
            vec![
                robominer_db::LeaderboardMiningAreaScoreRecord {
                    mining_area_id: 1,
                    robot_name: "Leader Bot".to_string(),
                    username: "Leader".to_string(),
                    score: 42.0,
                    total_runs: 10,
                },
                robominer_db::LeaderboardMiningAreaScoreRecord {
                    mining_area_id: 1,
                    robot_name: "My Bot".to_string(),
                    username: "Player".to_string(),
                    score: 30.0,
                    total_runs: 5,
                },
            ],
            vec![robominer_db::LeaderboardTopRobotRecord {
                robot_id: 7,
                robot_name: "My Bot".to_string(),
                username: "Player".to_string(),
                ore_per_run: 9.5,
            }],
            vec![],
            Some(robominer_db::LeaderboardViewerStandingRecord {
                achievement_points: 55,
                achievement_rank: 4,
                area_standings: vec![robominer_db::LeaderboardViewerAreaStandingRecord {
                    mining_area_id: 1,
                    area_name: "Crystal Cave".to_string(),
                    robot_name: "My Bot".to_string(),
                    score: 30.0,
                    rank: 2,
                }],
            }),
        ),
    );

    assert_contains_all(
        &html,
        &[
            "leaderboard-tab-link-active",
            r#"href="leaderboard?tab=robots">Top robots</a>"#,
            r#"class="leaderboard-section-title">Top robots</h2>"#,
            r#"class="leaderboard-table leaderboard-table-split""#,
            r#"href="robotStats?robotId=7">My Bot</a>"#,
            "leaderboard-row-self",
            r#"class="leaderboard-you-badge">You</span>"#,
            r#"class="leaderboard-section-title">Your standings</h2>"#,
            "#4 · 55 pts",
            r#"href="achievements">#4 · 55 pts</a>"#,
            "Crystal Cave · 12.0 behind leader (#2)",
            "#2 · 30.0 with My Bot",
        ],
    );
    assert_html_not_contains(
        &html,
        r#"leaderboard-standing-value">#1 · 30.0 with My Bot</span>"#,
    );
    assert_html_not_contains(&html, r#"class="leaderboard-area-filter-select""#);
}
