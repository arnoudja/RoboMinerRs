use std::collections::HashMap;
use std::path::PathBuf;

use crate::session::format_authenticated_cookie;
use crate::{Request, ServerConfig};

use super::render::{format_queue_time_left, render_mining_queue_page};
use super::{
    MiningQueueDisplayItem, MiningQueuePageState, cancel_mining_rejection_message,
    enqueue_mining_rejection_message, mining_queue_page,
};

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

#[test]
fn mining_queue_requires_database_configuration() {
    let config = ServerConfig {
        static_root: PathBuf::from("robominer-web/static"),
        database_pool: None,
        allow_signup: true,
    };

    let response = mining_queue_page(&authenticated_request("/miningQueue"), &config);
    let body = String::from_utf8(response.body).expect("message should be utf-8");

    assert_eq!(response.status, 503);
    assert!(body.contains("ROBOMINER_DATABASE_URL"));
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

    assert!(!html.contains(r#"<script src="js/miningqueue.js"></script>"#));
    assert!(html.contains("Bot &lt;One&gt;"));
    assert!(html.contains("Area &amp; Two"));
    assert!(html.contains("Area &lt;Queued&gt;"));
    assert!(html.contains(r#"mining-queue-status-queued">Waiting for rally</span>"#));
    assert!(html.contains(r#"onclick="if(window.miningQueueRemoveRun){window.miningQueueRemoveRun(this);} return false;""#));
    assert!(
        html.contains(r#"type="button" class="mining-queue-remove-btn" data-queue-item-id="101""#)
    );
    assert!(html.contains(r#"aria-label="Remove queued run in Area &lt;Queued&gt;""#));
    assert!(html.contains(r#"class="mining-queue-remove-icon""#));
    assert!(html.contains(r#"<input type="hidden" name="robotId" value="1"/>"#));
    assert!(html.contains(r#"name="submitType" value="add">Add to queue</button>"#));
    assert!(html.contains(r#"name="submitType" value="fill">Fill queue</button>"#));
    assert!(html.contains(r#"class="mining-queue-deck""#));
    assert!(!html.contains("slots used"));
    assert!(!html.contains("runs per robot"));
    assert!(html.contains("Fill queue adds runs until this robot's slots are full."));
    assert!(html.contains("function removeQueuedRun(button)"));
    assert!(html.contains("function submitQueuedRunRemoval(form, queueItemId)"));
    assert!(html.contains("function areaNameOverflows(area)"));
    assert!(html.contains("function syncQueuedStatusVisibility(row)"));
    assert!(html.contains("mining-queue-status-compact-hidden"));
    assert!(html.contains("function observeQueuedStatusVisibility()"));
    assert!(html.contains("window.miningQueueRemoveRun = removeQueuedRun"));
    assert!(html.contains("typeof window.robominerConfirm === 'function'"));
    assert!(html.contains("try {"));
    assert!(html.contains("restoreAreaSelectionsFromStorage();"));
    assert!(!html.contains(r#"name="submitType" value="remove">Remove selected</button>"#));
    assert!(!html.contains(r#"name="selectedQueueItemId" value="101" checked"#));
    assert!(html.contains(
        r#"id="miningArea1" name="miningArea1" class="tableitem mining-queue-area-select""#
    ));
    assert!(html.contains(r#"<input type="hidden" name="infoMiningAreaId" value="20"/>"#));
    assert!(html.contains(r#"<option value="20" selected>Area &amp; Two</option>"#));
    assert!(html.contains(r#"id="miningAreaDetails20""#));
    assert!(html.contains(r#"id="miningAreaDetails21""#));
    assert!(html.contains(r#"class="mining-queue-area-panel mining-queue-area-panel-active""#));
    assert!(html.contains(r#"class="mining-queue-area-panel mining-queue-area-panel-active"><tr><td colspan="4">Upfront costs:</td></tr>"#));
    assert!(
        html.contains(r#"Upfront costs:</td></tr><tr><td></td><td>Ore &amp; Two:</td><td>30</td>"#)
    );
    assert!(html.contains(
        r#"class="mining-queue-area-panel"><tr><td>Tax rate:</td><td colspan="3">5%</td></tr>"#
    ));
    assert!(!html.contains(r#"<button type="submit">Show details</button>"#));
    assert!(html.contains("function collectQueueQueryParams()"));
    assert!(html.contains("function showMiningAreaDetails(areaId)"));
    assert!(html.contains("function syncInspectorArea(areaId)"));
    assert!(html.contains("inspectorSelect.addEventListener('change'"));
    assert!(html.contains(r#"class="mining-queue-page" data-area-storage-key="#));
    assert!(html.contains("function readStoredAreaSelections()"));
    assert!(html.contains("function restoreAreaSelectionsFromStorage()"));
    assert!(html.contains("function writeStoredAreaSelections()"));
    assert!(html.contains("window.sessionStorage.setItem(STORAGE_KEY"));
    assert!(html.contains(r#"class="mining-queue-wallet""#));
    assert!(html.contains(r#"class="mining-queue-card""#));
    assert!(html.contains(
        r#">Ore &amp; Two</span><span class="mining-queue-wallet-amount">40/100</span>"#
    ));
    assert!(html.contains(r#"href="robot?robotId=1">Bot &lt;One&gt;</a>"#));
    assert!(html.contains(r#"mining-queue-status-mining">Mining</span>"#));
    assert!(html.contains(
        r#"data-seconds-left="60" data-refresh-on-complete="true" data-progress-total="120""#
    ));
    assert!(html.contains(r#"class="mining-queue-progress-bar" style="width: 50.0%""#));
    assert!(html.contains(r#"href="miningResults?rallyResultId=55">Area &amp; Two</a>"#));
    assert!(html.contains(r#"class="mining-queue-claim-banner"><span class="claim-banner-label">Added to wallet:</span>"#));
    assert!(html.contains(r#"class="claim-banner-reward-amount">+9</span>"#));
    assert!(html.contains("Ore &amp; Two"));
    assert!(html.contains(r#"data-seconds-left="180""#));
    assert!(html.contains("function formatTimeLeft(seconds)"));
    assert!(html.contains("function refreshQueue()"));
    assert!(html.contains("function startTimer(cell)"));
    assert!(html.contains("encodeURIComponent(select.name)"));
    assert!(html.contains(
            r#"document.querySelectorAll('select[name="infoMiningAreaId"], select[name^="miningArea"]')"#
        ));
    assert!(html.contains("if (seconds <= 0)"));
    assert!(
        html.contains(
            r#"window.location.replace(query ? 'miningQueue?' + query : 'miningQueue');"#
        )
    );
    assert!(html.contains(">1:00<"));
    assert!(html.contains(">3:00<"));
    assert!(html.contains(r#"class="sufficientbalance">(40)"#));
    assert!(html.contains(">45.7<"));
    assert!(html.contains(">12.3%<"));
    assert!(html.contains(">Unable &lt;queue&gt;<"));
    assert!(html.contains(
            r#"class="buttonlink mining-queue-overview-link" href="miningAreaOverview">Compare all areas</a>"#
        ));
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

    assert!(html.contains(r#"name="submitType" value="add" disabled"#));
    assert!(html.contains(r#"name="submitType" value="fill" disabled"#));
    assert!(html.contains("Need 20 more Iron."));
    assert!(html.contains(r#"data-block-reason="Need 20 more Iron.""#));
    assert!(html.contains(r#"class="page-help-hint""#));
    assert!(html.contains(r#"href="helpTutorial?step=1">Follow the step-by-step tutorial</a>"#));
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

    assert!(html.contains(
        r#"<option value="20" selected data-block-reason="Need 10 more Iron.">Expensive</option>"#
    ));
    assert!(html.contains(r#"<option value="21">Affordable</option>"#));
    assert!(!html.contains(r#"value="21" data-block-reason="#));
    assert!(html.contains("function updateRobotEnqueueState(select)"));
    assert!(html.contains(r#"class="mining-queue-action-hint">Need 10 more Iron.</p>"#));
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

    assert!(html.contains(r#"class="mining-queue-empty mining-queue-no-robots""#));
    assert!(html.contains(r#"href="shop">Visit the shop</a>"#));
    assert!(!html.contains(r#"class="page-help-hint""#));
    assert!(!html.contains(r#"class="mining-queue-card""#));
}

#[test]
fn mining_queue_time_left_uses_countdown_format() {
    assert_eq!(format_queue_time_left(0), "0:00");
    assert_eq!(format_queue_time_left(60), "1:00");
    assert_eq!(format_queue_time_left(150), "2:30");
    assert_eq!(format_queue_time_left(3_661), "1:01:01");
}

#[test]
fn mining_queue_rejection_messages_match_legacy_copy() {
    assert_eq!(
        enqueue_mining_rejection_message(
            robominer_db::EnqueueMiningRejection::MiningAreaUnavailable
        ),
        "Unable to add to the mining queue: The mining area is not available."
    );
    assert_eq!(
        enqueue_mining_rejection_message(robominer_db::EnqueueMiningRejection::QueueFull),
        "Unable to add to the mining queue: The mining queue is full."
    );
    assert_eq!(
        enqueue_mining_rejection_message(robominer_db::EnqueueMiningRejection::InsufficientFunds),
        "Unable to add to the mining queue: You do not have enough funds to pay the mining costs."
    );
}

#[test]
fn cancel_mining_rejection_messages_match_legacy_copy() {
    assert_eq!(
        cancel_mining_rejection_message(robominer_db::CancelMiningQueueRejection::UnknownQueue),
        "Unknown mining queue item."
    );
    assert_eq!(
        cancel_mining_rejection_message(robominer_db::CancelMiningQueueRejection::NotCancelable),
        "Unable to cancel mining queue item: The mining queue item is not cancelable."
    );
}
