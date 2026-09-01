use std::collections::HashMap;
use std::path::PathBuf;

use crate::html::{assert_contains_all, assert_html_contains, assert_html_not_contains};
use crate::session::format_authenticated_cookie;
use crate::test_support::route;
use crate::{Request, ServerConfig};

use super::MiningAreaOverviewPageState;
use super::render::render_mining_area_overview_page;

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
                total_average_ore_per_run: 12.34,
            },
            robominer_db::MiningAreaOverviewAreaRecord {
                mining_area_id: 11,
                area_name: "Area B".to_string(),
                total_average_ore_per_run: 5.0,
            },
        ],
        ore_averages: vec![robominer_db::MiningAreaOverviewOreAverageRecord {
            mining_area_id: 10,
            ore_id: 2,
            average_ore_per_run: 7.89,
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
            depot_max_allowed: 0,
        }],
    }
}

#[tokio::test(flavor = "current_thread")]
async fn mining_area_overview_requires_database_configuration() {
    let config = ServerConfig {
        static_root: PathBuf::from("robominer-web/static"),
        database_pool: None,
        allow_signup: true,
        trust_proxy: false,
    };

    let response = route(&authenticated_request("/miningAreaOverview"), &config).await;
    let body = response.body_utf8();

    assert_eq!(response.status, 503);
    assert_html_contains(&body, "ROBOMINER_DATABASE_URL");
}

#[tokio::test(flavor = "current_thread")]
async fn mining_area_overview_requires_login() {
    let config = ServerConfig {
        static_root: PathBuf::from("robominer-web/static"),
        database_pool: None,
        allow_signup: true,
        trust_proxy: false,
    };

    let response = route(
        &Request {
            method: "GET".to_string(),
            path: "/miningAreaOverview".to_string(),
            query: HashMap::new(),
            form: HashMap::new(),
            form_values: HashMap::new(),
            headers: HashMap::new(),
        },
        &config,
    )
    .await;

    assert_eq!(response.status, 302);
    assert!(response.headers.iter().any(|(name, value)| {
        *name == "Location" && value == "login?returnTo=miningAreaOverview"
    }));
}

#[test]
fn mining_area_overview_rendering_escapes_fields_and_defaults_missing_averages() {
    let html = render_mining_area_overview_page("Player".to_string(), None, &sample_state());

    assert_contains_all(
        &html,
        &[
            r#"class="mining-area-atlas-page""#,
            r#"class="mining-area-atlas-controls""#,
            r#"id="miningAreaAtlasSort""#,
            r#"<option value="level">Area level</option>"#,
            r#"id="miningAreaAtlasAffordableOnly""#,
            r#"class="mining-area-atlas-title">Mining area atlas</h1>"#,
            r#"href="miningQueue">Back to queue</a>"#,
            r#"href="miningQueue?infoMiningAreaId=10">Area &lt;A&gt;</a>"#,
            r#"class="mining-area-atlas-cost-affordable">30 Ore &amp; Two ✓</span>"#,
            r#"class="mining-area-atlas-cost-unaffordable">Need 10 more Ore &amp; Two.</span>"#,
            r#"data-affordable="1""#,
            r#"data-affordable="0""#,
            "Ore &amp; Two",
            ">12.3<",
            ">7.9<",
            ">0.0<",
            "Averages reflect historic ore mined per claimed run",
        ],
    );
    assert_html_not_contains(&html, "Ore &lt;One&gt;");
}
