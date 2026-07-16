use std::collections::HashMap;

use crate::{Request, ServerConfig};

use super::render::render_leaderboard_page;
use super::{
    LEADERBOARD_PAGE_SIZE, LeaderboardPageState, LeaderboardQuery, LeaderboardTab, leaderboard_page,
};

use std::path::PathBuf;

fn default_leaderboard_query() -> LeaderboardQuery {
    LeaderboardQuery {
        tab: LeaderboardTab::Areas,
        area_id: Some(1),
        limit: LEADERBOARD_PAGE_SIZE,
    }
}

fn sample_leaderboard_state(
    mining_areas: Vec<robominer_db::LeaderboardMiningAreaRecord>,
    mining_area_scores: Vec<robominer_db::LeaderboardMiningAreaScoreRecord>,
    top_robots: Vec<robominer_db::LeaderboardTopRobotRecord>,
    top_users: Vec<robominer_db::LeaderboardTopUserRecord>,
    viewer_standing: Option<robominer_db::LeaderboardViewerStandingRecord>,
) -> LeaderboardPageState {
    sample_leaderboard_state_with_more(
        mining_areas,
        mining_area_scores,
        top_robots,
        top_users,
        viewer_standing,
        false,
        false,
    )
}

fn sample_leaderboard_state_with_more(
    mining_areas: Vec<robominer_db::LeaderboardMiningAreaRecord>,
    mining_area_scores: Vec<robominer_db::LeaderboardMiningAreaScoreRecord>,
    top_robots: Vec<robominer_db::LeaderboardTopRobotRecord>,
    top_users: Vec<robominer_db::LeaderboardTopUserRecord>,
    viewer_standing: Option<robominer_db::LeaderboardViewerStandingRecord>,
    has_more_robots: bool,
    has_more_players: bool,
) -> LeaderboardPageState {
    LeaderboardPageState {
        mining_areas,
        mining_area_scores,
        top_robots,
        top_users,
        viewer_standing,
        has_more_robots,
        has_more_players,
    }
}

fn request(path: &str) -> Request {
    Request {
        method: "GET".to_string(),
        path: path.to_string(),
        query: HashMap::new(),
        form: HashMap::new(),
        form_values: HashMap::new(),
        headers: HashMap::new(),
    }
}

#[tokio::test(flavor = "current_thread")]
async fn leaderboard_requires_database_configuration() {
    let config = ServerConfig {
        static_root: PathBuf::from("robominer-web/static"),
        database_pool: None,
        allow_signup: true,
        trust_proxy: false,
    };

    let response = leaderboard_page(&request("/leaderboard"), &config).await;
    let body = String::from_utf8(response.body).expect("message should be utf-8");

    assert_eq!(response.status, 503);
    assert!(body.contains("ROBOMINER_DATABASE_URL"));
}

#[test]
fn leaderboard_rendering_escapes_dynamic_fields() {
    let html = render_leaderboard_page(
        "User <One>".to_string(),
        None,
        default_leaderboard_query(),
        &sample_leaderboard_state(
            vec![robominer_db::LeaderboardMiningAreaRecord {
                id: 1,
                area_name: "Area <A>".to_string(),
            }],
            vec![robominer_db::LeaderboardMiningAreaScoreRecord {
                mining_area_id: 1,
                robot_name: "Bot <1>".to_string(),
                username: "Owner & Co".to_string(),
                score: 12.34,
                total_runs: 5,
            }],
            vec![robominer_db::LeaderboardTopRobotRecord {
                robot_name: "Top \"Bot\"".to_string(),
                username: "Top Owner".to_string(),
                ore_per_run: 7.89,
            }],
            vec![robominer_db::LeaderboardTopUserRecord {
                username: "Player <X>".to_string(),
                achievement_points: 42,
            }],
            None,
        ),
    );

    assert!(html.contains("Area &lt;A&gt;"));
    assert!(html.contains("Bot &lt;1&gt;"));
    assert!(html.contains("Owner &amp; Co"));
    assert!(!html.contains("Top &quot;Bot&quot;"));
    assert!(!html.contains("Player &lt;X&gt;"));
    assert!(html.contains(">12.3<"));
    assert!(!html.contains("User <One>"));
}

