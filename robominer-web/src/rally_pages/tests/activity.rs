use std::path::PathBuf;

use crate::ServerConfig;
use crate::html::{
    assert_contains_all, assert_html_contains, assert_html_has_class, assert_html_not_contains,
};
use crate::test_support::route;

use super::super::{
    ACTIVITY_RALLY_PAGE_SIZE, ActivityFeedQuery, ActivityRallyFilter, render_activity_page_at,
};
use super::fixtures::{default_activity_feed_query, request, sample_activity_state};

#[tokio::test(flavor = "current_thread")]
async fn activity_requires_database_configuration() {
    let config = ServerConfig {
        static_root: PathBuf::from("robominer-web/static"),
        database_pool: None,
        allow_signup: true,
        trust_proxy: false,
    };

    let response = route(&request("/activity"), &config).await;
    let body = response.body_utf8();

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
                score: 42.5,
                mining_end_time_millis: 0,
            }],
            vec![
                robominer_db::ActivityRecentRallyParticipantRecord {
                    mining_queue_id: 10,
                    player_number: 0,
                    robot_name: "Lead <Bot>".to_string(),
                    username: "Owner".to_string(),
                    score: 42.5,
                },
                robominer_db::ActivityRecentRallyParticipantRecord {
                    mining_queue_id: 10,
                    player_number: 1,
                    robot_name: "Other & Bot".to_string(),
                    username: "Other <Owner>".to_string(),
                    score: 18.0,
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
            r#"class="activity-rally-participant-score" title="Rally score">42.5</span>"#,
            r#"class="activity-rally-participant-score" title="Rally score">18.0</span>"#,
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
                score: 7.5,
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
            r#"class="activity-rally-participant-score" title="Rally score">7.5</span>"#,
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
                score: 30.0,
                mining_end_time_millis: 0,
            }],
            vec![robominer_db::ActivityRecentRallyParticipantRecord {
                mining_queue_id: 10,
                player_number: 1,
                robot_name: "Other Bot".to_string(),
                username: "Other Owner".to_string(),
                score: 12.0,
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
                score: 30.0,
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
                score: 55.0,
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
