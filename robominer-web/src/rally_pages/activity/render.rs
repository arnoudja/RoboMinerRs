use std::collections::HashMap;
use std::time::{SystemTime, UNIX_EPOCH};

use crate::help_pages;
use crate::html::layout;
use crate::rally_pages::{ActivityFeedQuery, ActivityPageState};

use super::render_feed::render_activity_rallies;
use super::render_sidebar::render_activity_sidebar;

pub fn render_activity_page(
    username: String,
    hud: Option<&str>,
    state: &ActivityPageState,
    feed_query: ActivityFeedQuery,
) -> String {
    render_activity_page_at(username, hud, state, activity_now_millis(), feed_query)
}

fn activity_now_millis() -> i64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|duration| i64::try_from(duration.as_millis()).unwrap_or(0))
        .unwrap_or(0)
}

pub fn render_activity_page_at(
    username: String,
    hud: Option<&str>,
    state: &ActivityPageState,
    now_millis: i64,
    feed_query: ActivityFeedQuery,
) -> String {
    let mut participant_map: HashMap<
        i64,
        Vec<&robominer_db::ActivityRecentRallyParticipantRecord>,
    > = HashMap::new();
    for participant in &state.participants {
        participant_map
            .entry(participant.mining_queue_id)
            .or_default()
            .push(participant);
    }

    let mut body = String::from(r#"<div class="activity-page">"#);
    render_activity_header(&mut body);
    body.push_str(r#"<div class="activity-deck">"#);
    render_activity_rallies(
        &mut body,
        state,
        &participant_map,
        now_millis,
        &username,
        feed_query,
    );
    render_activity_sidebar(&mut body, state, now_millis);
    body.push_str("</div></div>");

    layout("RoboMiner - Activity", "activity", &username, hud, &body)
}

fn render_activity_header(body: &mut String) {
    body.push_str(r#"<header class="activity-header">"#);
    body.push_str(r#"<div class="activity-heading">"#);
    body.push_str(r#"<h1 class="activity-title">Activity</h1>"#);
    body.push_str(
        r#"<p class="activity-subtitle">Watch recent multiplayer mining runs and replay robot behavior.</p>"#,
    );
    body.push_str(&help_pages::render_page_help_hint_line(&[
        ("helpTutorial?step=1", "Tutorial"),
        ("helpProgramTips", "Programming tips"),
    ]));
    body.push_str("</div></header>");
}
