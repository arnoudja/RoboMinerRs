use std::collections::HashMap;
use std::path::PathBuf;

use crate::html::{assert_contains_all, assert_html_contains, assert_html_not_contains};
use crate::{Request, ServerConfig};

use super::super::render::render_mining_results_page;
use super::super::{MiningResultsPageState, mining_results_page};
use super::fixtures::{authenticated_request, sample_mining_results_state};

#[tokio::test(flavor = "current_thread")]
async fn mining_results_requires_database_configuration() {
    let config = ServerConfig {
        static_root: PathBuf::from("robominer-web/static"),
        database_pool: None,
        allow_signup: true,
        trust_proxy: false,
    };

    let response = mining_results_page(&authenticated_request("/miningResults"), &config).await;
    let body = String::from_utf8(response.body).expect("message should be utf-8");

    assert_eq!(response.status, 503);
    assert_html_contains(&body, "ROBOMINER_DATABASE_URL");
}

#[tokio::test(flavor = "current_thread")]
async fn mining_results_redirects_to_login_when_logged_out() {
    let config = ServerConfig {
        static_root: PathBuf::from("robominer-web/static"),
        database_pool: None,
        allow_signup: true,
        trust_proxy: false,
    };
    let request = Request {
        method: "GET".to_string(),
        path: "/miningResults".to_string(),
        query: HashMap::new(),
        form: HashMap::new(),
        form_values: HashMap::new(),
        headers: HashMap::new(),
    };

    let response = mining_results_page(&request, &config).await;
    assert_eq!(response.status, 302);
    assert!(
        response
            .headers
            .iter()
            .any(|(name, value)| *name == "Location" && value.starts_with("login?"))
    );
}

#[tokio::test(flavor = "current_thread")]
async fn mining_results_unknown_rally_returns_not_found_without_database() {
    // Without a pool the handler still short-circuits on missing DB before rally lookup.
    let config = ServerConfig {
        static_root: PathBuf::from("robominer-web/static"),
        database_pool: None,
        allow_signup: true,
        trust_proxy: false,
    };
    let mut request = authenticated_request("/miningResults");
    request
        .query
        .insert("rallyResultId".to_string(), "1".to_string());

    let response = mining_results_page(&request, &config).await;
    assert_eq!(response.status, 503);
}

#[test]
fn mining_results_rendering_groups_results_and_escapes_fields() {
    let html =
        render_mining_results_page("Player".to_string(), None, &sample_mining_results_state());

    assert_contains_all(
        &html,
        &[
            r#"class="mining-results-page""#,
            r#"class="mining-results-deck""#,
            r#"class="mining-results-log""#,
            r#"class="mining-results-filters""#,
            r#"id="miningResultsRobotFilter""#,
            r#"id="miningResultsAreaFilter""#,
            r#"id="miningResultsSortFilter""#,
            r#"class="mining-results-wallet-delta""#,
            r#"class="mining-results-wallet-delta-amount">+9</span>"#,
            r#"class="mining-results-wallet-delta-amount">+18</span>"#,
            r#"class="mining-results-robot-empty">No recent runs for Bot &amp; Two.</p>"#,
            r#"data-sort-reward="27""#,
            r#"data-rally-result-id="99""#,
            r#"src="js/common/url_query.js?v="#,
            r#"src="js/mining_results/page.js?v="#,
            r#"data-robot-id="1" data-area-name="Area &amp; One""#,
            r#"class="mining-results-atlas-helper""#,
            r#"class="mining-results-ore-values">20 mined · 2 tax · +18 net</span></li><li><span class="mining-results-ore-name">Ore &lt;A&gt;</span><span class="mining-results-ore-values">10 mined · 1 tax · +9 net</span></li>"#,
            r#"title="Tax is deducted before ore is added to your wallet. Container tax applies to cargo still in the robot; depot tax applies to ore already banked in the depot.""#,
            r#"class="mining-results-run-card mining-results-run-card-active" data-run-id="10""#,
            r#"class="mining-results-detail-panel mining-results-detail-panel-active" id="miningResultDetails10" data-run-id="10""#,
            "Showing last completed runs",
            r#"class="mining-results-robot-title">Bot &lt;One&gt;</h3>"#,
            "Area &amp; One",
            "Ore &amp; B · Ore &lt;A&gt;",
            "miningResults?rallyResultId=99",
            "Replay rally",
            "+27 net",
            "Score 12.3",
            ">12.3<",
            "Scan",
            "50.0%",
            "33.3%",
            "16.7%",
            "1970-01-01 00:00:00 UTC",
            "1970-01-01 00:00:01 UTC",
        ],
    );
    for absent in [
        r#"<script src="js/miningresults.js"></script>"#,
        "function applyMiningResultsSort()",
        r#"<details class="mining-results-run-card""#,
    ] {
        assert_html_not_contains(&html, absent);
    }
}

#[test]
fn mining_results_shows_empty_state_and_claim_banner() {
    let empty_html = render_mining_results_page(
        "Player".to_string(),
        None,
        &MiningResultsPageState {
            robots: vec![robominer_db::MiningQueuePageRobotRecord {
                robot_id: 1,
                robot_name: "Idle".to_string(),
                recharge_time: 60,
            }],
            results: Vec::new(),
            ore_results: Vec::new(),
            action_results: Vec::new(),
            claimed_results: robominer_db::ClaimedUserResults {
                claimed_queues: 2,
                ore_rewards: vec![
                    robominer_db::ClaimedOreRewardRecord {
                        ore_id: 2,
                        ore_name: "Ore & Two".to_string(),
                        reward: 9,
                    },
                    robominer_db::ClaimedOreRewardRecord {
                        ore_id: 1,
                        ore_name: "Cerbonium".to_string(),
                        reward: 18,
                    },
                ],
            },
            selected_mining_queue_id: None,
        },
    );

    assert_contains_all(
        &empty_html,
        &[
            r#"class="mining-results-empty""#,
            r#"href="miningQueue">Check the mining queue</a>"#,
            r#"class="mining-results-claim-banner"><span class="claim-banner-label">Added to wallet:</span>"#,
            r#"class="claim-banner-reward-amount">+18</span>"#,
            r#"class="claim-banner-reward-amount">+9</span>"#,
        ],
    );
    for absent in [
        "Claimed 2 mining result(s) into your wallet",
        r#"class="mining-results-run-card""#,
    ] {
        assert_html_not_contains(&empty_html, absent);
    }
}
