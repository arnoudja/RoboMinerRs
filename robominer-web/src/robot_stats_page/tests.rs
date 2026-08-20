use std::collections::HashMap;
use std::path::PathBuf;

use crate::html::{assert_contains_all, assert_html_contains, assert_html_not_contains};
use crate::session::format_authenticated_cookie;
use crate::{Request, ServerConfig};

use super::render::{render_robot_stats_page, render_robot_stats_page_at};
use super::{RobotStatsPageState, robot_stats_page};

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

fn sample_robot_stats_state() -> RobotStatsPageState {
    RobotStatsPageState {
        robot_not_found: false,
        header: Some(robominer_db::RobotStatsHeaderRecord {
            robot_id: 7,
            robot_name: "Bot <Alpha>".to_string(),
            username: "Owner & Co".to_string(),
            total_mining_runs: 10,
        }),
        ore_stats: vec![
            robominer_db::RobotLifetimeOreStatRecord {
                ore_id: 1,
                ore_name: "Ore <Iron>".to_string(),
                amount: 100,
                tax: 20,
            },
            robominer_db::RobotLifetimeOreStatRecord {
                ore_id: 2,
                ore_name: "Copper".to_string(),
                amount: 50,
                tax: 5,
            },
        ],
        area_stats: vec![robominer_db::RobotMiningAreaStatRecord {
            mining_area_id: 3,
            area_name: "Cave & Crystals".to_string(),
            total_runs: 4,
            score: 12.34,
        }],
        recent_runs: vec![
            robominer_db::MiningResultStateRecord {
                robot_id: 7,
                mining_queue_id: 101,
                mining_area_id: 3,
                mining_area_name: "Area <One>".to_string(),
                rally_result_id: Some(55),
                score: 18.5,
                score_ore_target: 30,
                total_ore_mined: 30,
                total_tax: 5,
                total_reward: 25,
                creation_time_millis: 1_000,
                mining_end_time_millis: 3_540_000,
            },
            robominer_db::MiningResultStateRecord {
                robot_id: 7,
                mining_queue_id: 100,
                mining_area_id: 3,
                mining_area_name: "Older Area".to_string(),
                rally_result_id: None,
                score: 9.0,
                score_ore_target: 30,
                total_ore_mined: 12,
                total_tax: 2,
                total_reward: 10,
                creation_time_millis: 500,
                mining_end_time_millis: 1_800_000,
            },
        ],
    }
}

#[tokio::test(flavor = "current_thread")]
async fn robot_stats_requires_database_configuration() {
    let config = ServerConfig {
        static_root: PathBuf::from("robominer-web/static"),
        database_pool: None,
        allow_signup: true,
        trust_proxy: false,
    };

    let response = robot_stats_page(&authenticated_request("/robotStats"), &config).await;
    let body = String::from_utf8(response.body).expect("message should be utf-8");

    assert_eq!(response.status, 503);
    assert_html_contains(&body, "ROBOMINER_DATABASE_URL");
}

#[test]
fn robot_stats_rendering_escapes_fields_and_shows_tables() {
    let html = render_robot_stats_page_at(
        "Player".to_string(),
        None,
        &sample_robot_stats_state(),
        3_600_000,
    );

    assert_contains_all(
        &html,
        &[
            r#"class="robot-stats-page""#,
            "Bot &lt;Alpha&gt;",
            "Owner &amp; Co",
            r#">Total runs</span><span class="robot-stats-summary-value">10</span>"#,
            r#">Ore mined</span><span class="robot-stats-summary-value">150</span>"#,
            r#">Tax paid</span><span class="robot-stats-summary-value">25</span>"#,
            r#">Ore per run</span><span class="robot-stats-summary-value">15.0</span>"#,
            "Cave &amp; Crystals",
            ">12.3<",
            "Ore &lt;Iron&gt;",
            ">80<",
            r#"id="robot-stats-runs-title""#,
            "Area &lt;One&gt;",
            ">18.5<",
            r#"href="activity?rallyResultId=55">View rally</a>"#,
            r#"class="robot-stats-muted">—</td>"#,
            r#"href="leaderboard?tab=robots">Back to Top robots</a>"#,
        ],
    );
    assert!(html.contains("1 minute ago") || html.contains("just now"));
    assert_html_not_contains(&html, "Robot not found.");
}

#[test]
fn robot_stats_rendering_shows_empty_sections_and_not_found() {
    let empty = RobotStatsPageState {
        robot_not_found: false,
        header: Some(robominer_db::RobotStatsHeaderRecord {
            robot_id: 9,
            robot_name: "Idle".to_string(),
            username: "Newbie".to_string(),
            total_mining_runs: 0,
        }),
        ore_stats: Vec::new(),
        area_stats: Vec::new(),
        recent_runs: Vec::new(),
    };
    let empty_html = render_robot_stats_page("Player".to_string(), None, &empty);
    assert_contains_all(
        &empty_html,
        &[
            "No mining area history yet.",
            "No claimed ore totals yet.",
            "No claimed runs yet.",
            r#">Ore per run</span><span class="robot-stats-summary-value">—</span>"#,
        ],
    );

    let missing = RobotStatsPageState {
        robot_not_found: true,
        header: None,
        ore_stats: Vec::new(),
        area_stats: Vec::new(),
        recent_runs: Vec::new(),
    };
    let missing_html = render_robot_stats_page("Player".to_string(), None, &missing);
    assert_contains_all(
        &missing_html,
        &[
            "Robot not found.",
            r#"href="leaderboard?tab=robots">Back to Top robots</a>"#,
        ],
    );
}
