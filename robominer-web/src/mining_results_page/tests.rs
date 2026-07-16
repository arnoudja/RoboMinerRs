use std::collections::HashMap;

use crate::{Request, ServerConfig};

use super::render::{
    mining_result_unique_areas, mining_result_wallet_deltas, render_mining_results_page,
};
use super::{MiningResultsPageState, mining_results_page, selected_mining_queue_id};

use std::path::PathBuf;

use crate::session::format_authenticated_cookie;

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

fn sample_mining_results_state() -> MiningResultsPageState {
    MiningResultsPageState {
        robots: vec![
            robominer_db::MiningQueuePageRobotRecord {
                robot_id: 1,
                robot_name: "Bot <One>".to_string(),
                recharge_time: 60,
            },
            robominer_db::MiningQueuePageRobotRecord {
                robot_id: 2,
                robot_name: "Bot & Two".to_string(),
                recharge_time: 60,
            },
        ],
        results: vec![robominer_db::MiningResultStateRecord {
            robot_id: 1,
            mining_queue_id: 10,
            mining_area_name: "Area & One".to_string(),
            rally_result_id: Some(99),
            score: 12.34,
            total_ore_mined: 30,
            total_tax: 3,
            total_reward: 27,
            creation_time_millis: 0,
            mining_end_time_millis: 1_000,
        }],
        ore_results: vec![
            robominer_db::MiningResultOreStateRecord {
                mining_queue_id: 10,
                ore_id: 1,
                ore_name: "Ore <A>".to_string(),
                amount: 10,
                tax: 1,
                reward: 9,
            },
            robominer_db::MiningResultOreStateRecord {
                mining_queue_id: 10,
                ore_id: 2,
                ore_name: "Ore & B".to_string(),
                amount: 20,
                tax: 2,
                reward: 18,
            },
        ],
        action_results: vec![
            robominer_db::MiningResultActionStateRecord {
                mining_queue_id: 10,
                action_type: 0,
                amount: 2,
            },
            robominer_db::MiningResultActionStateRecord {
                mining_queue_id: 10,
                action_type: 2,
                amount: 3,
            },
            robominer_db::MiningResultActionStateRecord {
                mining_queue_id: 10,
                action_type: 6,
                amount: 1,
            },
        ],
        claimed_results: robominer_db::ClaimedUserResults {
            claimed_queues: 0,
            ore_rewards: vec![],
        },
        selected_mining_queue_id: Some(10),
    }
}

#[tokio::test(flavor = "current_thread")]
async fn mining_results_requires_database_configuration() {
    let config = ServerConfig {
        static_root: PathBuf::from("robominer-web/static"),
        database_pool: None,
        allow_signup: true,
    };

    let response = mining_results_page(&authenticated_request("/miningResults"), &config).await;
    let body = String::from_utf8(response.body).expect("message should be utf-8");

    assert_eq!(response.status, 503);
    assert!(body.contains("ROBOMINER_DATABASE_URL"));
}

