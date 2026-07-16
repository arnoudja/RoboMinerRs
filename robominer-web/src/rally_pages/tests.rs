use std::collections::HashMap;
use std::path::PathBuf;

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
        result_data: "var myOreTypes = {};".to_string(),
        ores: Vec::new(),
        slots,
        mining_area_name: "Area & One".to_string(),
        viewer_player_number: None,
        viewer_robot_id: None,
        viewer_robot_name: None,
        viewer_score: None,
        viewer_total_reward: None,
        viewer_result_claimed: false,
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
    assert!(body.contains("ROBOMINER_DATABASE_URL"));
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

    assert!(html.contains(r#"class="activity-page""#));
    assert!(html.contains(r#"class="activity-title">Activity</h1>"#));
    assert!(html.contains(r#"class="page-help-hint""#));
    assert!(html.contains(r#"href="helpTutorial?step=1">Tutorial</a>"#));
    assert!(html.contains("Area &amp; One"));
    assert!(html.contains("Lead &lt;Bot&gt;"));
    assert!(html.contains(r#"class="activity-rally-card activity-rally-card-replayable""#));
    assert!(html.contains(r#"href="activity">All rallies</a>"#));
    assert!(html.contains(r#"href="activity?filter=mine">Your rallies</a>"#));
    assert!(html.contains("activity-rally-filter-link-active"));
    assert!(html.contains(r#"href="activity?rallyResultId=20""#));
    assert!(html.contains(
        r#"class="activity-rally-badge activity-rally-badge-replay">Replay ready</span>"#
    ));
    assert!(
        html.contains(
            r#"class="activity-rally-badge activity-rally-badge-players">2 players</span>"#
        )
    );
    assert!(html.contains(r#"class="activity-rally-replay-cta">Watch replay"#));
    assert!(html.contains("activity-rally-participant-1"));
    assert!(html.contains("Other &amp; Bot"));
    assert!(html.contains("Other &lt;Owner&gt;"));
    assert!(html.contains("User &lt;A&gt;"));
    assert!(html.contains(r#"title="1970-01-01 00:00:00 UTC">Ended 1 hour ago</p>"#));
    assert!(html.contains(r#"title="1970-01-01 00:00:00 UTC">1 hour ago</span>"#));
    assert!(html.contains(r#"class="activity-feed-stats""#));
    assert!(html.contains(r#"<dt>Showing</dt><dd>1 rallies</dd>"#));
    assert!(html.contains(r#"<dt>Replays</dt><dd>1 ready</dd>"#));
    assert!(html.contains(r#"class="tableitem activity-area-filter-select""#));
    assert!(html.contains("Area &amp; One</option>"));
    assert!(html.contains(r#"class="activity-sidebar""#));
    assert!(html.contains("activity-sidebar-players"));
    assert!(html.contains(r#"class="activity-deck""#));
    assert!(!html.contains(r#"class="activity-deck activity-deck-full""#));
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

    assert!(!html.contains("activity-sidebar-players"));
    assert!(!html.contains("No recent players to show."));
    assert!(html.contains(r#"class="activity-sidebar""#));
    assert!(html.contains(r#"class="activity-deck""#));
    assert!(!html.contains(r#"class="activity-deck activity-deck-full""#));
    assert!(html.contains(r#"class="activity-rally-card activity-rally-card-unavailable""#));
    assert!(html.contains(r#"class="activity-rally-badge activity-rally-badge-unavailable""#));
    assert!(html.contains("No replay stored</span>"));
    assert!(!html.contains("Watch replay"));
    assert!(
        html.contains(r#"class="activity-rally-badge activity-rally-badge-players">Solo</span>"#)
    );
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

    assert!(html.contains("No finished rallies yet."));
    assert!(html.contains(r#"href="miningQueue">Add runs to your mining queue</a>"#));
    assert!(html.contains(r#"href="helpTutorial?step=1">follow the tutorial</a>"#));
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

    assert!(
        html.contains(
            r#"class="activity-rally-badge activity-rally-badge-self">You played</span>"#
        )
    );
    assert!(html.contains(r#"class="activity-rally-participant activity-rally-participant-0 activity-rally-participant-self""#));
    assert!(html.contains(r#"class="activity-rally-participant-you">You</span>"#));
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

    assert!(html.contains(r#"href="activity">All rallies</a>"#));
    assert!(html.contains(r#"href="activity?filter=mine">Your rallies</a>"#));
    assert!(html.contains("activity-rally-filter-link-active"));
    assert!(html.contains("No rallies you've joined yet."));
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

    assert!(html.contains(r#"href="activity?rallyResultId=20&amp;filter=mine""#));
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

    assert!(html.contains(r#"href="activity?areaId=5&amp;limit=20">Load more rallies</a>"#));
    assert!(html.contains(r#"value="activity?areaId=5" selected>Crystal Cave</option>"#));
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

    assert!(html.contains("No finished rallies in this area yet."));
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

    assert!(html.contains(r#"class="activity-section-title">Your mining queue</h2>"#));
    assert!(html.contains("2/3 slots in use"));
    assert!(html.contains(r#"href="miningQueue?robotId=7">Crystal &amp; Cave</a>"#));
    assert!(html.contains(r#"class="activity-queue-manage" href="miningQueue">Manage queue</a>"#));
}

#[test]
fn rally_view_rendering_escapes_slots_and_javascript_ore_names() {
    let html = render_rally_view_page(
        "Player".to_string(),
        None,
        &RallyViewPageState {
            result_data: "var myOreTypes = {};".to_string(),
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
        },
        None,
    );

    assert!(!html.contains(r#"<script src="js/animation.js"></script>"#));
    assert!(html.contains(r#"class="rally-view-page""#));
    assert!(html.contains(r#"class="rally-view-title">Rally replay</h1>"#));
    assert!(
        html.contains(
            r#"class="rally-view-context-item"><dt>Area</dt><dd>Area &amp; One</dd></div>"#
        )
    );
    assert!(html.contains(r#"class="rally-view-player-user">User &lt;0&gt;</p>"#));
    assert!(html.contains(r#"class="rally-view-player-robot">Bot &lt;0&gt;</p>"#));
    assert!(html.contains(r#"id="oreCanvas0""#));
    assert!(html.contains(r#"id="progressCanvas""#));
    assert!(html.contains(r#"id="rallyPlayPause">Play</button>"#));
    assert!(html.contains(r#"id="rallyRestart">Restart</button>"#));
    assert!(html.contains(r#"id="rallyProgressTrack""#));
    assert!(html.contains(r#"id="rallyCycleCurrent">0</span>"#));
    assert!(html.contains("function rallyPlay()"));
    assert!(html.contains("function rallySeekToRatio(ratio)"));
    assert!(html.contains("function robotDrawRadiusPixels(robot, scale)"));
    assert!(
        html.contains("myRallyContext.clearRect(minPxX, minPxY, maxPxX - minPxX, maxPxY - minPxY)")
    );
    assert!(html.contains("var myRallyViewerSlot = null;"));
    assert!(html.contains("var myRallyContext = myRallyCanvas.getContext('2d');"));
    assert!(html.contains("function runanimation()"));
    assert!(html.contains("return 'Ore \\x3cA\\x3e \\x26 \\'B\\'';"));
    assert!(html.contains("var myOreTypes = {};"));
}

#[test]
fn rally_view_highlights_viewer_robot_and_shows_context() {
    let html = render_rally_view_page(
        "Player".to_string(),
        None,
        &RallyViewPageState {
            result_data: "var myOreTypes = {};".to_string(),
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
        },
        Some("runId=10&robotId=7").map(RallyViewBackLink::MiningResults),
    );

    assert!(
        html.contains(r#"class="rally-view-player rally-view-player-0 rally-view-player-self""#)
    );
    assert!(html.contains(r#"<span class="rally-view-player-you">You</span>"#));
    assert!(html.contains(r#"<dt>Your robot</dt><dd>Lead Bot · green slot</dd>"#));
    assert!(html.contains(r#"<dt>Score</dt><dd>42.5</dd>"#));
    assert!(html.contains(r#"class="rally-view-context-payout">+17</dd>"#));
    assert!(html.contains(r#"var myRallyViewerSlot = 0;"#));
    assert!(html.contains(r#"href="miningQueue?robotId=7">Mining queue</a>"#));
    assert!(html.contains(r#"href="robot?robotId=7">Robot workshop</a>"#));
    assert!(html.contains(r#"href="miningAreaOverview">Compare areas</a>"#));
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

    assert!(html.contains(r#"class="rally-view-back-link" href="miningResults?runId=10&amp;robotId=1">Back to results</a>"#));
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

    assert!(html.contains(r#"class="rally-view-back-link" href="activity">Back to activity</a>"#));
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

    assert!(html.contains(
        r#"class="rally-view-back-link" href="activity?filter=mine">Back to activity</a>"#
    ));
}

#[test]
fn valid_mining_results_return_to_rejects_external_urls() {
    assert_eq!(valid_mining_results_return_to("runId=10"), Some("runId=10"));
    assert_eq!(valid_mining_results_return_to("https://evil.test"), None);
    assert_eq!(valid_mining_results_return_to("/login"), None);
}
