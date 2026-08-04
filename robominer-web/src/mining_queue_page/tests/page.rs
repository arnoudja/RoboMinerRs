use std::collections::HashMap;
use std::path::PathBuf;

use crate::ServerConfig;
use crate::html::{assert_contains_all, assert_html_contains, assert_html_not_contains};

use super::super::render::render_mining_queue_page;
use super::super::{MiningQueueDisplayItem, MiningQueuePageState, mining_queue_page};
use super::fixtures::authenticated_request;

#[tokio::test(flavor = "current_thread")]
async fn mining_queue_requires_database_configuration() {
    let config = ServerConfig {
        static_root: PathBuf::from("robominer-web/static"),
        database_pool: None,
        allow_signup: true,
        trust_proxy: false,
    };

    let response = mining_queue_page(&authenticated_request("/miningQueue"), &config).await;
    let body = String::from_utf8(response.body).expect("message should be utf-8");

    assert_eq!(response.status, 503);
    assert_html_contains(&body, "ROBOMINER_DATABASE_URL");
}

#[test]
fn mining_queue_rendering_preserves_controls_and_escapes_fields() {
    let mut selected_robot_area_ids = HashMap::new();
    selected_robot_area_ids.insert(1, 20);
    let html = render_mining_queue_page(
        "Player".to_string(),
        None,
        &MiningQueuePageState {
            asset_summary: robominer_db::UserAssetSummaryRecord {
                username: "Player".to_string(),
                achievement_points: 5,
                mining_queue_size: 3,
                robot_count: 1,
            },
            ore_assets: vec![robominer_db::UserOreAssetStateRecord {
                ore_id: 2,
                ore_name: "Ore & Two".to_string(),
                amount: 40,
                max_allowed: 100,
                depot_max_allowed: 0,
            }],
            robots: vec![robominer_db::MiningQueuePageRobotRecord {
                robot_id: 1,
                robot_name: "Bot <One>".to_string(),
                recharge_time: 300,
            }],
            areas: vec![
                robominer_db::MiningQueuePageAreaRecord {
                    mining_area_id: 20,
                    area_name: "Area & Two".to_string(),
                    tax_rate: 12,
                    mining_time: 120,
                    max_moves: 50,
                    size_x: 10,
                    size_y: 11,
                },
                robominer_db::MiningQueuePageAreaRecord {
                    mining_area_id: 21,
                    area_name: "Area Three".to_string(),
                    tax_rate: 5,
                    mining_time: 60,
                    max_moves: 20,
                    size_x: 6,
                    size_y: 7,
                },
            ],
            costs: vec![robominer_db::MiningQueuePageAreaCostRecord {
                mining_area_id: 20,
                ore_id: 2,
                ore_name: "Ore & Two".to_string(),
                amount: 30,
            }],
            supplies: vec![robominer_db::MiningQueuePageAreaSupplyRecord {
                mining_area_id: 20,
                ore_id: 2,
                ore_name: "Ore <Two>".to_string(),
                supply: 8,
                radius: 3,
            }],
            yields: vec![robominer_db::MiningQueuePageAreaYieldRecord {
                mining_area_id: 20,
                ore_id: 2,
                ore_name: "Ore & Two".to_string(),
                percentage: 12.34,
            }],
            scores: vec![robominer_db::RobotMiningAreaScoreRecord {
                robot_id: 1,
                mining_area_id: 20,
                score: 45.67,
            }],
            items: vec![
                MiningQueueDisplayItem {
                    mining_queue_id: 100,
                    robot_id: 1,
                    mining_area_id: 20,
                    area_name: "Area & Two".to_string(),
                    rally_result_id: Some(55),
                    status: robominer_db::MiningQueueStatus::Mining,
                    time_left_seconds: 60,
                },
                MiningQueueDisplayItem {
                    mining_queue_id: 101,
                    robot_id: 1,
                    mining_area_id: 21,
                    area_name: "Area <Queued>".to_string(),
                    rally_result_id: None,
                    status: robominer_db::MiningQueueStatus::Queued,
                    time_left_seconds: 180,
                },
            ],
            selected_info_area_id: 20,
            selected_robot_area_ids,
            error_message: Some("Unable <queue>".to_string()),
            claimed_results: robominer_db::ClaimedUserResults {
                claimed_queues: 2,
                ore_rewards: vec![robominer_db::ClaimedOreRewardRecord {
                    ore_id: 2,
                    ore_name: "Ore & Two".to_string(),
                    reward: 9,
                }],
            },
        },
    );

    assert_contains_all(
        &html,
        &[
            "Bot &lt;One&gt;",
            "Area &amp; Two",
            "Area &lt;Queued&gt;",
            r#"mining-queue-status-queued">Waiting for rally</span>"#,
            r#"type="button" class="mining-queue-remove-btn" data-queue-item-id="101" data-mining-area-id="21""#,
            r#"aria-label="Remove queued run in Area &lt;Queued&gt;""#,
            r#"class="mining-queue-remove-icon""#,
            r#"<input type="hidden" name="robotId" value="1"/>"#,
            r#"name="submitType" value="add">Add to queue</button>"#,
            r#"name="submitType" value="fill">Fill queue</button>"#,
            r#"class="mining-queue-btn mining-queue-clear-btn" data-clearable-count="1">Clear queue</button>"#,
            r#"id="mining-queue-clear-config""#,
            r#""ores":{"2":{"amount":40,"maxAllowed":100}}"#,
            r#""areaCosts":{"20":[{"oreId":2,"amount":30}]}"#,
            r#""initialOreWalletMax":5"#,
            r#"class="mining-queue-deck""#,
            "Fill queue adds runs until this robot's slots are full.",
            r#"src="js/mining_queue/clear_wallet.js?v="#,
            r#"src="js/mining_queue/page.js?v="#,
            r#"id="miningArea1" name="miningArea1" class="tableitem mining-queue-area-select""#,
            r#"<input type="hidden" name="infoMiningAreaId" value="20"/>"#,
            r#"<option value="20" selected>Area &amp; Two</option>"#,
            r#"id="miningAreaDetails20""#,
            r#"id="miningAreaDetails21""#,
            r#"class="mining-queue-area-panel mining-queue-area-panel-active""#,
            r#"class="mining-queue-area-panel mining-queue-area-panel-active"><tr><td colspan="4">Upfront costs:</td></tr>"#,
            r#"Upfront costs:</td></tr><tr><td></td><td>Ore &amp; Two:</td><td>30</td>"#,
            r#"class="mining-queue-area-panel"><tr><td>Tax rate:</td><td colspan="3">5%</td></tr>"#,
            r#"class="mining-queue-page" data-area-storage-key="#,
            r#"class="page-wallet mining-queue-wallet""#,
            r#"class="mining-queue-card""#,
            r#">Ore &amp; Two</span><span class="page-wallet-amount">40/100</span>"#,
            r#"href="robot?robotId=1">Bot &lt;One&gt;</a>"#,
            r#"mining-queue-status-mining">Mining</span>"#,
            r#"data-seconds-left="60" data-refresh-on-complete="true" data-progress-total="120""#,
            r#"class="mining-queue-progress-bar" style="width: 50.0%""#,
            r#"href="miningResults?rallyResultId=55">Area &amp; Two</a>"#,
            r#"class="mining-queue-claim-banner"><span class="claim-banner-label">Added to wallet:</span>"#,
            r#"class="claim-banner-reward-amount">+9</span>"#,
            "Ore &amp; Two",
            r#"data-seconds-left="180""#,
            ">1:00<",
            ">3:00<",
            r#"class="sufficientbalance">(40)"#,
            ">45.7<",
            ">12.3%<",
            ">Unable &lt;queue&gt;<",
            r#"class="buttonlink mining-queue-overview-link" href="miningAreaOverview">Compare all areas</a>"#,
        ],
    );
    for absent in [
        r#"<script src="js/miningqueue.js"></script>"#,
        r#"onclick="if(window.miningQueueRemoveRun)"#,
        "slots used",
        "runs per robot",
        "function removeQueuedRun(button)",
        r#"name="submitType" value="remove">Remove selected</button>"#,
        r#"name="selectedQueueItemId" value="101" checked"#,
        r#"<button type="submit">Show details</button>"#,
    ] {
        assert_html_not_contains(&html, absent);
    }
}

