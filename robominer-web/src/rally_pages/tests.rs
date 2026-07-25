use std::collections::HashMap;
use std::path::PathBuf;

use crate::html::{
    assert_contains_all, assert_html_contains, assert_html_has_class, assert_html_not_contains,
};
use crate::{Request, ServerConfig};

use super::{
    ACTIVITY_RALLY_PAGE_SIZE, ActivityFeedQuery, ActivityPageState, ActivityRallyFilter,
    RallyViewBackLink, RallyViewPageState, activity_page, render_activity_page_at,
    render_rally_view_page, valid_mining_results_return_to,
};

fn sample_activity_state(
    recent_users: Vec<robominer_db::ActivityRecentUserRecord>,
    recent_rallies: Vec<robominer_db::ActivityRecentRallyRecord>,
    participants: Vec<robominer_db::ActivityRecentRallyParticipantRecord>,
    rally_areas: Vec<robominer_db::ActivityRallyAreaOption>,
    has_more_rallies: bool,
) -> ActivityPageState {
    sample_activity_state_with_queue(
        recent_users,
        recent_rallies,
        participants,
        rally_areas,
        has_more_rallies,
        vec![],
        None,
    )
}

fn sample_activity_state_with_queue(
    recent_users: Vec<robominer_db::ActivityRecentUserRecord>,
    recent_rallies: Vec<robominer_db::ActivityRecentRallyRecord>,
    participants: Vec<robominer_db::ActivityRecentRallyParticipantRecord>,
    rally_areas: Vec<robominer_db::ActivityRallyAreaOption>,
    has_more_rallies: bool,
    queue_items: Vec<robominer_db::MiningQueuePageItemRecord>,
    asset_summary: Option<robominer_db::UserAssetSummaryRecord>,
) -> ActivityPageState {
    ActivityPageState {
        recent_users,
        recent_rallies,
        participants,
        rally_areas,
        has_more_rallies,
        queue_items,
        asset_summary,
    }
}

fn default_activity_feed_query() -> ActivityFeedQuery {
    ActivityFeedQuery {
        filter: ActivityRallyFilter::All,
        area_id: None,
        limit: ACTIVITY_RALLY_PAGE_SIZE,
    }
}

