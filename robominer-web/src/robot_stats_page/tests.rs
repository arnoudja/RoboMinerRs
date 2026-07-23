use std::collections::HashMap;
use std::path::PathBuf;

use crate::session::format_authenticated_cookie;
use crate::{Request, ServerConfig};

use super::render::render_robot_stats_page;
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
    assert!(body.contains("ROBOMINER_DATABASE_URL"));
}

#[test]
fn robot_stats_rendering_escapes_fields_and_shows_tables() {
    let html = render_robot_stats_page("Player".to_string(), None, &sample_robot_stats_state());

    assert!(html.contains(r#"class="robot-stats-page""#));
    assert!(html.contains("Bot &lt;Alpha&gt;"));
    assert!(html.contains("Owner &amp; Co"));
    assert!(
        html.contains(r#">Total runs</span><span class="robot-stats-summary-value">10</span>"#)
    );
    assert!(
        html.contains(r#">Ore mined</span><span class="robot-stats-summary-value">150</span>"#)
    );
    assert!(html.contains(r#">Tax paid</span><span class="robot-stats-summary-value">25</span>"#));
    assert!(
        html.contains(r#">Ore per run</span><span class="robot-stats-summary-value">15.0</span>"#)
    );
    assert!(html.contains("Cave &amp; Crystals"));
    assert!(html.contains(">12.3<"));
    assert!(html.contains("Ore &lt;Iron&gt;"));
    assert!(html.contains(">80<"));
    assert!(html.contains(r#"href="leaderboard?tab=robots">Back to Top robots</a>"#));
    assert!(!html.contains("Robot not found."));
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
    };
    let empty_html = render_robot_stats_page("Player".to_string(), None, &empty);
    assert!(empty_html.contains("No mining area history yet."));
    assert!(empty_html.contains("No claimed ore totals yet."));
    assert!(
        empty_html
            .contains(r#">Ore per run</span><span class="robot-stats-summary-value">—</span>"#)
    );

    let missing = RobotStatsPageState {
        robot_not_found: true,
        header: None,
        ore_stats: Vec::new(),
        area_stats: Vec::new(),
    };
    let missing_html = render_robot_stats_page("Player".to_string(), None, &missing);
    assert!(missing_html.contains("Robot not found."));
    assert!(missing_html.contains(r#"href="leaderboard?tab=robots">Back to Top robots</a>"#));
}
