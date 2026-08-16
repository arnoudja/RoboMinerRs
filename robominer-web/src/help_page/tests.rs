use std::collections::HashMap;
use std::path::PathBuf;

use crate::help_pages;
use crate::html::{assert_contains_all, assert_html_not_contains};
use crate::{Request, ServerConfig};

use super::{help_page, help_text_page};

fn request(path: &str) -> Request {
    let (path, query) = crate::http::split_target(path);
    Request {
        method: "GET".to_string(),
        path,
        query,
        form: HashMap::new(),
        form_values: HashMap::new(),
        headers: HashMap::new(),
    }
}

fn config() -> ServerConfig {
    ServerConfig {
        static_root: PathBuf::from("static"),
        database_pool: None,
        allow_signup: true,
        trust_proxy: false,
    }
}

#[tokio::test(flavor = "current_thread")]
async fn help_route_renders_themed_help_center() {
    let response = help_page(&request("/help"), &config(), false).await;
    let body = String::from_utf8(response.body).expect("html should be utf-8");

    assert_eq!(response.status, 200);
    assert_contains_all(
        &body,
        &[
            r#"class="help-page""#,
            "Help center",
            r#"class="help-card""#,
            r#"href="helpTutorial?step=1""#,
            r#"href="helpProgramTips""#,
        ],
    );
    assert_html_not_contains(&body, "target=\"tutorialWindow\"");
}

#[tokio::test(flavor = "current_thread")]
async fn help_route_shows_signup_welcome_banner() {
    let response = help_page(&request("/help?welcome=1"), &config(), true).await;
    let body = String::from_utf8(response.body).expect("html should be utf-8");

    assert_contains_all(
        &body,
        &[
            r#"class="help-welcome-banner""#,
            r#"href="helpTutorial?step=1""#,
        ],
    );
}

#[tokio::test(flavor = "current_thread")]
async fn help_text_routes_render_reader_shell_with_sidebar() {
    let tutorial = help_text_page(&request("/helpTutorial"), &config(), "helpTutorial", None).await;
    let program_tips = help_text_page(
        &request("/helpProgramTips"),
        &config(),
        "helpProgramTips",
        None,
    )
    .await;
    let robot_program = help_text_page(
        &request("/helpRobotProgram"),
        &config(),
        "helpRobotProgram",
        None,
    )
    .await;
    let mechanics =
        help_text_page(&request("/helpMechanics"), &config(), "helpMechanics", None).await;

    assert_eq!(tutorial.status, 200);
    let tutorial_body = String::from_utf8(tutorial.body).expect("html should be utf-8");
    assert_contains_all(
        &tutorial_body,
        &[
            r#"class="help-page""#,
            r#"class="help-sidebar""#,
            "help-nav-item-active",
            "<h1>Tutorial</h1>",
            "Step 1 of 5",
            "Add to queue",
            r#"href="miningQueue""#,
            r#"href="helpTutorial?step=2""#,
        ],
    );
    assert_html_not_contains(&tutorial_body, r#"class="help-article-toc""#);
    assert_html_not_contains(&tutorial_body, "'add' button");

    assert_eq!(program_tips.status, 200);
    let tips_body = String::from_utf8(program_tips.body).expect("html should be utf-8");
    assert_contains_all(
        &tips_body,
        &[
            r#"class="help-spoiler-banner""#,
            r#"class="help-article-toc""#,
            "href=\"#repeated-mining\"",
            r#"<h2 id="repeated-mining">Repeated mining</h2>"#,
            r#"<h2 id="ore-scanner">Ore Scanner</h2>"#,
            r#"<h2 id="nested-heaps-and-mixed-cells">Nested heaps and mixed cells</h2>"#,
            "scan(90)",
            "move(oreDistance())",
            "when scan() started",
            "you'll mine the same ore again",
            "robot.depotStoredA",
            "Depot tax is half the container tax",
            r#"<pre class="help-code-block"><code>"#,
            "<h1>Programming tips</h1>",
        ],
    );

    assert_eq!(robot_program.status, 200);
    let robot_program_body = String::from_utf8(robot_program.body).expect("html should be utf-8");
    assert_contains_all(
        &robot_program_body,
        &[
            r#"class="help-article-toc""#,
            r#"<h2 id="statements">Statements</h2>"#,
            "collects every ore type present on that cell",
            "When the robot is already standing on ore, the distance is 0",
            "position and orientation at the moment scan() starts",
            "robot.depotSizeA",
            "robot.depotStoredA",
            "<h1>Robot programming help</h1>",
        ],
    );

    assert_eq!(mechanics.status, 200);
    let mechanics_body = String::from_utf8(mechanics.body).expect("html should be utf-8");
    assert_contains_all(
        &mechanics_body,
        &[
            r#"class="help-article-toc""#,
            r#"<h2 id="ore-container">Ore Container</h2>"#,
            r#"<h2 id="depot">Depot</h2>"#,
            "personal ore bank for each ore type",
            "robot.depotSizeA",
            "robot.depotStoredA",
            r#"<h2 id="tax">Tax</h2>"#,
            "href=\"#tax\"",
            "Container tax applies to ore still in the robot container",
            "Depot tax applies to ore already banked",
            r#"<h2 id="rally-score">Rally score</h2>"#,
            "Ore target",
            "worth up to 900 points",
            "smoothed score per robot per area",
            r#"<h2 id="scanning-and-ore-heaps">Scanning and ore heaps</h2>"#,
            "wait until the full scan countdown finishes",
            "when the scan started",
            r#"<div class="help-table-wrap"><table class="helptable">"#,
            "<h1>RoboMiner Mechanics</h1>",
        ],
    );
}

#[tokio::test(flavor = "current_thread")]
async fn tutorial_step_navigation_links_previous_and_next() {
    let step_three = help_text_page(
        &request("/helpTutorial?step=3"),
        &config(),
        "helpTutorial",
        Some(3),
    )
    .await;
    let body = String::from_utf8(step_three.body).expect("html should be utf-8");

    assert_contains_all(
        &body,
        &[
            "Step 3 of 5",
            "Review mining results",
            r#"href="helpTutorial?step=2""#,
            r#"href="helpTutorial?step=4""#,
            r#"href="miningResults""#,
            "Replay rally",
        ],
    );
}

#[tokio::test(flavor = "current_thread")]
async fn tutorial_final_step_links_to_programming_tips() {
    let step_five = help_text_page(
        &request("/helpTutorial?step=5"),
        &config(),
        "helpTutorial",
        Some(5),
    )
    .await;
    let body = String::from_utf8(step_five.body).expect("html should be utf-8");

    assert_contains_all(
        &body,
        &[
            "Step 5 of 5",
            "Save program",
            "Apply changes",
            r#"href="helpTutorial?step=4""#,
            r#"href="helpProgramTips""#,
            r#"href="editCode""#,
        ],
    );
}

#[tokio::test(flavor = "current_thread")]
async fn help_text_route_returns_not_found_for_unknown_guide() {
    let response = help_text_page(&request("/helpUnknown"), &config(), "helpUnknown", None).await;
    assert_eq!(response.status, 404);
}

#[test]
fn help_guides_are_registered_for_all_legacy_routes() {
    assert!(help_pages::guide_by_href("helpTutorial").is_some());
    assert!(help_pages::guide_by_href("helpProgramTips").is_some());
    assert!(help_pages::guide_by_href("helpRobotProgram").is_some());
    assert!(help_pages::guide_by_href("helpMechanics").is_some());
}
