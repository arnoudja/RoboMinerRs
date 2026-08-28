use super::AchievementsPageState;
use super::render::{
    achievement_progress_percent, render_achievement_progress, render_achievements_summary,
};
use crate::html::EscapedHtml;

pub(super) fn render_achievements_overview(
    viewed_username: &str,
    state: &AchievementsPageState,
) -> String {
    let mut tracks = state.overview_tracks.clone();
    tracks.sort_by_key(|track| std::cmp::Reverse(track.achievement_id));

    let mut body = String::from(r#"<div class="achievements-page achievements-page-overview">"#);
    render_achievements_summary(
        &mut body,
        &format!("{}'s achievements", viewed_username),
        state.points_summary.points_earned,
        state.points_summary.points_achievable,
        None,
        tracks.len(),
    );
    body.push_str(
        r#"<p class="achievements-overview-back"><a class="achievements-overview-back-link" href="leaderboard">Back to Top players</a></p>"#,
    );

    body.push_str(r#"<div class="achievements-list">"#);
    if tracks.is_empty() {
        body.push_str(
            r#"<p class="achievements-empty">This player has not unlocked any achievements yet.</p>"#,
        );
    } else {
        for track in &tracks {
            render_overview_track_card(&mut body, track);
        }
    }
    body.push_str("</div></div>");
    body
}

pub(super) fn render_achievements_not_found(viewed_username: &str) -> String {
    let mut body = String::from(r#"<div class="achievements-page achievements-page-overview">"#);
    body.push_str(r#"<section class="achievements-summary" aria-label="Achievement progress">"#);
    body.push_str(r#"<div class="achievements-summary-heading">"#);
    body.push_str(&format!(
        r#"<h1 class="achievements-page-title">{}'s achievements</h1>"#,
        EscapedHtml::from(viewed_username)
    ));
    body.push_str("</div></section>");
    body.push_str(r#"<p class="achievements-empty">Player not found.</p>"#);
    body.push_str(
        r#"<p class="achievements-overview-back"><a class="achievements-overview-back-link" href="leaderboard">Back to Top players</a></p>"#,
    );
    body.push_str("</div>");
    body
}

fn render_overview_track_card(
    body: &mut String,
    track: &robominer_db::AchievementOverviewTrackRecord,
) {
    let completed = overview_track_completed(track);
    let card_class = if completed {
        " achievement-card-complete"
    } else {
        ""
    };
    let steps_percent =
        achievement_progress_percent(i64::from(track.steps_claimed), track.number_of_steps);
    let points_percent = achievement_progress_percent(track.points_earned, track.total_points);

    body.push_str(&format!(
        r#"<article class="achievement-card{card_class}" id="achievement{}">"#,
        track.achievement_id
    ));
    body.push_str(r#"<header class="achievement-card-header">"#);
    body.push_str(&format!(
        r#"<div><h2 class="achievement-card-title">{}</h2><p class="achievement-card-description">{}</p></div>"#,
        EscapedHtml::from(track.title.as_str()),
        EscapedHtml::from(track.description.as_str())
    ));
    if completed {
        body.push_str(r#"<span class="achievement-status-badge achievement-status-complete">Completed</span>"#);
    } else {
        body.push_str(r#"<span class="achievement-status-badge achievement-status-progress">In progress</span>"#);
    }
    body.push_str("</header>");

    render_achievement_progress(
        body,
        "Steps completed",
        i64::from(track.steps_claimed),
        track.number_of_steps,
        steps_percent,
    );
    render_achievement_progress(
        body,
        "Achievement points",
        track.points_earned,
        track.total_points,
        points_percent,
    );

    body.push_str("</article>");
}

fn overview_track_completed(track: &robominer_db::AchievementOverviewTrackRecord) -> bool {
    i64::from(track.steps_claimed) >= track.number_of_steps
}
