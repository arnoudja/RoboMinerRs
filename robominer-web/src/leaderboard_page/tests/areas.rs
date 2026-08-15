use crate::html::{assert_contains_all, assert_html_contains, assert_html_not_contains};

use super::super::render::render_leaderboard_page;
use super::super::render_areas::render_leaderboard_area_section;
use super::super::{LEADERBOARD_PAGE_SIZE, LeaderboardQuery, LeaderboardTab};
use super::fixtures::{areas_leaderboard_query, sample_leaderboard_state};

#[test]
fn leaderboard_rendering_shows_themed_shell_and_rank_rows() {
    let html = render_leaderboard_page(
        "Player".to_string(),
        None,
        areas_leaderboard_query(),
        &sample_leaderboard_state(
            vec![robominer_db::LeaderboardMiningAreaRecord {
                id: 1,
                area_name: "Crystal Cave".to_string(),
            }],
            vec![
                robominer_db::LeaderboardMiningAreaScoreRecord {
                    mining_area_id: 1,
                    robot_name: "Alpha".to_string(),
                    username: "First".to_string(),
                    score: 42.0,
                    total_runs: 12,
                },
                robominer_db::LeaderboardMiningAreaScoreRecord {
                    mining_area_id: 1,
                    robot_name: "Beta".to_string(),
                    username: "Second".to_string(),
                    score: 38.5,
                    total_runs: 8,
                },
            ],
            vec![robominer_db::LeaderboardTopRobotRecord {
                robot_id: 7,
                robot_name: "Gamma".to_string(),
                username: "Third".to_string(),
                ore_per_run: 9.5,
            }],
            vec![robominer_db::LeaderboardTopUserRecord {
                username: "Champion".to_string(),
                achievement_points: 100,
            }],
            None,
        ),
    );

    assert_contains_all(
        &html,
        &[
            r#"class="leaderboard-page""#,
            r#"class="leaderboard-title">Leaderboard</h1>"#,
            r#"class="leaderboard-stats""#,
            r#"class="leaderboard-tab-filter""#,
            r#">Top players</a><a class="leaderboard-tab-link" href="leaderboard?tab=robots">Top robots</a><a class="leaderboard-tab-link leaderboard-tab-link-active" href="leaderboard?tab=areas&amp;areaId=1">By area</a>"#,
            r#"for="leaderboardAreaFilter""#,
            r#"id="leaderboardAreaFilter""#,
            r#"class="tableitem leaderboard-area-filter-select""#,
            r#"class="leaderboard-sidebar""#,
            r#"class="leaderboard-table leaderboard-table-split""#,
            "leaderboard-row-rank-1",
            r#">#1</td>"#,
            r#">#2</td>"#,
            r#"class="leaderboard-score-meta">12 runs</span>"#,
        ],
    );
    assert_html_not_contains(
        &html,
        r#"class="leaderboard-section-title">Top robots</h2>"#,
    );
}

#[test]
fn leaderboard_sidebar_shows_rank_one_for_area_leader() {
    let html = render_leaderboard_page(
        "Player".to_string(),
        None,
        areas_leaderboard_query(),
        &sample_leaderboard_state(
            vec![robominer_db::LeaderboardMiningAreaRecord {
                id: 1,
                area_name: "Crystal Cave".to_string(),
            }],
            vec![robominer_db::LeaderboardMiningAreaScoreRecord {
                mining_area_id: 1,
                robot_name: "My Bot".to_string(),
                username: "Player".to_string(),
                score: 42.0,
                total_runs: 10,
            }],
            vec![],
            vec![],
            Some(robominer_db::LeaderboardViewerStandingRecord {
                achievement_points: 10,
                achievement_rank: 5,
                area_standings: vec![robominer_db::LeaderboardViewerAreaStandingRecord {
                    mining_area_id: 1,
                    area_name: "Crystal Cave".to_string(),
                    robot_name: "My Bot".to_string(),
                    score: 42.0,
                    rank: 1,
                }],
            }),
        ),
    );

    assert_html_contains(
        &html,
        r#"leaderboard-standing-value">#1 · 42.0 with My Bot</span>"#,
    );
    assert_html_not_contains(&html, "Closest to #1");
}

#[test]
fn leaderboard_rendering_shows_climb_hints_and_metric_glossary() {
    let html = render_leaderboard_page(
        "Player".to_string(),
        None,
        areas_leaderboard_query(),
        &sample_leaderboard_state(
            vec![robominer_db::LeaderboardMiningAreaRecord {
                id: 1,
                area_name: "Crystal Cave".to_string(),
            }],
            vec![robominer_db::LeaderboardMiningAreaScoreRecord {
                mining_area_id: 1,
                robot_name: "Leader Bot".to_string(),
                username: "Leader".to_string(),
                score: 42.0,
                total_runs: 10,
            }],
            vec![],
            vec![],
            None,
        ),
    );

    assert_contains_all(
        &html,
        &[
            r#"class="leaderboard-metric-glossary""#,
            r#"class="leaderboard-climb-hint""#,
            r#"class="leaderboard-climb-title">How to climb</h3>"#,
            r#"href="editCode">Edit code</a>"#,
            r#"href="activity">Activity replays</a>"#,
            "Smoothed running score per robot in a mining area",
        ],
    );
}

#[test]
fn leaderboard_areas_empty_ranked_shows_queue_cta() {
    let html = render_leaderboard_page(
        "Player".to_string(),
        None,
        LeaderboardQuery {
            tab: LeaderboardTab::Areas,
            area_id: None,
            limit: LEADERBOARD_PAGE_SIZE,
        },
        &sample_leaderboard_state(vec![], vec![], vec![], vec![], None),
    );

    assert_contains_all(
        &html,
        &[
            "No area scores yet. Queue mining runs to start climbing the board.",
            r#"href="miningQueue""#,
            r#"href="helpTutorial?step=1""#,
        ],
    );
}

#[test]
fn leaderboard_areas_without_selected_area_prompts_choice() {
    let mut body = String::new();
    render_leaderboard_area_section(
        &mut body,
        LeaderboardQuery {
            tab: LeaderboardTab::Areas,
            area_id: None,
            limit: LEADERBOARD_PAGE_SIZE,
        },
        &[&robominer_db::LeaderboardMiningAreaRecord {
            id: 3,
            area_name: "Crystal Cave".to_string(),
        }],
        &[],
        "Player",
    );

    assert_html_contains(&body, "Choose a mining area to view its leaderboard.");
}
