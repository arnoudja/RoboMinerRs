use std::collections::HashMap;
use std::path::PathBuf;

use crate::session::format_authenticated_cookie;
use crate::{Request, ServerConfig};

use super::render::render_mining_area_overview_page;
use super::{MiningAreaOverviewPageState, mining_area_overview_page};

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

fn sample_state() -> MiningAreaOverviewPageState {
    MiningAreaOverviewPageState {
        ores: vec![robominer_db::MiningAreaOverviewOreRecord {
            ore_id: 2,
            ore_name: "Ore & Two".to_string(),
        }],
        areas: vec![
            robominer_db::MiningAreaOverviewAreaRecord {
                mining_area_id: 10,
                area_name: "Area <A>".to_string(),
                total_percentage: 12.34,
            },
            robominer_db::MiningAreaOverviewAreaRecord {
                mining_area_id: 11,
                area_name: "Area B".to_string(),
                total_percentage: 5.0,
            },
        ],
        percentages: vec![robominer_db::MiningAreaOverviewPercentageRecord {
            mining_area_id: 10,
            ore_id: 2,
            percentage: 7.89,
        }],
        costs: vec![
            robominer_db::MiningQueuePageAreaCostRecord {
                mining_area_id: 10,
                ore_id: 2,
                ore_name: "Ore & Two".to_string(),
                amount: 30,
            },
            robominer_db::MiningQueuePageAreaCostRecord {
                mining_area_id: 11,
                ore_id: 2,
                ore_name: "Ore & Two".to_string(),
                amount: 50,
            },
        ],
        ore_assets: vec![robominer_db::UserOreAssetStateRecord {
            ore_id: 2,
            ore_name: "Ore & Two".to_string(),
            amount: 40,
            max_allowed: 100,
        }],
    }
}

#[test]
fn mining_area_overview_requires_database_configuration() {
    let config = ServerConfig {
        static_root: PathBuf::from("robominer-web/static"),
        database_pool: None,
        allow_signup: true,
    };

    let response = mining_area_overview_page(&authenticated_request("/miningAreaOverview"), &config);
    let body = String::from_utf8(response.body).expect("message should be utf-8");

    assert_eq!(response.status, 503);
    assert!(body.contains("ROBOMINER_DATABASE_URL"));
}

#[test]
fn mining_area_overview_requires_login() {
    let config = ServerConfig {
        static_root: PathBuf::from("robominer-web/static"),
        database_pool: None,
        allow_signup: true,
    };

    let response = mining_area_overview_page(
        &Request {
            method: "GET".to_string(),
            path: "/miningAreaOverview".to_string(),
            query: HashMap::new(),
            form: HashMap::new(),
            form_values: HashMap::new(),
            headers: HashMap::new(),
        },
        &config,
    );

    assert_eq!(response.status, 302);
    assert!(
        response
            .headers
            .iter()
            .any(|(name, value)| {
                *name == "Location" && value == "login?returnTo=miningAreaOverview"
            })
    );
}

#[test]
fn mining_area_overview_rendering_escapes_fields_and_defaults_missing_percentages() {
    let html = render_mining_area_overview_page("Player".to_string(), None, &sample_state());

    assert!(html.contains(r#"class="mining-area-atlas-page""#));
    assert!(html.contains(r#"class="mining-area-atlas-controls""#));
    assert!(html.contains(r#"id="miningAreaAtlasSort""#));
    assert!(html.contains(r#"id="miningAreaAtlasAffordableOnly""#));
    assert!(html.contains(r#"class="mining-area-atlas-title">Mining area atlas</h1>"#));
    assert!(html.contains(r#"href="miningQueue">Back to queue</a>"#));
    assert!(html.contains(r#"href="miningQueue?infoMiningAreaId=10">Area &lt;A&gt;</a>"#));
    assert!(html.contains(r#"class="mining-area-atlas-cost-affordable">30 Ore &amp; Two ✓</span>"#));
    assert!(html.contains(
        r#"class="mining-area-atlas-cost-unaffordable">Need 10 more Ore &amp; Two.</span>"#
    ));
    assert!(html.contains(r#"data-affordable="1""#));
    assert!(html.contains(r#"data-affordable="0""#));
    assert!(!html.contains("Ore &lt;One&gt;"));
    assert!(html.contains("Ore &amp; Two"));
    assert!(html.contains(">12.3%<"));
    assert!(html.contains(">7.9%<"));
    assert!(html.contains(">0.0%<"));
    assert!(html.contains("function applyMiningAreaAtlasControls()"));
    assert!(html.contains("Percentages reflect historic rally yields"));
}
