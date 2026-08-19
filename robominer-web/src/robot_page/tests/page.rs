use std::path::PathBuf;

use crate::ServerConfig;
use crate::html::{assert_contains_all, assert_html_contains, assert_html_not_contains};

use super::super::render::render_robot_page;
use super::super::robot_page;
use super::fixtures::{authenticated_request, sample_robot_state};

#[tokio::test(flavor = "current_thread")]
async fn robot_requires_database_configuration() {
    let config = ServerConfig {
        static_root: PathBuf::from("robominer-web/static"),
        database_pool: None,
        allow_signup: true,
        trust_proxy: false,
    };

    let response = robot_page(&authenticated_request("/robot"), &config).await;
    let body = String::from_utf8(response.body).expect("message should be utf-8");

    assert_eq!(response.status, 503);
    assert_html_contains(&body, "ROBOMINER_DATABASE_URL");
}

#[test]
fn robot_rendering_preserves_form_contract_and_escapes_fields() {
    let html = render_robot_page(
        "Player".to_string(),
        None,
        &sample_robot_state(Some(
            "Unable to apply robot changes: Invalid <robot>".to_string(),
        )),
    );

    assert_contains_all(
        &html,
        &[
            r#"class="robot-page""#,
            r#"class="robot-summary""#,
            r#"class="robot-deck""#,
            r#"class="robot-fleet-card robot-fleet-card-active""#,
            r#"class="robot-fleet-hint""#,
            r#"class="robot-quick-link robot-quick-link-edit-program""#,
            r#"href="editCode?nextProgramSourceId=11""#,
            r#"href="helpProgramTips">Programming tips</a>"#,
            r#"href="helpMechanics">Mechanics guide</a>"#,
            r#"class="robot-status-badge robot-status-dirty" hidden>Unsaved changes</span>"#,
            r#"class="robot-btn robot-btn-secondary robot-reset-btn" hidden>Reset changes</button>"#,
            "Apply queues part and program changes for this robot.",
            r#"src="js/common/panel_state.js?v="#,
            r#"src="js/common/url_query.js?v="#,
            r#"src="js/robot/page.js?v="#,
            r#"href="miningQueue?robotId=7""#,
            r#"data-compiled-size="12""#,
            r#"data-memory-capacity="20""#,
            "Memory &amp; Spare (30 i)",
            r#"<form id="robotForm" action="robot" method="post" class="robot-config-form">"#,
            r#"<input type="hidden" name="robotId" value="7"/>"#,
            "Bot &lt;One&gt;",
            "Source &lt;One&gt;",
            "Container &amp; Current (10 Ore)",
            "Container &lt;Spare&gt; (25 Ore)",
            "Mining Unit (2 u/t)",
            "Mining &lt;Spare&gt; (4 u/t)",
            "Battery (5000 pc)",
            "Battery &lt;Spare&gt; (9000 pc)",
            "Memory &lt;Current&gt; (20 i)",
            "CPU (3 i/t)",
            "CPU &lt;Spare&gt; (11 i/t)",
            "Engine (11 fc)",
            "Engine &lt;Spare&gt; (15 fc)",
            "Ore Scanner (5 sd)",
            "Scanner &lt;Spare&gt; (8 sd)",
            r#"id="robotName7" name="robotName7""#,
            r#"name="oreContainerId7""#,
            r#"id="memoryModuleId7" name="memoryModuleId7""#,
            r#"class="robot-progress-value">12/20</span>"#,
            r#"class="robot-btn robot-btn-primary">Apply changes</button>"#,
            ">2 minutes<",
            r#"class="robot-banner robot-banner-error">Unable to apply robot changes: Invalid &lt;robot&gt;</p>"#,
        ],
    );
    for absent in [
        r#"<script src="js/robot.js"></script>"#,
        r#"id="robotId""#,
        r#"<button type="submit">Select</button>"#,
        "Container Hidden",
    ] {
        assert_html_not_contains(&html, absent);
    }
}

#[test]
fn robot_shows_success_banner_after_apply() {
    let html = render_robot_page(
        "Player".to_string(),
        None,
        &sample_robot_state(Some("Robot changes queued".to_string())),
    );

    assert_html_contains(
        &html,
        r#"class="robot-banner robot-banner-success">Robot changes queued</p>"#,
    );
}

#[test]
fn robot_shows_claim_banner_when_results_claimed() {
    let mut state = sample_robot_state(None);
    state.claimed_results = robominer_db::ClaimedUserResults {
        claimed_queues: 2,
        ore_rewards: vec![robominer_db::ClaimedOreRewardRecord {
            ore_id: 2,
            ore_name: "Iron".to_string(),
            reward: 12,
        }],
    };

    let html = render_robot_page("Player".to_string(), None, &state);

    assert_contains_all(
        &html,
        &[
            r#"class="robot-claim-banner"><span class="claim-banner-label">Added to wallet:</span>"#,
            r#"class="claim-banner-reward-amount">+12</span>"#,
        ],
    );
}
