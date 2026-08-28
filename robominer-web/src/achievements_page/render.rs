use std::collections::HashMap;

use super::AchievementsPageState;
use super::card::render_achievement_card;
use super::overview::{render_achievements_not_found, render_achievements_overview};
use crate::html::layout;
use crate::static_assets::PageStylesheet;

pub(super) fn render_achievements_page(
    username: String,
    hud: Option<&str>,
    state: &AchievementsPageState,
) -> String {
    let body = if let Some(viewed_username) = &state.viewed_username {
        if state.player_not_found {
            render_achievements_not_found(viewed_username)
        } else {
            render_achievements_overview(viewed_username, state)
        }
    } else {
        render_own_achievements(state)
    };

    layout(
        "RoboMiner - Achievements",
        "achievements",
        &username,
        hud,
        &body,
        &[PageStylesheet::Achievements],
    )
}

fn render_own_achievements(state: &AchievementsPageState) -> String {
    let mut total_requirement_map: HashMap<
        i64,
        Vec<&robominer_db::AchievementPageTotalRequirementRecord>,
    > = HashMap::new();
    for requirement in &state.total_requirements {
        total_requirement_map
            .entry(requirement.achievement_id)
            .or_default()
            .push(requirement);
    }

    let mut score_requirement_map: HashMap<
        i64,
        Vec<&robominer_db::AchievementPageScoreRequirementRecord>,
    > = HashMap::new();
    for requirement in &state.score_requirements {
        score_requirement_map
            .entry(requirement.achievement_id)
            .or_default()
            .push(requirement);
    }

    let mut depot_total_requirement_map: HashMap<
        i64,
        Vec<&robominer_db::AchievementPageDepotTotalRequirementRecord>,
    > = HashMap::new();
    for requirement in &state.depot_total_requirements {
        depot_total_requirement_map
            .entry(requirement.achievement_id)
            .or_default()
            .push(requirement);
    }

    let mut achievements = state.achievements.clone();
    achievements.sort_by(|left, right| {
        right.claimable.cmp(&left.claimable).then_with(|| {
            if left.claimable {
                left.title.cmp(&right.title)
            } else {
                right.achievement_id.cmp(&left.achievement_id)
            }
        })
    });

    let claimable_count = achievements
        .iter()
        .filter(|achievement| achievement.claimable)
        .count();

    let mut body = String::from(r#"<div class="achievements-page">"#);
    render_achievements_summary(
        &mut body,
        "Achievements",
        state.points_summary.points_earned,
        state.points_summary.points_achievable,
        Some(claimable_count),
        achievements.len(),
    );
    render_achievements_message(&mut body, state);

    body.push_str(r#"<div class="achievements-list">"#);
    if achievements.is_empty() {
        body.push_str(r#"<p class="achievements-empty">No achievements are available yet.</p>"#);
    } else {
        for achievement in &achievements {
            render_achievement_card(
                &mut body,
                achievement,
                state.robot_count,
                total_requirement_map
                    .get(&achievement.achievement_id)
                    .map(Vec::as_slice)
                    .unwrap_or(&[]),
                score_requirement_map
                    .get(&achievement.achievement_id)
                    .map(Vec::as_slice)
                    .unwrap_or(&[]),
                depot_total_requirement_map
                    .get(&achievement.achievement_id)
                    .map(Vec::as_slice)
                    .unwrap_or(&[]),
            );
        }
    }
    body.push_str("</div></div>");
    body
}

pub(super) fn render_achievements_summary(
    body: &mut String,
    title: &str,
    points_earned: i64,
    points_available: i64,
    claimable_count: Option<usize>,
    achievement_count: usize,
) {
    body.push_str(r#"<section class="achievements-summary" aria-label="Achievement progress">"#);
    body.push_str(r#"<div class="achievements-summary-heading">"#);
    body.push_str(&format!(
        r#"<h1 class="achievements-page-title">{}</h1>"#,
        crate::html::escape_html(title)
    ));
    body.push_str("</div>");
    body.push_str(r#"<ul class="achievements-summary-list">"#);
    body.push_str(&format!(
        r#"<li class="achievements-summary-item"><span class="achievements-summary-label">Points earned</span><span class="achievements-summary-value">{}/{}</span></li>"#,
        points_earned, points_available
    ));
    if let Some(claimable_count) = claimable_count {
        body.push_str(&format!(
            r#"<li class="achievements-summary-item"><span class="achievements-summary-label">Ready to claim</span><span class="achievements-summary-value">{}</span></li>"#,
            claimable_count
        ));
    }
    body.push_str(&format!(
        r#"<li class="achievements-summary-item"><span class="achievements-summary-label">Tracks</span><span class="achievements-summary-value">{}</span></li>"#,
        achievement_count
    ));
    body.push_str("</ul></section>");
}

fn render_achievements_message(body: &mut String, state: &AchievementsPageState) {
    crate::html::render_status_banner(body, "achievements", state.claim_message.as_deref());
}

pub(super) fn render_achievement_progress(
    body: &mut String,
    label: &str,
    current: i64,
    total: i64,
    percent: f64,
) {
    body.push_str(r#"<div class="achievement-progress">"#);
    body.push_str(&format!(
        r#"<div class="achievement-progress-heading"><span>{}</span><span class="achievement-progress-value">{}/{}</span></div>"#,
        label, current, total
    ));
    body.push_str(&format!(
        r#"<progress class="achievement-progress-meter" value="{percent:.1}" max="100" aria-hidden="true"></progress>"#,
    ));
    body.push_str("</div>");
}

pub(super) fn achievement_progress_percent(current: i64, total: i64) -> f64 {
    if total <= 0 {
        return 100.0;
    }
    ((current as f64 / total as f64) * 100.0).clamp(0.0, 100.0)
}