#[test]
fn mining_queue_shows_disabled_enqueue_with_reason() {
    let mut selected_robot_area_ids = HashMap::new();
    selected_robot_area_ids.insert(1, 20);
    let html = render_mining_queue_page(
        "Player".to_string(),
        None,
        &MiningQueuePageState {
            asset_summary: robominer_db::UserAssetSummaryRecord {
                username: "Player".to_string(),
                achievement_points: 0,
                mining_queue_size: 3,
                robot_count: 1,
            },
            ore_assets: vec![robominer_db::UserOreAssetStateRecord {
                ore_id: 2,
                ore_name: "Iron".to_string(),
                amount: 10,
                max_allowed: 100,
                depot_max_allowed: 0,
            }],
            robots: vec![robominer_db::MiningQueuePageRobotRecord {
                robot_id: 1,
                robot_name: "Bot".to_string(),
                recharge_time: 60,
            }],
            areas: vec![robominer_db::MiningQueuePageAreaRecord {
                mining_area_id: 20,
                area_name: "Area".to_string(),
                tax_rate: 0,
                mining_time: 120,
                max_moves: 10,
                size_x: 5,
                size_y: 5,
            }],
            costs: vec![robominer_db::MiningQueuePageAreaCostRecord {
                mining_area_id: 20,
                ore_id: 2,
                ore_name: "Iron".to_string(),
                amount: 30,
            }],
            supplies: vec![],
            yields: vec![],
            scores: vec![],
            items: vec![],
            selected_info_area_id: 20,
            selected_robot_area_ids,
            error_message: None,
            claimed_results: robominer_db::ClaimedUserResults {
                claimed_queues: 0,
                ore_rewards: vec![],
            },
        },
    );

    assert_contains_all(
        &html,
        &[
            r#"name="submitType" value="add" disabled"#,
            r#"name="submitType" value="fill" disabled"#,
            r#"class="mining-queue-btn mining-queue-clear-btn" data-clearable-count="0" disabled"#,
            r#"title="No queued runs to clear""#,
            "Need 20 more Iron.",
            r#"data-block-reason="Need 20 more Iron.""#,
            r#"class="page-help-hint""#,
            r#"href="helpTutorial?step=1">Follow the step-by-step tutorial</a>"#,
        ],
    );
}

