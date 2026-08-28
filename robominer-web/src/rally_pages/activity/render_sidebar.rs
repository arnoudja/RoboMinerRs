use super::super::ACTIVITY_SIDEBAR_QUEUE_PREVIEW;
use crate::html::{EscapedHtml, format_relative_time_millis, format_utc_millis};
use crate::rally_pages::ActivityPageState;

pub(super) fn render_activity_sidebar(
    body: &mut String,
    state: &ActivityPageState,
    now_millis: i64,
) {
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
    body.push_str(r#"<h2 class="activity-section-title">Your mining queue</h2>"#);
    if let Some(summary) = asset_summary {
        body.push_str(&format!(
            r#"<p class="activity-section-hint">{}</p>"#,
            EscapedHtml::from(activity_queue_usage_hint(queue_items.len(), summary).as_str()),
        ));
    }
    body.push_str(r#"<ul class="activity-queue-list">"#);
    for item in queue_items.iter().take(ACTIVITY_SIDEBAR_QUEUE_PREVIEW) {
        body.push_str(&format!(
            r#"<li class="activity-queue-item"><a class="activity-queue-link" href="miningQueue?robotId={}">{}</a></li>"#,
            item.robot_id,
            EscapedHtml::from(item.area_name.as_str()),
        ));
    }
    if queue_items.len() > ACTIVITY_SIDEBAR_QUEUE_PREVIEW {
        body.push_str(&format!(
            r#"<li class="activity-queue-item activity-queue-item-more">+{} more</li>"#,
            queue_items.len() - ACTIVITY_SIDEBAR_QUEUE_PREVIEW
        ));
    }
    body.push_str("</ul>");
    body.push_str(r#"<a class="activity-queue-manage" href="miningQueue">Manage queue</a>"#);
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
    body.push_str(
        r#"<h2 id="activity-players-title" class="activity-section-title">Recent players</h2>"#,
    );
    body.push_str(r#"<p class="activity-section-hint">Players active most recently.</p>"#);
    body.push_str(r#"<ul class="activity-player-list">"#);
    for user in recent_users {
        let login_relative = format_relative_time_millis(user.last_login_time_millis, now_millis);
        let login_absolute = format_utc_millis(user.last_login_time_millis);
        body.push_str(&format!(
            r#"<li class="activity-player-item"><span class="activity-player-name">{}</span><span class="activity-player-login" title="{}">{}</span></li>"#,
            EscapedHtml::from(user.username.as_str()),
            EscapedHtml::from(login_absolute.as_str()),
            EscapedHtml::from(login_relative.as_str()),
        ));
    }
    body.push_str("</ul></section>");
}
