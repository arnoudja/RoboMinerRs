use std::collections::HashMap;
use std::path::PathBuf;

use crate::ServerConfig;
use crate::html::{assert_contains_all, assert_html_contains, assert_html_not_contains};

use super::super::render::{render_mining_queue_fragment, render_mining_queue_page};
use super::super::{
    MiningQueueAreaCostView, MiningQueueAreaSupplyView, MiningQueueAreaView,
    MiningQueueAssetSummaryView, MiningQueueDisplayItem, MiningQueueOreAssetView,
    MiningQueuePageState, MiningQueueRobotView, MiningQueueScoreView, mining_queue_page,
};
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
    let body = response.body_utf8();

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
            asset_summary: MiningQueueAssetSummaryView {
                mining_queue_size: 3,
            },
            ore_assets: vec![MiningQueueOreAssetView {
                ore_id: 2,
                ore_name: "Ore & Two".to_string(),
                amount: 40,
                max_allowed: 100,
                depot_max_allowed: 250,
            }],
            robots: vec![MiningQueueRobotView {
                robot_id: 1,
                robot_name: "Bot <One>".to_string(),
                recharge_time: 300,
            }],
            areas: vec![
                MiningQueueAreaView {
                    mining_area_id: 20,
                    area_name: "Area & Two".to_string(),
                    tax_rate: 12,
                    depot_tax_rate: 6,
                    mining_time: 120,
                    max_moves: 50,
                    size_x: 10,
                    size_y: 11,
                    score_ore_target: 30,
                },
                MiningQueueAreaView {
                    mining_area_id: 21,
                    area_name: "Area Three".to_string(),
                    tax_rate: 5,
                    depot_tax_rate: 2,
                    mining_time: 60,
                    max_moves: 20,
                    size_x: 6,
                    size_y: 7,
                    score_ore_target: 15,
                },
            ],
            costs: vec![MiningQueueAreaCostView {
                mining_area_id: 20,
                ore_id: 2,
                ore_name: "Ore & Two".to_string(),
                amount: 30,
            }],
            supplies: vec![MiningQueueAreaSupplyView {
                mining_area_id: 20,
                ore_id: 2,
                ore_name: "Ore <Two>".to_string(),
                supply: 8,
                radius: 3,
            }],
            scores: vec![MiningQueueScoreView {
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
            r#"class="mining-queue-item-check" data-queue-item-id="101" data-mining-area-id="21""#,
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
            r#"class="mining-queue-area-panel"><tr><td>Container tax:</td><td colspan="3">5%</td></tr>
<tr><td>Depot tax:</td><td colspan="3">2%</td></tr>"#,
            r#"Turns:</td><td colspan="3">50</td></tr>
<tr><td>Area size:</td><td colspan="3">10 x 11</td></tr>
<tr><td>Mining target:</td><td colspan="3">30 ore</td></tr>
<tr><td colspan="4">Estimated ore:</td></tr>"#,
            r#"Ore &lt;Two&gt;:</td><td colspan="2">72</td></tr>"#,
            r#"class="mining-queue-page" data-area-storage-key="#,
            r#"class="page-wallet mining-queue-wallet""#,
            r#"class="mining-queue-card" data-robot-id="1""#,
            r#"class="mining-queue-card-status""#,
            r#">Ore &amp; Two</span><span class="page-wallet-amount">40/100</span></div><span class="page-wallet-depot">depot 250</span>"#,
            r#"href="robot?robotId=1">Bot &lt;One&gt;</a>"#,
            r#"mining-queue-status-mining">Mining</span>"#,
            r#"data-seconds-left="61" data-refresh-on-complete="true" data-progress-total="121""#,
            r#"class="mining-queue-progress" value="49.6" max="100""#,
            r#"href="miningResults?rallyResultId=55">Area &amp; Two</a>"#,
            "Ore &amp; Two",
            r#"data-seconds-left="180""#,
            ">1:01<",
            ">3:00<",
            r#"class="sufficientbalance">(40)"#,
            ">45.7<",
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
        "Historic yield:",
        ">12.3%<",
    ] {
        assert_html_not_contains(&html, absent);
    }
}