#[test]
fn leaderboard_rendering_shows_themed_shell_and_rank_rows() {
    let html = render_leaderboard_page(
        "Player".to_string(),
        None,
        default_leaderboard_query(),
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

    assert!(html.contains(r#"class="leaderboard-page""#));
    assert!(html.contains(r#"class="leaderboard-title">Leaderboard</h1>"#));
    assert!(html.contains(r#"class="leaderboard-stats""#));
    assert!(html.contains(r#"class="leaderboard-tab-filter""#));
    assert!(html.contains(r#"href="leaderboard?tab=robots">Top robots</a>"#));
    assert!(html.contains(r#"for="leaderboardAreaFilter""#));
    assert!(html.contains(r#"id="leaderboardAreaFilter""#));
    assert!(html.contains(r#"class="tableitem leaderboard-area-filter-select""#));
    assert!(html.contains(r#"class="leaderboard-sidebar""#));
    assert!(html.contains(r#"class="leaderboard-table""#));
    assert!(html.contains("leaderboard-row-rank-1"));
    assert!(html.contains(r#">#1</td>"#));
    assert!(html.contains(r#">#2</td>"#));
    assert!(html.contains(r#"class="leaderboard-score-meta">12 runs</span>"#));
    assert!(!html.contains(r#"class="leaderboard-section-title">Top robots</h2>"#));
}

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

    assert!(html.contains("leaderboard-tab-link-active"));
    assert!(html.contains(r#"href="leaderboard?tab=robots">Top robots</a>"#));
    assert!(html.contains(r#"class="leaderboard-section-title">Top robots</h2>"#));
    assert!(html.contains("leaderboard-row-self"));
    assert!(html.contains(r#"class="leaderboard-you-badge">You</span>"#));
    assert!(html.contains(r#"class="leaderboard-section-title">Your standings</h2>"#));
    assert!(html.contains("#4 · 55 pts"));
    assert!(html.contains(r#"href="achievements">#4 · 55 pts</a>"#));
    assert!(html.contains("Crystal Cave · 12.0 behind leader (#2)"));
    assert!(html.contains("#2 · 30.0 with My Bot"));
    assert!(!html.contains(r#"leaderboard-standing-value">#1 · 30.0 with My Bot</span>"#));
    assert!(!html.contains(r#"class="leaderboard-area-filter-select""#));
}

#[test]
fn leaderboard_sidebar_shows_rank_one_for_area_leader() {
    let html = render_leaderboard_page(
        "Player".to_string(),
        None,
        default_leaderboard_query(),
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

    assert!(html.contains(r#"leaderboard-standing-value">#1 · 42.0 with My Bot</span>"#));
    assert!(!html.contains("Closest to #1"));
}

#[test]
fn leaderboard_rendering_shows_players_tab() {
    let html = render_leaderboard_page(
        "Champion".to_string(),
        None,
        LeaderboardQuery {
            tab: LeaderboardTab::Players,
            area_id: None,
            limit: LEADERBOARD_PAGE_SIZE,
        },
        &sample_leaderboard_state(
            vec![],
            vec![],
            vec![],
            vec![robominer_db::LeaderboardTopUserRecord {
                username: "Champion".to_string(),
                achievement_points: 100,
            }],
            None,
        ),
    );

    assert!(html.contains(r#"href="leaderboard?tab=players">Top players</a>"#));
    assert!(html.contains(r#"class="leaderboard-section-title">Top players</h2>"#));
    assert!(html.contains("leaderboard-name-self"));
}

#[test]
fn leaderboard_rendering_shows_load_more_cross_links_and_metric_hints() {
    let mut area_scores = Vec::new();
    for index in 0..11 {
        area_scores.push(robominer_db::LeaderboardMiningAreaScoreRecord {
            mining_area_id: 1,
            robot_name: format!("Bot {index}"),
            username: format!("Owner {index}"),
            score: 50.0 - f64::from(index),
            total_runs: index + 1,
        });
    }

    let html = render_leaderboard_page(
        "Player".to_string(),
        None,
        LeaderboardQuery {
            tab: LeaderboardTab::Areas,
            area_id: Some(1),
            limit: LEADERBOARD_PAGE_SIZE,
        },
        &sample_leaderboard_state_with_more(
            vec![robominer_db::LeaderboardMiningAreaRecord {
                id: 1,
                area_name: "Crystal Cave".to_string(),
            }],
            area_scores,
            vec![],
            vec![],
            None,
            false,
            false,
        ),
    );

    assert!(html.contains(r#"href="activity?areaId=1">View area rallies</a>"#));
    assert!(html.contains(r#"href="leaderboard?areaId=1&amp;limit=20">Load more entries</a>"#));
    assert!(html.contains(r#"title="Best single-run score recorded in this mining area.""#));

    let robots_html = render_leaderboard_page(
        "Player".to_string(),
        None,
        LeaderboardQuery {
            tab: LeaderboardTab::Robots,
            area_id: None,
            limit: LEADERBOARD_PAGE_SIZE,
        },
        &sample_leaderboard_state_with_more(
            vec![],
            vec![],
            vec![robominer_db::LeaderboardTopRobotRecord {
                robot_name: "Alpha".to_string(),
                username: "Owner".to_string(),
                ore_per_run: 8.0,
            }],
            vec![],
            None,
            true,
            false,
        ),
    );

    assert!(robots_html.contains(r#"href="miningResults">View mining results</a>"#));
    assert!(
        robots_html.contains(r#"href="leaderboard?tab=robots&amp;limit=20">Load more entries</a>"#)
    );
    assert!(robots_html.contains(r#"title="Lifetime ore gathered divided by total mining runs.""#));

    let players_html = render_leaderboard_page(
        "Player".to_string(),
        None,
        LeaderboardQuery {
            tab: LeaderboardTab::Players,
            area_id: None,
            limit: LEADERBOARD_PAGE_SIZE,
        },
        &sample_leaderboard_state_with_more(
            vec![],
            vec![],
            vec![],
            vec![robominer_db::LeaderboardTopUserRecord {
                username: "Champion".to_string(),
                achievement_points: 100,
            }],
            None,
            false,
            true,
        ),
    );

    assert!(players_html.contains(r#"href="achievements">Champion</a>"#));
    assert!(players_html.contains(r#"href="achievements">View achievements</a>"#));
    assert!(
        players_html.contains(r#"title="Total achievement points claimed across all tracks.""#)
    );
}

#[test]
fn leaderboard_rendering_shows_climb_hints_and_metric_glossary() {
    let html = render_leaderboard_page(
        "Player".to_string(),
        None,
        default_leaderboard_query(),
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

    assert!(html.contains(r#"class="leaderboard-metric-glossary""#));
    assert!(html.contains(r#"class="leaderboard-climb-hint""#));
    assert!(html.contains(r#"class="leaderboard-climb-title">How to climb</h3>"#));
    assert!(html.contains(r#"href="editCode">Edit code</a>"#));
    assert!(html.contains(r#"href="activity">Activity replays</a>"#));
    assert!(html.contains("Best single-run score per robot in a mining area."));
}