#[test]
fn mining_results_rendering_groups_results_and_escapes_fields() {
    let html =
        render_mining_results_page("Player".to_string(), None, &sample_mining_results_state());

    assert!(!html.contains(r#"<script src="js/miningresults.js"></script>"#));
    assert!(html.contains(r#"class="mining-results-page""#));
    assert!(html.contains(r#"class="mining-results-deck""#));
    assert!(html.contains(r#"class="mining-results-log""#));
    assert!(html.contains(r#"class="mining-results-filters""#));
    assert!(html.contains(r#"id="miningResultsRobotFilter""#));
    assert!(html.contains(r#"id="miningResultsAreaFilter""#));
    assert!(html.contains(r#"id="miningResultsSortFilter""#));
    assert!(html.contains(r#"class="mining-results-wallet-delta""#));
    assert!(html.contains(r#"class="mining-results-wallet-delta-amount">+9</span>"#));
    assert!(html.contains(r#"class="mining-results-wallet-delta-amount">+18</span>"#));
    assert!(
        html.contains(
            r#"class="mining-results-robot-empty">No recent runs for Bot &amp; Two.</p>"#
        )
    );
    assert!(html.contains(r#"data-sort-reward="27""#));
    assert!(html.contains(r#"data-rally-result-id="99""#));
    assert!(html.contains("function applyMiningResultsSort()"));
    assert!(html.contains("function syncReplayReturnLinks()"));
    assert!(html.contains(r#"data-robot-id="1" data-area-name="Area &amp; One""#));
    assert!(html.contains(r#"class="mining-results-atlas-helper""#));
    assert!(
        html.contains(
            r#"class="mining-results-ore-values">20 mined · 2 tax · +18 net</span></li>"#
        )
    );
    assert!(html.contains(r#"title="Tax is deducted before ore is added to your wallet.""#));
    assert!(html.contains(
        r#"class="mining-results-run-card mining-results-run-card-active" data-run-id="10""#
    ));
    assert!(html.contains(
        r#"class="mining-results-detail-panel mining-results-detail-panel-active" id="miningResultDetails10" data-run-id="10""#
    ));
    assert!(html.contains(r#"Showing last completed runs"#));
    assert!(html.contains(r#"class="mining-results-robot-title">Bot &lt;One&gt;</h3>"#));
    assert!(html.contains("Area &amp; One"));
    assert!(html.contains("Ore &lt;A&gt;"));
    assert!(html.contains("Ore &amp; B"));
    assert!(html.contains(r#"miningResults?rallyResultId=99"#));
    assert!(html.contains("Replay rally"));
    assert!(html.contains("+27 net"));
    assert!(html.contains("Score 12.3"));
    assert!(html.contains(">12.3<"));
    assert!(html.contains("Scan"));
    assert!(html.contains("50.0%"));
    assert!(html.contains("33.3%"));
    assert!(html.contains("16.7%"));
    assert!(html.contains("1970-01-01 00:00:00 UTC"));
    assert!(html.contains("1970-01-01 00:00:01 UTC"));
    assert!(html.contains("function selectMiningResultRun(runId, updateUrl)"));
    assert!(html.contains("function applyMiningResultsFilters(preferredRunId)"));
    assert!(html.contains("encodeURIComponent('runId')"));
    assert!(!html.contains(r#"<details class="mining-results-run-card""#));
}

#[test]
fn mining_result_unique_areas_are_sorted_and_deduped() {
    let results = vec![
        robominer_db::MiningResultStateRecord {
            robot_id: 1,
            mining_queue_id: 10,
            mining_area_name: "Beta".to_string(),
            rally_result_id: None,
            score: 1.0,
            total_ore_mined: 1,
            total_tax: 0,
            total_reward: 1,
            creation_time_millis: 0,
            mining_end_time_millis: 0,
        },
        robominer_db::MiningResultStateRecord {
            robot_id: 1,
            mining_queue_id: 11,
            mining_area_name: "Alpha".to_string(),
            rally_result_id: None,
            score: 2.0,
            total_ore_mined: 2,
            total_tax: 0,
            total_reward: 2,
            creation_time_millis: 0,
            mining_end_time_millis: 0,
        },
        robominer_db::MiningResultStateRecord {
            robot_id: 1,
            mining_queue_id: 12,
            mining_area_name: "Beta".to_string(),
            rally_result_id: None,
            score: 3.0,
            total_ore_mined: 3,
            total_tax: 0,
            total_reward: 3,
            creation_time_millis: 0,
            mining_end_time_millis: 0,
        },
    ];

    assert_eq!(
        mining_result_unique_areas(&results),
        vec!["Alpha".to_string(), "Beta".to_string()]
    );
}

#[test]
fn mining_result_wallet_deltas_aggregate_net_ore_rewards() {
    let ore_results = vec![
        robominer_db::MiningResultOreStateRecord {
            mining_queue_id: 10,
            ore_id: 1,
            ore_name: "Iron".to_string(),
            amount: 10,
            tax: 1,
            reward: 9,
        },
        robominer_db::MiningResultOreStateRecord {
            mining_queue_id: 11,
            ore_id: 1,
            ore_name: "Iron".to_string(),
            amount: 5,
            tax: 0,
            reward: 5,
        },
        robominer_db::MiningResultOreStateRecord {
            mining_queue_id: 11,
            ore_id: 2,
            ore_name: "Copper".to_string(),
            amount: 3,
            tax: 0,
            reward: 3,
        },
    ];

    assert_eq!(
        mining_result_wallet_deltas(&ore_results),
        vec![("Copper".to_string(), 3), ("Iron".to_string(), 14)]
    );
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

    assert!(empty_html.contains(r#"class="mining-results-empty""#));
    assert!(empty_html.contains(r#"href="miningQueue">Check the mining queue</a>"#));
    assert!(empty_html.contains(
        r#"class="mining-results-claim-banner"><span class="claim-banner-label">Added to wallet:</span>"#
    ));
    assert!(empty_html.contains(r#"class="claim-banner-reward-amount">+18</span>"#));
    assert!(empty_html.contains(r#"class="claim-banner-reward-amount">+9</span>"#));
    assert!(!empty_html.contains("Claimed 2 mining result(s) into your wallet"));
    assert!(!empty_html.contains(r#"class="mining-results-run-card""#));
}

#[test]
fn selected_mining_queue_id_prefers_valid_run_from_url() {
    let results = vec![
        robominer_db::MiningResultStateRecord {
            robot_id: 1,
            mining_queue_id: 10,
            mining_area_name: "A".to_string(),
            rally_result_id: None,
            score: 1.0,
            total_ore_mined: 1,
            total_tax: 0,
            total_reward: 1,
            creation_time_millis: 0,
            mining_end_time_millis: 0,
        },
        robominer_db::MiningResultStateRecord {
            robot_id: 1,
            mining_queue_id: 11,
            mining_area_name: "B".to_string(),
            rally_result_id: None,
            score: 2.0,
            total_ore_mined: 2,
            total_tax: 0,
            total_reward: 2,
            creation_time_millis: 0,
            mining_end_time_millis: 0,
        },
    ];

    assert_eq!(selected_mining_queue_id(&results, Some(11)), Some(11));
    assert_eq!(selected_mining_queue_id(&results, Some(99)), Some(10));
    assert_eq!(selected_mining_queue_id(&results, None), Some(10));
}