#[test]
fn mining_queue_estimated_ore_sums_heaps_of_same_ore_type() {
    let mut selected_robot_area_ids = HashMap::new();
    selected_robot_area_ids.insert(1, 20);
    let html = render_mining_queue_page(
        "Player".to_string(),
        None,
        &MiningQueuePageState {
            asset_summary: MiningQueueAssetSummaryView {
                mining_queue_size: 3,
            },
            ore_assets: vec![],
            robots: vec![MiningQueueRobotView {
                robot_id: 1,
                robot_name: "Bot".to_string(),
                recharge_time: 60,
            }],
            areas: vec![MiningQueueAreaView {
                mining_area_id: 20,
                area_name: "Area".to_string(),
                tax_rate: 0,
                depot_tax_rate: 0,
                mining_time: 120,
                max_moves: 10,
                size_x: 5,
                size_y: 5,
                score_ore_target: 30,
            }],
            costs: vec![],
            supplies: vec![
                MiningQueueAreaSupplyView {
                    mining_area_id: 20,
                    ore_id: 2,
                    ore_name: "Iron".to_string(),
                    supply: 8,
                    radius: 3,
                },
                MiningQueueAreaSupplyView {
                    mining_area_id: 20,
                    ore_id: 3,
                    ore_name: "Copper".to_string(),
                    supply: 10,
                    radius: 2,
                },
                MiningQueueAreaSupplyView {
                    mining_area_id: 20,
                    ore_id: 2,
                    ore_name: "Iron".to_string(),
                    supply: 5,
                    radius: 2,
                },
            ],
            scores: vec![],
            items: vec![],
            selected_info_area_id: 20,
            selected_robot_area_ids,
            error_message: None,
        },
    );

    // 8@r3 => 72, 5@r2 => 21, same ore_id summed; 10@r2 => 42 kept separate.
    // First-seen order: Iron then Copper.
    assert_contains_all(
        &html,
        &[
            r#"<tr><td colspan="4">Estimated ore:</td></tr><tr><td></td><td>Iron:</td><td colspan="2">93</td></tr><tr><td></td><td>Copper:</td><td colspan="2">42</td></tr>"#,
        ],
    );
    assert_html_not_contains(&html, r#"Iron:</td><td colspan="2">72</td></tr>"#);
    assert_html_not_contains(&html, r#"Iron:</td><td colspan="2">21</td></tr>"#);
}

#[test]
fn mining_queue_shows_disabled_enqueue_with_reason() {
    let mut selected_robot_area_ids = HashMap::new();
    selected_robot_area_ids.insert(1, 20);
    let html = render_mining_queue_page(
        "Player".to_string(),
        None,
        &MiningQueuePageState {
            asset_summary: MiningQueueAssetSummaryView {
                mining_queue_size: 3,
            },
            ore_assets: vec![MiningQueueOreAssetView {
                ore_id: 2,
                ore_name: "Iron".to_string(),
                amount: 10,
                max_allowed: 100,
                depot_max_allowed: 0,
            }],
            robots: vec![MiningQueueRobotView {
                robot_id: 1,
                robot_name: "Bot".to_string(),
                recharge_time: 60,
            }],
            areas: vec![MiningQueueAreaView {
                mining_area_id: 20,
                area_name: "Area".to_string(),
                tax_rate: 0,
                depot_tax_rate: 0,
                mining_time: 120,
                max_moves: 10,
                size_x: 5,
                size_y: 5,
                score_ore_target: 30,
            }],
            costs: vec![MiningQueueAreaCostView {
                mining_area_id: 20,
                ore_id: 2,
                ore_name: "Iron".to_string(),
                amount: 30,
            }],
            supplies: vec![],
            scores: vec![],
            items: vec![],
            selected_info_area_id: 20,
            selected_robot_area_ids,
            error_message: None,
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
            asset_summary: MiningQueueAssetSummaryView {
                mining_queue_size: 3,
            },
            ore_assets: vec![MiningQueueOreAssetView {
                ore_id: 2,
                ore_name: "Iron".to_string(),
                amount: 40,
                max_allowed: 100,
                depot_max_allowed: 0,
            }],
            robots: vec![MiningQueueRobotView {
                robot_id: 1,
                robot_name: "Bot".to_string(),
                recharge_time: 60,
            }],
            areas: vec![
                MiningQueueAreaView {
                    mining_area_id: 20,
                    area_name: "Expensive".to_string(),
                    tax_rate: 0,
                    depot_tax_rate: 0,
                    mining_time: 120,
                    max_moves: 10,
                    size_x: 5,
                    size_y: 5,
                    score_ore_target: 30,
                },
                MiningQueueAreaView {
                    mining_area_id: 21,
                    area_name: "Affordable".to_string(),
                    tax_rate: 0,
                    depot_tax_rate: 0,
                    mining_time: 60,
                    max_moves: 10,
                    size_x: 5,
                    size_y: 5,
                    score_ore_target: 30,
                },
            ],
            costs: vec![MiningQueueAreaCostView {
                mining_area_id: 20,
                ore_id: 2,
                ore_name: "Iron".to_string(),
                amount: 50,
            }],
            supplies: vec![],
            scores: vec![],
            items: vec![],
            selected_info_area_id: 20,
            selected_robot_area_ids,
            error_message: None,
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
            asset_summary: MiningQueueAssetSummaryView {
                mining_queue_size: 3,
            },
            ore_assets: vec![],
            robots: vec![],
            areas: vec![],
            costs: vec![],
            supplies: vec![],
            scores: vec![],
            items: vec![],
            selected_info_area_id: 0,
            selected_robot_area_ids: HashMap::new(),
            error_message: None,
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

#[test]
fn mining_queue_fragment_renders_dynamic_sections_without_inspector() {
    let mut selected_robot_area_ids = HashMap::new();
    selected_robot_area_ids.insert(1, 20);
    let state = MiningQueuePageState {
        asset_summary: MiningQueueAssetSummaryView {
            mining_queue_size: 3,
        },
        ore_assets: vec![MiningQueueOreAssetView {
            ore_id: 2,
            ore_name: "Iron".to_string(),
            amount: 40,
            max_allowed: 100,
            depot_max_allowed: 250,
        }],
        robots: vec![MiningQueueRobotView {
            robot_id: 1,
            robot_name: "Bot".to_string(),
            recharge_time: 300,
        }],
        areas: vec![MiningQueueAreaView {
            mining_area_id: 20,
            area_name: "Area".to_string(),
            tax_rate: 0,
            depot_tax_rate: 0,
            mining_time: 120,
            max_moves: 10,
            size_x: 5,
            size_y: 5,
            score_ore_target: 30,
        }],
        costs: vec![],
        supplies: vec![],
        scores: vec![],
        items: vec![MiningQueueDisplayItem {
            mining_queue_id: 10,
            robot_id: 1,
            mining_area_id: 20,
            area_name: "Area".to_string(),
            rally_result_id: Some(55),
            status: robominer_db::MiningQueueStatus::Mining,
            time_left_seconds: 60,
        }],
        selected_info_area_id: 20,
        selected_robot_area_ids,
        error_message: Some("Queue full for this robot.".to_string()),
    };
    let hud = r#"<div class="app-shell-hud"><a class="app-shell-hud-item">2/6</a></div>"#;
    let html = render_mining_queue_fragment(hud, &state);

    assert_contains_all(
        &html,
        &[
            r#"id="mining-queue-fragment""#,
            r#"id="mining-queue-hud-fragment""#,
            r#"id="mining-queue-dynamic-fragment""#,
            r#"id="mining-queue-robots-fragment""#,
            r#"class="page-wallet mining-queue-wallet""#,
            r#"class="mining-queue-card" data-robot-id="1""#,
            r#"class="mining-queue-card-status""#,
            r#"class="error mining-queue-error">Queue full for this robot."#,
            r#"id="mining-queue-clear-config""#,
            r#"app-shell-hud-item">2/6</a>"#,
        ],
    );
    assert_html_not_contains(&html, "mining-queue-inspector");
    assert_html_not_contains(&html, "miningAreaDetails");
    assert_html_not_contains(&html, "<!DOCTYPE html>");
}
