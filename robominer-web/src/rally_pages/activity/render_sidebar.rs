use crate::html::{EscapedHtml, format_relative_time_millis, format_utc_millis};
use crate::rally_pages::ActivityPageState;

pub(super) fn render_activity_sidebar(
    body: &mut String,
    state: &ActivityPageState,
    now_millis: i64,
) {
    body.push_str(r#"<aside class="activity-sidebar" aria-label="Activity sidebar">"#);
    render_activity_sidebar_recent_players(body, &state.recent_users, now_millis);
    body.push_str("</aside>");
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
