use crate::html::{assert_contains_all, assert_html_contains, assert_html_not_contains};

use super::super::{
    ACTIVITY_RALLY_PAGE_SIZE, ActivityFeedQuery, ActivityRallyFilter, RallyViewBackLink,
    RallyViewPageState, render_rally_view_page, valid_mining_results_return_to,
};
use super::fixtures::{default_activity_feed_query, sample_rally_view_state};

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
    assert_html_not_contains(&html, r#"class="rally-view-panel-order-button""#);
    assert_html_not_contains(&html, r#"id="rallyViewProgramPanel""#);
    assert_html_contains(&html, r#"id="rallyViewPlayersPanel""#);
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
            "Shift+← → next turn",
            r#"id="rallyTurnCurrent">0</span>"#,
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
        "js/rally_animation/draw_ground.js",
        "js/rally_animation/draw_robots.js",
        "js/rally_animation/debug_status.js",
        "js/rally_animation/debug_source.js",
        "js/rally_animation/timeline.js",
        "js/rally_animation/pose.js",
        "js/rally_animation/transport.js",
        "js/rally_animation/controls.js",
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
        Some(RallyViewBackLink::MiningResults("runId=10&robotId=7")),
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
            "js/rally_animation/debug_status.js",
            "js/rally_animation/debug_source.js",
            "js/rally_animation/side_panels.js",
            "Highlighted token is the program work running this CPU cycle. Source is the private snapshot from this rally.",
            r#"id="rallySourceStepResult""#,
            r#"class="rally-view-source-return""#,
            r#"class="rally-view-source-return-label" for="rallySourceStepResult">Return value</label>"#,
            r#"class="rally-view-source-result" id="rallySourceStepResult""#,
            r#"id="rallySourceVariables""#,
            r#"class="rally-view-source-variables""#,
            r#"class="rally-view-source-variables-table""#,
            r#"class="rally-view-source-variables-label" id="rallySourceVariablesLabel">Variables</div>"#,
            r#"id="rallyEditCodeLink" class="rally-view-source-edit-link" data-edit-href="editCode?nextProgramSourceId=11" href="editCode?nextProgramSourceId=11">Edit code at highlighted line</a>"#,
            "Opens the robot's current linked program in the editor (may differ from this snapshot).",
            r#"href="editCode?nextProgramSourceId=11">Edit code</a>"#,
            r#"href="miningQueue?robotId=7">Mining queue</a>"#,
            r#"href="robot?robotId=7">Robot workshop</a>"#,
            r#"href="miningAreaOverview">Compare areas</a>"#,
            r#"id="rallyViewProgramPanel""#,
            r#"id="rallyViewPlayersPanel""#,
            r#"data-rally-panel="program""#,
            r#"data-rally-panel="players""#,
            r#"class="rally-view-panel-order-button""#,
            r#"class="rally-view-panel-header""#,
        ],
    );
    assert_html_not_contains(&html, "<pre class=\"rally-view-source-code\"");
    assert_html_not_contains(&html, "Source snapshot unavailable.");
    let source_pos = html
        .find(r#"class="rally-view-source""#)
        .expect("source panel missing");
    let players_pos = html
        .find(r#"class="rally-view-players""#)
        .expect("players panel missing");
    assert!(
        players_pos < source_pos,
        "player cards should render above the debug source panel by default"
    );
    assert!(
        html.contains(r#"class="rally-view-side-column""#),
        "source and players should sit in a side column of separate boxes"
    );
    let sidebar_pos = html
        .find(r#"class="rally-view-sidebar""#)
        .expect("players sidebar missing");
    assert!(
        sidebar_pos < source_pos,
        "players sidebar box should be outside and above the debug source box"
    );
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
        Some(RallyViewBackLink::MiningResults("runId=10&robotId=1")),
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