fn sample_rally_view_state(slots: [(String, String); 4]) -> RallyViewPageState {
    RallyViewPageState {
        result_data: r#"{"v":1,"robots":{"robot":[]},"ground":{"sizeX":1,"sizeY":1,"positions":[]},"oreTypes":{}}"#.to_string(),
        ores: Vec::new(),
        slots,
        mining_area_name: "Area & One".to_string(),
        viewer_player_number: None,
        viewer_robot_id: None,
        viewer_robot_name: None,
        viewer_score: None,
        viewer_total_reward: None,
        viewer_result_claimed: false,
        viewer_source_code: None,
        viewer_program_source_id: None,
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
async fn activity_requires_database_configuration() {
    let config = ServerConfig {
        static_root: PathBuf::from("robominer-web/static"),
        database_pool: None,
        allow_signup: true,
        trust_proxy: false,
    };

    let response = activity_page(&request("/activity"), &config).await;
    let body = String::from_utf8(response.body).expect("message should be utf-8");

    assert_eq!(response.status, 503);
    assert_html_contains(&body, "ROBOMINER_DATABASE_URL");
}

#[test]
fn activity_rendering_groups_participants_and_formats_utc_dates() {
    let now_millis = 3_600_000;
    let html = render_activity_page_at(
        "Player".to_string(),
        None,
        &sample_activity_state(
            vec![robominer_db::ActivityRecentUserRecord {
                user_id: 1,
                username: "User <A>".to_string(),
                last_login_time_millis: 0,
            }],
            vec![robominer_db::ActivityRecentRallyRecord {
                mining_queue_id: 10,
                rally_result_id: Some(20),
                mining_area_id: 1,
                mining_area_name: "Area & One".to_string(),
                robot_name: "Lead <Bot>".to_string(),
                username: "Owner".to_string(),
                mining_end_time_millis: 0,
            }],
            vec![
                robominer_db::ActivityRecentRallyParticipantRecord {
                    mining_queue_id: 10,
                    player_number: 0,
                    robot_name: "Lead <Bot>".to_string(),
                    username: "Owner".to_string(),
                },
                robominer_db::ActivityRecentRallyParticipantRecord {
                    mining_queue_id: 10,
                    player_number: 1,
                    robot_name: "Other & Bot".to_string(),
                    username: "Other <Owner>".to_string(),
                },
            ],
            vec![robominer_db::ActivityRallyAreaOption {
                mining_area_id: 1,
                area_name: "Area & One".to_string(),
            }],
            false,
        ),
        now_millis,
        default_activity_feed_query(),
    );

    assert_contains_all(
        &html,
        &[
            r#"class="activity-page""#,
            r#"class="activity-title">Activity</h1>"#,
            r#"class="page-help-hint""#,
            r#"href="helpTutorial?step=1">Tutorial</a>"#,
            "Area &amp; One",
            "Lead &lt;Bot&gt;",
            r#"class="activity-rally-card activity-rally-card-replayable""#,
            r#"href="activity">All rallies</a>"#,
            r#"href="activity?filter=mine">Your rallies</a>"#,
            r#"href="activity?rallyResultId=20""#,
            r#"class="activity-rally-badge activity-rally-badge-replay">Replay ready</span>"#,
            r#"class="activity-rally-badge activity-rally-badge-players">2 players</span>"#,
            r#"class="activity-rally-replay-cta">Watch replay"#,
            "activity-rally-participant-1",
            "Other &amp; Bot",
            "Other &lt;Owner&gt;",
            "User &lt;A&gt;",
            r#"title="1970-01-01 00:00:00 UTC">Ended 1 hour ago</p>"#,
            r#"title="1970-01-01 00:00:00 UTC">1 hour ago</span>"#,
            r#"class="activity-feed-stats""#,
            r#"<dt>Showing</dt><dd>1 rallies</dd>"#,
            r#"<dt>Replays</dt><dd>1 ready</dd>"#,
            r#"class="tableitem activity-area-filter-select""#,
            "Area &amp; One</option>",
            r#"class="activity-sidebar""#,
            "activity-sidebar-players",
            r#"class="activity-deck""#,
        ],
    );
    assert_html_has_class(&html, "activity-rally-filter-link-active");
    assert_html_not_contains(&html, r#"class="activity-deck activity-deck-full""#);
}

#[test]
fn activity_rendering_shows_empty_states_and_unavailable_replay() {
    let html = render_activity_page_at(
        "Player".to_string(),
        None,
        &sample_activity_state(
            vec![],
            vec![robominer_db::ActivityRecentRallyRecord {
                mining_queue_id: 11,
                rally_result_id: None,
                mining_area_id: 2,
                mining_area_name: "Quiet Mine".to_string(),
                robot_name: "Solo Bot".to_string(),
                username: "Solo User".to_string(),
                mining_end_time_millis: 0,
            }],
            vec![],
            vec![],
            false,
        ),
        3_600_000,
        default_activity_feed_query(),
    );

    assert_html_not_contains(&html, "activity-sidebar-players");
    assert_html_not_contains(&html, "No recent players to show.");
    assert_contains_all(
        &html,
        &[
            r#"class="activity-sidebar""#,
            r#"class="activity-deck""#,
            r#"class="activity-rally-card activity-rally-card-unavailable""#,
            r#"class="activity-rally-badge activity-rally-badge-unavailable""#,
            "No replay stored</span>",
            r#"class="activity-rally-badge activity-rally-badge-players">Solo</span>"#,
        ],
    );
    assert_html_not_contains(&html, r#"class="activity-deck activity-deck-full""#);
    assert_html_not_contains(&html, "Watch replay");
}

#[test]
fn activity_rendering_shows_actionable_empty_rally_state() {
    let html = render_activity_page_at(
        "Player".to_string(),
        None,
        &sample_activity_state(vec![], vec![], vec![], vec![], false),
        0,
        default_activity_feed_query(),
    );

    assert_contains_all(
        &html,
        &[
            "No finished rallies yet.",
            r#"href="miningQueue">Add runs to your mining queue</a>"#,
            r#"href="helpTutorial?step=1">follow the tutorial</a>"#,
        ],
    );
}

#[test]
fn activity_rendering_highlights_viewer_participation() {
    let html = render_activity_page_at(
        "Player".to_string(),
        None,
        &sample_activity_state(
            vec![],
            vec![robominer_db::ActivityRecentRallyRecord {
                mining_queue_id: 10,
                rally_result_id: Some(20),
                mining_area_id: 1,
                mining_area_name: "Shared Mine".to_string(),
                robot_name: "Lead Bot".to_string(),
                username: "Player".to_string(),
                mining_end_time_millis: 0,
            }],
            vec![robominer_db::ActivityRecentRallyParticipantRecord {
                mining_queue_id: 10,
                player_number: 1,
                robot_name: "Other Bot".to_string(),
                username: "Other Owner".to_string(),
            }],
            vec![],
            false,
        ),
        3_600_000,
        default_activity_feed_query(),
    );

    assert_contains_all(
        &html,
        &[
            r#"class="activity-rally-badge activity-rally-badge-self">You played</span>"#,
            r#"class="activity-rally-participant activity-rally-participant-0 activity-rally-participant-self""#,
            r#"class="activity-rally-participant-you">You</span>"#,
        ],
    );
}

#[test]
fn activity_rendering_shows_your_rallies_filter_and_empty_state() {
    let html = render_activity_page_at(
        "Player".to_string(),
        None,
        &sample_activity_state(vec![], vec![], vec![], vec![], false),
        0,
        ActivityFeedQuery {
            filter: ActivityRallyFilter::Mine,
            area_id: None,
            limit: ACTIVITY_RALLY_PAGE_SIZE,
        },
    );

    assert_contains_all(
        &html,
        &[
            r#"href="activity">All rallies</a>"#,
            r#"href="activity?filter=mine">Your rallies</a>"#,
            "No rallies you've joined yet.",
        ],
    );
    assert_html_has_class(&html, "activity-rally-filter-link-active");
}

#[test]
fn activity_mine_filter_preserves_replay_link() {
    let html = render_activity_page_at(
        "Player".to_string(),
        None,
        &sample_activity_state(
            vec![],
            vec![robominer_db::ActivityRecentRallyRecord {
                mining_queue_id: 10,
                rally_result_id: Some(20),
                mining_area_id: 1,
                mining_area_name: "Shared Mine".to_string(),
                robot_name: "Lead Bot".to_string(),
                username: "Player".to_string(),
                mining_end_time_millis: 0,
            }],
            vec![],
            vec![],
            false,
        ),
        3_600_000,
        ActivityFeedQuery {
            filter: ActivityRallyFilter::Mine,
            area_id: None,
            limit: ACTIVITY_RALLY_PAGE_SIZE,
        },
    );

    assert_html_contains(&html, r#"href="activity?rallyResultId=20&amp;filter=mine""#);
}

#[test]
fn activity_rendering_shows_load_more_and_area_filter_links() {
    let html = render_activity_page_at(
        "Player".to_string(),
        None,
        &sample_activity_state(
            vec![],
            vec![robominer_db::ActivityRecentRallyRecord {
                mining_queue_id: 10,
                rally_result_id: Some(20),
                mining_area_id: 5,
                mining_area_name: "Crystal Cave".to_string(),
                robot_name: "Lead Bot".to_string(),
                username: "Owner".to_string(),
                mining_end_time_millis: 0,
            }],
            vec![],
            vec![robominer_db::ActivityRallyAreaOption {
                mining_area_id: 5,
                area_name: "Crystal Cave".to_string(),
            }],
            true,
        ),
        3_600_000,
        ActivityFeedQuery {
            filter: ActivityRallyFilter::All,
            area_id: Some(5),
            limit: ACTIVITY_RALLY_PAGE_SIZE,
        },
    );

    assert_contains_all(
        &html,
        &[
            r#"href="activity?areaId=5&amp;limit=20">Load more rallies</a>"#,
            r#"value="activity?areaId=5" selected>Crystal Cave</option>"#,
        ],
    );
}

#[test]
fn activity_rendering_shows_area_specific_empty_state() {
    let html = render_activity_page_at(
        "Player".to_string(),
        None,
        &sample_activity_state(vec![], vec![], vec![], vec![], false),
        0,
        ActivityFeedQuery {
            filter: ActivityRallyFilter::All,
            area_id: Some(5),
            limit: ACTIVITY_RALLY_PAGE_SIZE,
        },
    );

    assert_html_contains(&html, "No finished rallies in this area yet.");
}

#[test]
fn activity_rendering_shows_sidebar_queue_snapshot() {
    let html = render_activity_page_at(
        "Player".to_string(),
        None,
        &sample_activity_state_with_queue(
            vec![],
            vec![],
            vec![],
            vec![],
            false,
            vec![
                robominer_db::MiningQueuePageItemRecord {
                    mining_queue_id: 1,
                    robot_id: 7,
                    mining_area_id: 3,
                    area_name: "Crystal & Cave".to_string(),
                    rally_result_id: None,
                },
                robominer_db::MiningQueuePageItemRecord {
                    mining_queue_id: 2,
                    robot_id: 7,
                    mining_area_id: 4,
                    area_name: "Dust Bowl".to_string(),
                    rally_result_id: None,
                },
            ],
            Some(robominer_db::UserAssetSummaryRecord {
                username: "Player".to_string(),
                achievement_points: 0,
                mining_queue_size: 3,
                robot_count: 1,
            }),
        ),
        0,
        default_activity_feed_query(),
    );

    assert_contains_all(
        &html,
        &[
            r#"class="activity-section-title">Your mining queue</h2>"#,
            "2/3 slots in use",
            r#"href="miningQueue?robotId=7">Crystal &amp; Cave</a>"#,
            r#"class="activity-queue-manage" href="miningQueue">Manage queue</a>"#,
        ],
    );
}

#[test]
fn rally_view_rendering_refuses_legacy_javascript_result_data() {
    let html = render_rally_view_page(
        "Player".to_string(),
        None,
        &RallyViewPageState {
            result_data: "var myOreTypes = {};".to_string(),
            ores: vec![],
            slots: [
                ("Bot 0".to_string(), "User 0".to_string()),
                ("Bot 1".to_string(), "User 1".to_string()),
                ("Bot 2".to_string(), "User 2".to_string()),
                ("Bot 3".to_string(), "User 3".to_string()),
            ],
            mining_area_name: "Area".to_string(),
            viewer_player_number: None,
            viewer_robot_id: None,
            viewer_robot_name: None,
            viewer_score: None,
            viewer_total_reward: None,
            viewer_result_claimed: false,
            viewer_source_code: None,
            viewer_program_source_id: None,
        },
        None,
    );

    for absent in [
        r#"id="rally-result-data""#,
        "var myOreTypes = {};",
        "applyRallyResultPayload(JSON.parse",
        "function runanimation(",
        r#"id="rallyCanvas""#,
    ] {
        assert_html_not_contains(&html, absent);
    }
    assert_contains_all(
        &html,
        &[
            "Replay unavailable",
            "This rally was stored in an older executable format that is no longer played for security reasons.",
        ],
    );
}

#[test]
fn rally_view_rendering_refuses_unsupported_result_data() {
    let html = render_rally_view_page(
        "Player".to_string(),
        None,
        &RallyViewPageState {
            result_data: r#"{"v":3,"robots":{"robot":[]},"ground":{"sizeX":1,"sizeY":1,"positions":[]},"oreTypes":{}}"#.to_string(),
            ores: vec![],
            slots: [
                ("Bot 0".to_string(), "User 0".to_string()),
                ("Bot 1".to_string(), "User 1".to_string()),
                ("Bot 2".to_string(), "User 2".to_string()),
                ("Bot 3".to_string(), "User 3".to_string()),
            ],
            mining_area_name: "Area".to_string(),
            viewer_player_number: None,
            viewer_robot_id: None,
            viewer_robot_name: None,
            viewer_score: None,
            viewer_total_reward: None,
            viewer_result_claimed: false,
            viewer_source_code: None,
            viewer_program_source_id: None,
        },
        None,
    );

    assert_html_not_contains(&html, r#"id="rally-result-data""#);
    assert_html_not_contains(&html, r#""v":3"#);
    assert_contains_all(
        &html,
        &[
            "Replay unavailable",
            "This rally replay payload is missing, corrupt, or uses an unsupported version.",
        ],
    );
}

#[test]
fn rally_view_rendering_accepts_v2_result_data() {
    let html = render_rally_view_page(
        "Player".to_string(),
        None,
        &RallyViewPageState {
            result_data: r#"{"v":2,"robots":{"robot":[{"robotnr":0,"locations":[{"x":0,"y":0}]}]},"ground":{"sizeX":1,"sizeY":1,"positions":[]},"oreTypes":{}}"#.to_string(),
            ores: vec![],
            slots: [
                ("Bot 0".to_string(), "User 0".to_string()),
                ("Bot 1".to_string(), "User 1".to_string()),
                ("Bot 2".to_string(), "User 2".to_string()),
                ("Bot 3".to_string(), "User 3".to_string()),
            ],
            mining_area_name: "Area".to_string(),
            viewer_player_number: Some(0),
            viewer_robot_id: Some(11),
            viewer_robot_name: Some("Bot".to_string()),
            viewer_score: Some(1.0),
            viewer_total_reward: Some(1),
            viewer_result_claimed: false,
            viewer_source_code: None,
            viewer_program_source_id: None,
        },
        None,
    );

    assert_contains_all(
        &html,
        &[
            r#"id="rally-result-data""#,
            r#""v":2"#,
            r#"id="rallyCanvas""#,
        ],
    );
    assert_html_not_contains(
        &html,
        r#"<p class="rally-view-replay-unavailable-title">Replay unavailable</p>"#,
    );
}

#[test]
fn rally_view_rendering_refuses_incomplete_versioned_result_data() {
    let html = render_rally_view_page(
        "Player".to_string(),
        None,
        &RallyViewPageState {
            result_data:
                r#"{"v":1,"robots":{"robot":[{"robotnr":0}]},"ground":{"sizeX":1,"sizeY":1}}"#
                    .to_string(),
            ores: vec![],
            slots: [
                ("Bot 0".to_string(), "User 0".to_string()),
                ("Bot 1".to_string(), "User 1".to_string()),
                ("Bot 2".to_string(), "User 2".to_string()),
                ("Bot 3".to_string(), "User 3".to_string()),
            ],
            mining_area_name: "Area".to_string(),
            viewer_player_number: None,
            viewer_robot_id: None,
            viewer_robot_name: None,
            viewer_score: None,
            viewer_total_reward: None,
            viewer_result_claimed: false,
            viewer_source_code: None,
            viewer_program_source_id: None,
        },
        None,
    );

    assert_html_not_contains(&html, r#"id="rally-result-data""#);
    assert_html_not_contains(&html, r#"id="rallyCanvas""#);
    assert_contains_all(
        &html,
        &[
            "Replay unavailable",
            "This rally replay payload is missing, corrupt, or uses an unsupported version.",
        ],
    );
}

#[test]
fn rally_view_rendering_escapes_slots_and_javascript_ore_names() {
    let html = render_rally_view_page(
        "Player".to_string(),
        None,
        &RallyViewPageState {
            result_data: r#"{"v":1,"robots":{"robot":[]},"ground":{"sizeX":1,"sizeY":1,"positions":[]},"oreTypes":{}}"#.to_string(),
            ores: vec![robominer_db::OreRecord {
                id: 1,
                ore_name: "Ore <A> & 'B'".to_string(),
            }],
            slots: [
                ("Bot <0>".to_string(), "User <0>".to_string()),
                ("Bot 1".to_string(), "User 1".to_string()),
                ("Bot 2".to_string(), "User 2".to_string()),
                ("Bot 3".to_string(), "User 3".to_string()),
            ],
            mining_area_name: "Area & One".to_string(),
            viewer_player_number: None,
            viewer_robot_id: None,
            viewer_robot_name: None,
            viewer_score: None,
            viewer_total_reward: None,
            viewer_result_claimed: false,
            viewer_source_code: None,
            viewer_program_source_id: None,
        },
        None,
    );

    assert_html_not_contains(&html, r#"<script src="js/animation.js"></script>"#);
    assert_contains_all(
        &html,
        &[
            r#"class="rally-view-page""#,
            r#"class="rally-view-title">Rally replay</h1>"#,
            r#"class="rally-view-context-item"><dt>Area</dt><dd>Area &amp; One</dd></div>"#,
            r#"class="rally-view-player-user">User &lt;0&gt;</p>"#,
            r#"class="rally-view-player-robot">Bot &lt;0&gt;</p>"#,
            r#"id="oreCanvas0""#,
            r#"id="depotCanvas0""#,
            r#"id="depotChart0""#,
            r#"id="robotBattery0""#,
            r#"id="robotBatteryFill0""#,
            r#"id="robotTurns0""#,
            r#"id="robotAction0""#,
            r#"id="rallyPlayer0""#,
            "rally-view-player-battery-fill",
            r#"id="progressCanvas""#,
            r#"id="rallyPlayPause""#,
            r#"aria-keyshortcuts="Space""#,
            r#">Play</button>"#,
            r#"data-speed="0.1">0.1×</button>"#,
            r#"id="rallyRestart">Restart</button>"#,
            r#"id="rallyProgressTrack""#,
            r#"role="slider""#,
            r#"class="rally-view-keyboard-hint""#,
            "← → one CPU cycle (when paused)",
            "Shift+← → next area cycle",
            r#"id="rallyCycleCurrent">0</span>"#,
            r#"<script type="application/json" id="rally-result-data">"#,
            r#"<script type="application/json" id="rally-view-config">"#,
            r#""v":1"#,
            r#""viewerSlot":null"#,
            // Ore names are JSON-encoded in config (angle brackets stay literal in JSON strings).
            "Ore <A>",
        ],
    );
    for absent in [
        r#"id="robotCargo0""#,
        r#"id="robotDepot0""#,
        "function rallyPlay()",
        "function updateRobotDebugPanel(",
        "oreLegendAName').innerHTML",
    ] {
        assert_html_not_contains(&html, absent);
    }
    // Viewer modules are linked as static assets (not inlined).
    for path in [
        "js/rally_animation/payload.js",
        "js/rally_animation/draw.js",
        "js/rally_animation/debug.js",
        "js/rally_animation/timeline.js",
        "js/rally_animation/pose.js",
        "js/rally_animation/player.js",
        "js/rally_animation/bootstrap.js",
    ] {
        assert!(html.contains(path), "expected script src for {path}");
    }
    assert!(
        html.contains(r#""Ore <A> & 'B'""#) || html.contains(r#""Ore <A> & \'B\'""#),
        "expected HTML to contain an ore name encoded as `\"Ore <A> & 'B'\"` or with an escaped apostrophe\nHTML:\n{html}"
    );
}

#[test]
fn rally_view_highlights_viewer_robot_and_shows_context() {
    let html = render_rally_view_page(
        "Player".to_string(),
        None,
        &RallyViewPageState {
            result_data: r#"{"v":1,"robots":{"robot":[]},"ground":{"sizeX":1,"sizeY":1,"positions":[]},"oreTypes":{}}"#.to_string(),
            ores: Vec::new(),
            slots: [
                ("Lead Bot".to_string(), "Owner".to_string()),
                ("Other Bot".to_string(), "Other Owner".to_string()),
                ("Bot 2".to_string(), "User 2".to_string()),
                ("Bot 3".to_string(), "User 3".to_string()),
            ],
            mining_area_name: "Deep Mine".to_string(),
            viewer_player_number: Some(0),
            viewer_robot_id: Some(7),
            viewer_robot_name: Some("Lead Bot".to_string()),
            viewer_score: Some(42.5),
            viewer_total_reward: Some(17),
            viewer_result_claimed: true,
            viewer_source_code: Some("scan();\nmine();\n".to_string()),
            viewer_program_source_id: Some(11),
        },
        Some("runId=10&robotId=7").map(RallyViewBackLink::MiningResults),
    );

    assert_contains_all(
        &html,
        &[
            r#"class="rally-view-player rally-view-player-0 rally-view-player-self""#,
            r#"<span class="rally-view-player-you">You</span>"#,
            r#"<dt>Your robot</dt><dd>Lead Bot · green slot</dd>"#,
            r#"<dt>Score</dt><dd>42.5</dd>"#,
            r#"class="rally-view-context-payout">+17</dd>"#,
            r#""viewerSlot":0"#,
            r#"id="rally-view-config""#,
            r#"id="rallySourceCode""#,
            r#"class="rally-view-source-code" id="rallySourceCode""#,
            r#"id="rallySourceLine1""#,
            r#"class="rally-view-source-text">scan();</code>"#,
            r#"class="rally-view-source-line" data-line="1""#,
            r#"class="rally-view-source-line" data-line="2""#,
            "js/rally_animation/debug.js",
            "Highlighted token is the program work running this CPU cycle. Source is the private snapshot from this rally.",
            r#"id="rallyEditCodeLink" class="rally-view-source-edit-link" data-edit-href="editCode?nextProgramSourceId=11" href="editCode?nextProgramSourceId=11">Edit code at highlighted line</a>"#,
            "Opens the robot's current linked program in the editor (may differ from this snapshot).",
            r#"href="editCode?nextProgramSourceId=11">Edit code</a>"#,
            r#"href="miningQueue?robotId=7">Mining queue</a>"#,
            r#"href="robot?robotId=7">Robot workshop</a>"#,
            r#"href="miningAreaOverview">Compare areas</a>"#,
        ],
    );
    assert_html_not_contains(&html, "<pre class=\"rally-view-source-code\"");
    assert_html_not_contains(&html, "Source snapshot unavailable.");
}

#[test]
fn rally_view_shows_snapshot_unavailable_without_executed_source() {
    let html = render_rally_view_page(
        "Player".to_string(),
        None,
        &RallyViewPageState {
            result_data: r#"{"v":1,"robots":{"robot":[]},"ground":{"sizeX":1,"sizeY":1,"positions":[]},"oreTypes":{}}"#.to_string(),
            ores: Vec::new(),
            slots: [
                ("Lead Bot".to_string(), "Owner".to_string()),
                ("Other Bot".to_string(), "Other Owner".to_string()),
                ("Bot 2".to_string(), "User 2".to_string()),
                ("Bot 3".to_string(), "User 3".to_string()),
            ],
            mining_area_name: "Deep Mine".to_string(),
            viewer_player_number: Some(0),
            viewer_robot_id: Some(7),
            viewer_robot_name: Some("Lead Bot".to_string()),
            viewer_score: Some(42.5),
            viewer_total_reward: Some(17),
            viewer_result_claimed: true,
            viewer_source_code: None,
            viewer_program_source_id: Some(11),
        },
        None,
    );

    assert_contains_all(
        &html,
        &[
            r#"class="rally-view-source""#,
            "Source snapshot unavailable.",
            "This rally did not store a private program snapshot, so line highlighting is not shown.",
            r#"href="editCode?nextProgramSourceId=11">Edit linked program</a>"#,
        ],
    );
    for absent in [
        r#"id="rallyEditCodeLink""#,
        r#"id="rallySourceCode""#,
        r#"id="rallySourceLine1""#,
    ] {
        assert_html_not_contains(&html, absent);
    }
}

#[test]
fn rally_view_shows_back_link_when_return_to_is_present() {
    let html = render_rally_view_page(
        "Player".to_string(),
        None,
        &sample_rally_view_state([
            ("Bot".to_string(), "User".to_string()),
            ("Bot".to_string(), "User".to_string()),
            ("Bot".to_string(), "User".to_string()),
            ("Bot".to_string(), "User".to_string()),
        ]),
        Some("runId=10&robotId=1").map(RallyViewBackLink::MiningResults),
    );

    assert_html_contains(
        &html,
        r#"class="rally-view-back-link" href="miningResults?runId=10&amp;robotId=1">Back to results</a>"#,
    );
}

#[test]
fn rally_view_shows_back_link_to_activity() {
    let html = render_rally_view_page(
        "Player".to_string(),
        None,
        &sample_rally_view_state([
            ("Bot".to_string(), "User".to_string()),
            ("Bot".to_string(), "User".to_string()),
            ("Bot".to_string(), "User".to_string()),
            ("Bot".to_string(), "User".to_string()),
        ]),
        Some(RallyViewBackLink::Activity(default_activity_feed_query())),
    );

    assert_html_contains(
        &html,
        r#"class="rally-view-back-link" href="activity">Back to activity</a>"#,
    );
}

#[test]
fn rally_view_back_link_preserves_your_rallies_filter() {
    let html = render_rally_view_page(
        "Player".to_string(),
        None,
        &sample_rally_view_state([
            ("Bot".to_string(), "User".to_string()),
            ("Bot".to_string(), "User".to_string()),
            ("Bot".to_string(), "User".to_string()),
            ("Bot".to_string(), "User".to_string()),
        ]),
        Some(RallyViewBackLink::Activity(ActivityFeedQuery {
            filter: ActivityRallyFilter::Mine,
            area_id: None,
            limit: ACTIVITY_RALLY_PAGE_SIZE,
        })),
    );

    assert_html_contains(
        &html,
        r#"class="rally-view-back-link" href="activity?filter=mine">Back to activity</a>"#,
    );
}

#[test]
fn valid_mining_results_return_to_rejects_external_urls() {
    assert_eq!(valid_mining_results_return_to("runId=10"), Some("runId=10"));
    assert_eq!(valid_mining_results_return_to("https://evil.test"), None);
    assert_eq!(valid_mining_results_return_to("/login"), None);
}
