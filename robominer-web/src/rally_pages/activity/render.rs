use std::collections::HashMap;
use std::time::{SystemTime, UNIX_EPOCH};

use crate::help_pages;
use crate::html::{escape_html, format_relative_time_millis, format_utc_millis, layout};
use crate::rally_pages::{ActivityFeedQuery, ActivityPageState, ActivityRallyFilter};
use super::super::{ACTIVITY_RALLY_MAX_LIMIT, ACTIVITY_SIDEBAR_QUEUE_PREVIEW};

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

fn render_activity_rally_filter(body: &mut String, feed_query: ActivityFeedQuery) {
    body.push_str(r#"<nav class="activity-rally-filter" aria-label="Rally feed">"#);
    for filter in [ActivityRallyFilter::All, ActivityRallyFilter::Mine] {
        let class_name = if filter == feed_query.filter {
            "activity-rally-filter-link activity-rally-filter-link-active"
        } else {
            "activity-rally-filter-link"
        };
        body.push_str(&format!(
            r#"<a class="{class_name}" href="{}">{}</a>"#,
            escape_html(&feed_query.filter_href(filter)),
            escape_html(filter.label()),
        ));
    }
    body.push_str("</nav>");
}

fn render_activity_area_filter(
    body: &mut String,
    feed_query: ActivityFeedQuery,
    rally_areas: &[robominer_db::ActivityRallyAreaOption],
) {
    if rally_areas.is_empty() {
        return;
    }

    body.push_str(r#"<label class="activity-area-filter" for="activityAreaFilter">Area "#);
    body.push_str(
        r#"<select id="activityAreaFilter" class="tableitem activity-area-filter-select" aria-label="Filter rallies by area" onchange="window.location = this.value;">"#,
    );
    body.push_str(&format!(
        r#"<option value="{}"{}>All areas</option>"#,
        escape_html(&feed_query.area_href(None)),
        if feed_query.area_id.is_none() {
            " selected"
        } else {
            ""
        },
    ));
    for area in rally_areas {
        body.push_str(&format!(
            r#"<option value="{}"{}>{}</option>"#,
            escape_html(&feed_query.area_href(Some(area.mining_area_id))),
            if feed_query.area_id == Some(area.mining_area_id) {
                " selected"
            } else {
                ""
            },
            escape_html(&area.area_name),
        ));
    }
    body.push_str("</select></label>");
}

fn render_activity_feed_stats(
    body: &mut String,
    recent_rallies: &[robominer_db::ActivityRecentRallyRecord],
    now_millis: i64,
) {
    let replay_count = recent_rallies
        .iter()
        .filter(|rally| rally.rally_result_id.is_some())
        .count();
    let latest_relative = recent_rallies
        .first()
        .map(|rally| format_relative_time_millis(rally.mining_end_time_millis, now_millis))
        .unwrap_or_else(|| "none yet".to_string());
    let latest_absolute = recent_rallies
        .first()
        .map(|rally| format_utc_millis(rally.mining_end_time_millis));

    body.push_str(r#"<dl class="activity-feed-stats">"#);
    body.push_str(&format!(
        r#"<div class="activity-feed-stat"><dt>Showing</dt><dd>{} rallies</dd></div>"#,
        recent_rallies.len()
    ));
    body.push_str(&format!(
        r#"<div class="activity-feed-stat"><dt>Replays</dt><dd>{} ready</dd></div>"#,
        replay_count
    ));
    if let Some(absolute) = latest_absolute {
        body.push_str(&format!(
            r#"<div class="activity-feed-stat"><dt>Latest</dt><dd title="{}">{}</dd></div>"#,
            escape_html(&absolute),
            escape_html(&latest_relative),
        ));
    } else {
        body.push_str(&format!(
            r#"<div class="activity-feed-stat"><dt>Latest</dt><dd>{}</dd></div>"#,
            escape_html(&latest_relative),
        ));
    }
    body.push_str("</dl>");
}

fn render_activity_rallies(
    body: &mut String,
    state: &ActivityPageState,
    participant_map: &HashMap<
        i64,
        Vec<&robominer_db::ActivityRecentRallyParticipantRecord>,
    >,
    now_millis: i64,
    viewer_username: &str,
    feed_query: ActivityFeedQuery,
) {
    body.push_str(
        r#"<section class="activity-rallies" aria-labelledby="activity-rallies-title">"#,
    );
    body.push_str(r#"<div class="activity-rallies-header">"#);
    body.push_str(r#"<h2 id="activity-rallies-title" class="activity-section-title">Latest rallies</h2>"#);
    body.push_str(r#"<div class="activity-rallies-controls">"#);
    render_activity_rally_filter(body, feed_query);
    render_activity_area_filter(body, feed_query, &state.rally_areas);
    body.push_str("</div></div>");
    body.push_str(
        r#"<p class="activity-section-hint">Finished mining runs from across RoboMiner. Open a card to watch stored replays.</p>"#,
    );

    if state.recent_rallies.is_empty() {
        body.push_str(r#"<div class="activity-empty-state">"#);
        if feed_query.filter == ActivityRallyFilter::Mine {
            body.push_str(r#"<p>No rallies you've joined yet.</p>"#);
        } else if feed_query.area_id.is_some() {
            body.push_str(r#"<p>No finished rallies in this area yet.</p>"#);
        } else {
            body.push_str(r#"<p>No finished rallies yet.</p>"#);
        }
        body.push_str(
            r#"<p><a class="page-help-link" href="miningQueue">Add runs to your mining queue</a> or <a class="page-help-link" href="helpTutorial?step=1">follow the tutorial</a>.</p>"#,
        );
        body.push_str("</div>");
    } else {
        render_activity_feed_stats(body, &state.recent_rallies, now_millis);
        body.push_str(r#"<div class="activity-rally-cards">"#);
        for rally in &state.recent_rallies {
            render_activity_rally_card(
                body,
                rally,
                participant_map,
                now_millis,
                viewer_username,
                feed_query,
            );
        }
        body.push_str("</div>");
        if state.has_more_rallies && feed_query.limit < ACTIVITY_RALLY_MAX_LIMIT {
            body.push_str(&format!(
                r#"<p class="activity-load-more-wrap"><a class="activity-load-more-link" href="{}">Load more rallies</a></p>"#,
                escape_html(&feed_query.load_more_href()),
            ));
        }
    }

    body.push_str("</section>");
}

fn activity_player_count_label(participant_count: usize) -> String {
    if participant_count <= 1 {
        "Solo".to_string()
    } else {
        format!("{participant_count} players")
    }
}

fn rally_participants_for_card(
    rally: &robominer_db::ActivityRecentRallyRecord,
    participant_map: &HashMap<
        i64,
        Vec<&robominer_db::ActivityRecentRallyParticipantRecord>,
    >,
) -> Vec<(i32, String, String)> {
    let mut rally_participants = vec![(
        0_i32,
        rally.robot_name.clone(),
        rally.username.clone(),
    )];
    if let Some(other_participants) = participant_map.get(&rally.mining_queue_id) {
        for participant in other_participants {
            if participant.player_number > 0 {
                rally_participants.push((
                    participant.player_number,
                    participant.robot_name.clone(),
                    participant.username.clone(),
                ));
            }
        }
    }
    rally_participants.sort_by_key(|participant| participant.0);
    rally_participants
}

fn viewer_participated_in_rally(
    viewer_username: &str,
    rally_participants: &[(i32, String, String)],
) -> bool {
    rally_participants
        .iter()
        .any(|(_, _, username)| username == viewer_username)
}

fn render_activity_rally_card(
    body: &mut String,
    rally: &robominer_db::ActivityRecentRallyRecord,
    participant_map: &HashMap<
        i64,
        Vec<&robominer_db::ActivityRecentRallyParticipantRecord>,
    >,
    now_millis: i64,
    viewer_username: &str,
    feed_query: ActivityFeedQuery,
) {
    let rally_participants = rally_participants_for_card(rally, participant_map);
    let viewer_participated = viewer_participated_in_rally(viewer_username, &rally_participants);
    let player_count_label = activity_player_count_label(rally_participants.len());
    let ended_relative = format_relative_time_millis(rally.mining_end_time_millis, now_millis);
    let ended_absolute = format_utc_millis(rally.mining_end_time_millis);
    let replay_available = rally.rally_result_id.is_some();
    let card_tag = if replay_available {
        "a"
    } else {
        "article"
    };
    let card_class = if replay_available {
        "activity-rally-card activity-rally-card-replayable"
    } else {
        "activity-rally-card activity-rally-card-unavailable"
    };
    if replay_available {
        let replay_href = feed_query.append_to_href(&format!(
            "activity?rallyResultId={}",
            rally.rally_result_id.unwrap_or_default()
        ));
        body.push_str(&format!(
            r#"<{card_tag} class="{card_class}" href="{}">"#,
            escape_html(&replay_href),
        ));
    } else {
        body.push_str(&format!(r#"<{card_tag} class="{card_class}">"#));
    }

    body.push_str(r#"<header class="activity-rally-card-header">"#);
    body.push_str(r#"<div class="activity-rally-card-heading">"#);
    body.push_str(&format!(
        r#"<h3 class="activity-rally-area">{}</h3>"#,
        escape_html(&rally.mining_area_name),
    ));
    body.push_str(&format!(
        r#"<p class="activity-rally-ended" title="{}">Ended {}</p>"#,
        escape_html(&ended_absolute),
        escape_html(&ended_relative),
    ));
    body.push_str("</div>");
    body.push_str(r#"<div class="activity-rally-badges">"#);
    if viewer_participated {
        body.push_str(
            r#"<span class="activity-rally-badge activity-rally-badge-self">You played</span>"#,
        );
    }
    body.push_str(&format!(
        r#"<span class="activity-rally-badge activity-rally-badge-players">{}</span>"#,
        escape_html(&player_count_label),
    ));
    if replay_available {
        body.push_str(
            r#"<span class="activity-rally-badge activity-rally-badge-replay">Replay ready</span>"#,
        );
    } else {
        body.push_str(
            r#"<span class="activity-rally-badge activity-rally-badge-unavailable" title="No animation stored for this rally.">No replay stored</span>"#,
        );
    }
    body.push_str("</div></header>");
    body.push_str(r#"<ul class="activity-rally-participants">"#);

    for (player_number, robot_name, username) in rally_participants {
        let Ok(index) = usize::try_from(player_number) else {
            continue;
        };
        let is_viewer = username == viewer_username;
        let participant_class = if is_viewer {
            format!("activity-rally-participant activity-rally-participant-{index} activity-rally-participant-self")
        } else {
            format!("activity-rally-participant activity-rally-participant-{index}")
        };
        let you_badge = if is_viewer {
            r#"<span class="activity-rally-participant-you">You</span>"#
        } else {
            ""
        };
        body.push_str(&format!(
            r#"<li class="{participant_class}"><span class="activity-rally-participant-color" aria-hidden="true"></span><span class="activity-rally-participant-name">{}{}</span><span class="activity-rally-participant-robot">{}</span></li>"#,
            escape_html(&username),
            you_badge,
            escape_html(&robot_name),
        ));
    }

    body.push_str("</ul>");
    if replay_available {
        body.push_str(
            r#"<span class="activity-rally-replay-cta">Watch replay<span aria-hidden="true"> →</span></span>"#,
        );
    }
    body.push_str(&format!("</{card_tag}>"));
}

fn render_activity_sidebar(body: &mut String, state: &ActivityPageState, now_millis: i64) {
    body.push_str(r#"<aside class="activity-sidebar" aria-label="Activity sidebar">"#);
    render_activity_sidebar_queue(body, &state.queue_items, state.asset_summary.as_ref());
    render_activity_sidebar_recent_players(body, &state.recent_users, now_millis);
    body.push_str("</aside>");
}

fn render_activity_sidebar_queue(
    body: &mut String,
    queue_items: &[robominer_db::MiningQueuePageItemRecord],
    asset_summary: Option<&robominer_db::UserAssetSummaryRecord>,
) {
    if queue_items.is_empty() {
        return;
    }

    body.push_str(r#"<section class="activity-sidebar-panel">"#);
    body.push_str(
        r#"<h2 class="activity-section-title">Your mining queue</h2>"#,
    );
    if let Some(summary) = asset_summary {
        body.push_str(&format!(
            r#"<p class="activity-section-hint">{}</p>"#,
            escape_html(&activity_queue_usage_hint(queue_items.len(), summary)),
        ));
    }
    body.push_str(r#"<ul class="activity-queue-list">"#);
    for item in queue_items.iter().take(ACTIVITY_SIDEBAR_QUEUE_PREVIEW) {
        body.push_str(&format!(
            r#"<li class="activity-queue-item"><a class="activity-queue-link" href="miningQueue?robotId={}">{}</a></li>"#,
            item.robot_id,
            escape_html(&item.area_name),
        ));
    }
    if queue_items.len() > ACTIVITY_SIDEBAR_QUEUE_PREVIEW {
        body.push_str(&format!(
            r#"<li class="activity-queue-item activity-queue-item-more">+{} more</li>"#,
            queue_items.len() - ACTIVITY_SIDEBAR_QUEUE_PREVIEW
        ));
    }
    body.push_str("</ul>");
    body.push_str(
        r#"<a class="activity-queue-manage" href="miningQueue">Manage queue</a>"#,
    );
    body.push_str("</section>");
}

fn activity_queue_usage_hint(
    queue_count: usize,
    summary: &robominer_db::UserAssetSummaryRecord,
) -> String {
    let capacity = summary.robot_count * i64::from(summary.mining_queue_size);
    if capacity > 0 {
        format!("{queue_count}/{capacity} slots in use")
    } else {
        format!("{queue_count} runs queued")
    }
}

fn render_activity_sidebar_recent_players(
    body: &mut String,
    recent_users: &[robominer_db::ActivityRecentUserRecord],
    now_millis: i64,
) {
    if recent_users.is_empty() {
        return;
    }

    body.push_str(
        r#"<section class="activity-sidebar-panel activity-sidebar-players" aria-labelledby="activity-players-title">"#,
    );
    body.push_str(r#"<h2 id="activity-players-title" class="activity-section-title">Recent players</h2>"#);
    body.push_str(
        r#"<p class="activity-section-hint">Players active most recently.</p>"#,
    );
    body.push_str(r#"<ul class="activity-player-list">"#);
    for user in recent_users {
        let login_relative = format_relative_time_millis(user.last_login_time_millis, now_millis);
        let login_absolute = format_utc_millis(user.last_login_time_millis);
        body.push_str(&format!(
            r#"<li class="activity-player-item"><span class="activity-player-name">{}</span><span class="activity-player-login" title="{}">{}</span></li>"#,
            escape_html(&user.username),
            escape_html(&login_absolute),
            escape_html(&login_relative),
        ));
    }
    body.push_str("</ul></section>");
}

