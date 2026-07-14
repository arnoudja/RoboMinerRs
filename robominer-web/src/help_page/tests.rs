use std::collections::HashMap;
use std::path::PathBuf;

use super::{help_page, help_text_page};
use crate::help_pages;
use crate::{Request, ServerConfig};

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
    }
}

#[test]
fn help_route_renders_themed_help_center() {
    let response = help_page(&request("/help"), &config(), false);
    let body = String::from_utf8(response.body).expect("html should be utf-8");

    assert_eq!(response.status, 200);
    assert!(body.contains(r#"class="help-page""#));
    assert!(body.contains("Help center"));
    assert!(body.contains(r#"class="help-card""#));
    assert!(body.contains(r#"href="helpTutorial?step=1""#));
    assert!(body.contains(r#"href="helpProgramTips""#));
    assert!(!body.contains("target=\"tutorialWindow\""));
}

#[test]
fn help_route_shows_signup_welcome_banner() {
    let response = help_page(&request("/help?welcome=1"), &config(), true);
    let body = String::from_utf8(response.body).expect("html should be utf-8");

    assert!(body.contains(r#"class="help-welcome-banner""#));
    assert!(body.contains(r#"href="helpTutorial?step=1""#));
}

#[test]
fn help_text_routes_render_reader_shell_with_sidebar() {
    let tutorial = help_text_page(&request("/helpTutorial"), &config(), "helpTutorial", None);
    let program_tips =
        help_text_page(&request("/helpProgramTips"), &config(), "helpProgramTips", None);
    let robot_program =
        help_text_page(&request("/helpRobotProgram"), &config(), "helpRobotProgram", None);
    let mechanics = help_text_page(&request("/helpMechanics"), &config(), "helpMechanics", None);

    assert_eq!(tutorial.status, 200);
    let tutorial_body = String::from_utf8(tutorial.body).expect("html should be utf-8");
    assert!(tutorial_body.contains(r#"class="help-page""#));
    assert!(tutorial_body.contains(r#"class="help-sidebar""#));
    assert!(tutorial_body.contains("help-nav-item-active"));
    assert!(tutorial_body.contains("<h1>Tutorial</h1>"));
    assert!(tutorial_body.contains("Step 1 of 5"));
    assert!(tutorial_body.contains("Add to queue"));
    assert!(tutorial_body.contains(r#"href="miningQueue""#));
    assert!(tutorial_body.contains(r#"href="helpTutorial?step=2""#));
    assert!(!tutorial_body.contains(r#"class="help-article-toc""#));
    assert!(!tutorial_body.contains("'add' button"));

    assert_eq!(program_tips.status, 200);
    let tips_body = String::from_utf8(program_tips.body).expect("html should be utf-8");
    assert!(tips_body.contains(r#"class="help-spoiler-banner""#));
    assert!(tips_body.contains(r#"class="help-article-toc""#));
    assert!(tips_body.contains("href=\"#repeated-mining\""));
    assert!(tips_body.contains(r#"<h2 id="repeated-mining">Repeated mining</h2>"#));
    assert!(tips_body.contains(r#"<pre class="help-code-block"><code>"#));
    assert!(tips_body.contains("<h1>Programming tips</h1>"));

    assert_eq!(robot_program.status, 200);
    let robot_program_body = String::from_utf8(robot_program.body).expect("html should be utf-8");
    assert!(robot_program_body.contains(r#"class="help-article-toc""#));
    assert!(robot_program_body.contains(r#"<h2 id="statements">Statements</h2>"#));
    assert!(robot_program_body.contains("<h1>Robot programming help</h1>"));

    assert_eq!(mechanics.status, 200);
    let mechanics_body = String::from_utf8(mechanics.body).expect("html should be utf-8");
    assert!(mechanics_body.contains(r#"class="help-article-toc""#));
    assert!(mechanics_body.contains(r#"<h2 id="ore-container">Ore Container</h2>"#));
    assert!(mechanics_body.contains(r#"<div class="help-table-wrap"><table class="helptable">"#));
    assert!(mechanics_body.contains("<h1>RoboMiner Mechanics</h1>"));
}

#[test]
fn tutorial_step_navigation_links_previous_and_next() {
    let step_three = help_text_page(
        &request("/helpTutorial?step=3"),
        &config(),
        "helpTutorial",
        Some(3),
    );
    let body = String::from_utf8(step_three.body).expect("html should be utf-8");

    assert!(body.contains("Step 3 of 5"));
    assert!(body.contains("Review mining results"));
    assert!(body.contains(r#"href="helpTutorial?step=2""#));
    assert!(body.contains(r#"href="helpTutorial?step=4""#));
    assert!(body.contains(r#"href="miningResults""#));
    assert!(body.contains("Replay rally"));
}

#[test]
fn tutorial_final_step_links_to_programming_tips() {
    let step_five = help_text_page(
        &request("/helpTutorial?step=5"),
        &config(),
        "helpTutorial",
        Some(5),
    );
    let body = String::from_utf8(step_five.body).expect("html should be utf-8");

    assert!(body.contains("Step 5 of 5"));
    assert!(body.contains("Save program"));
    assert!(body.contains("Apply changes"));
    assert!(body.contains(r#"href="helpTutorial?step=4""#));
    assert!(body.contains(r#"href="helpProgramTips""#));
    assert!(body.contains(r#"href="editCode""#));
}

#[test]
fn help_text_route_returns_not_found_for_unknown_guide() {
    let response = help_text_page(&request("/helpUnknown"), &config(), "helpUnknown", None);
    assert_eq!(response.status, 404);
}

#[test]
fn help_guides_are_registered_for_all_legacy_routes() {
    assert!(help_pages::guide_by_href("helpTutorial").is_some());
    assert!(help_pages::guide_by_href("helpProgramTips").is_some());
    assert!(help_pages::guide_by_href("helpRobotProgram").is_some());
    assert!(help_pages::guide_by_href("helpMechanics").is_some());
}