#[test]
fn mining_queue_area_options_include_per_area_enqueue_block_reasons() {
    let mut selected_robot_area_ids = HashMap::new();
    selected_robot_area_ids.insert(1, 20);
    let html = render_mining_queue_page(
        "Player".to_string(),
        None,
        &MiningQueuePageState {
            asset_summary: robominer_db::UserAssetSummaryRecord {
                username: "Player".to_string(),
                achievement_points: 0,
                mining_queue_size: 3,
                robot_count: 1,
            },
            ore_assets: vec![robominer_db::UserOreAssetStateRecord {
                ore_id: 2,
                ore_name: "Iron".to_string(),
                amount: 40,
                max_allowed: 100,
                depot_max_allowed: 0,
            }],
            robots: vec![robominer_db::MiningQueuePageRobotRecord {
                robot_id: 1,
                robot_name: "Bot".to_string(),
                recharge_time: 60,
            }],
            areas: vec![
                robominer_db::MiningQueuePageAreaRecord {
                    mining_area_id: 20,
                    area_name: "Expensive".to_string(),
                    tax_rate: 0,
                    mining_time: 120,
                    max_moves: 10,
                    size_x: 5,
                    size_y: 5,
                },
                robominer_db::MiningQueuePageAreaRecord {
                    mining_area_id: 21,
                    area_name: "Affordable".to_string(),
                    tax_rate: 0,
                    mining_time: 60,
                    max_moves: 10,
                    size_x: 5,
                    size_y: 5,
                },
            ],
            costs: vec![robominer_db::MiningQueuePageAreaCostRecord {
                mining_area_id: 20,
                ore_id: 2,
                ore_name: "Iron".to_string(),
                amount: 50,
            }],
            supplies: vec![],
            yields: vec![],
            scores: vec![],
            items: vec![],
            selected_info_area_id: 20,
            selected_robot_area_ids,
            error_message: None,
            claimed_results: robominer_db::ClaimedUserResults {
                claimed_queues: 0,
                ore_rewards: vec![],
            },
        },
    );

    assert_contains_all(
        &html,
        &[
            r#"<option value="20" selected data-block-reason="Need 10 more Iron.">Expensive</option>"#,
            r#"<option value="21">Affordable</option>"#,
            r#"src="js/mining_queue/page.js?v="#,
            r#"class="mining-queue-action-hint">Need 10 more Iron.</p>"#,
        ],
    );
    assert_html_not_contains(&html, r#"value="21" data-block-reason="#);
}

#[test]
fn mining_queue_shows_no_robots_empty_state() {
    let html = render_mining_queue_page(
        "Player".to_string(),
        None,
        &MiningQueuePageState {
            asset_summary: robominer_db::UserAssetSummaryRecord {
                username: "Player".to_string(),
                achievement_points: 0,
                mining_queue_size: 3,
                robot_count: 0,
            },
            ore_assets: vec![],
            robots: vec![],
            areas: vec![],
            costs: vec![],
            supplies: vec![],
            yields: vec![],
            scores: vec![],
            items: vec![],
            selected_info_area_id: 0,
            selected_robot_area_ids: HashMap::new(),
            error_message: None,
            claimed_results: robominer_db::ClaimedUserResults {
                claimed_queues: 0,
                ore_rewards: vec![],
            },
        },
    );

    assert_contains_all(
        &html,
        &[
            r#"class="mining-queue-empty mining-queue-no-robots""#,
            r#"href="shop">Visit the shop</a>"#,
        ],
    );
    assert_html_not_contains(&html, r#"class="page-help-hint""#);
    assert_html_not_contains(&html, r#"class="mining-queue-card""#);
}
