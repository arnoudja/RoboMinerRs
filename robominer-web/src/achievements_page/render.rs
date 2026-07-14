use std::collections::HashMap;

use crate::html::{escape_html, layout};
use crate::achievements_page::AchievementsPageState;

pub(super) fn render_achievements_page(
    username: String,
    hud: Option<&str>,
    state: &AchievementsPageState,
) -> String {
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

    let mut achievements = state.achievements.clone();
    achievements.sort_by(|left, right| {
        right
            .claimable
            .cmp(&left.claimable)
            .then_with(|| {
                if left.claimable {
                    left.title.cmp(&right.title)
                } else {
                    right.achievement_id.cmp(&left.achievement_id)
                }
            })
    });

    let claimable_count = achievements.iter().filter(|achievement| achievement.claimable).count();

    let mut body = String::from(r#"<div class="achievements-page">"#);
    render_achievements_summary(
        &mut body,
        state.points_summary.points_earned,
        state.points_summary.points_achievable,
        claimable_count,
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
            );
        }
    }
    body.push_str("</div></div>");

    layout("RoboMiner - Achievements", "achievements", &username, hud, &body)
}

fn render_achievements_summary(
    body: &mut String,
    points_earned: i64,
    points_available: i64,
    claimable_count: usize,
    achievement_count: usize,
) {
    body.push_str(r#"<section class="achievements-summary" aria-label="Achievement progress">"#);
    body.push_str(r#"<div class="achievements-summary-heading">"#);
    body.push_str(r#"<h1 class="achievements-page-title">Achievements</h1>"#);
    body.push_str("</div>");
    body.push_str(r#"<ul class="achievements-summary-list">"#);
    body.push_str(&format!(
        r#"<li class="achievements-summary-item"><span class="achievements-summary-label">Points earned</span><span class="achievements-summary-value">{}/{}</span></li>"#,
        points_earned, points_available
    ));
    body.push_str(&format!(
        r#"<li class="achievements-summary-item"><span class="achievements-summary-label">Ready to claim</span><span class="achievements-summary-value">{}</span></li>"#,
        claimable_count
    ));
    body.push_str(&format!(
        r#"<li class="achievements-summary-item"><span class="achievements-summary-label">Tracks</span><span class="achievements-summary-value">{}</span></li>"#,
        achievement_count
    ));
    body.push_str("</ul></section>");
}

fn render_achievements_message(body: &mut String, state: &AchievementsPageState) {
    let Some(message) = &state.claim_message else {
        return;
    };
    let banner_class = if message.starts_with("Unable") {
        "achievements-banner achievements-banner-error"
    } else {
        "achievements-banner achievements-banner-success"
    };
    body.push_str(&format!(
        r#"<p class="{banner_class}">{}</p>"#,
        escape_html(message)
    ));
}

fn render_achievement_card(
    body: &mut String,
    achievement: &robominer_db::AchievementPageStateRecord,
    robot_count: i64,
    total_requirements: &[&robominer_db::AchievementPageTotalRequirementRecord],
    score_requirements: &[&robominer_db::AchievementPageScoreRequirementRecord],
) {
    let completed = achievement_completed(achievement);
    let card_class = if achievement.claimable {
        " achievement-card-claimable"
    } else if completed {
        " achievement-card-complete"
    } else {
        ""
    };
    let steps_percent = achievement_progress_percent(
        i64::from(achievement.steps_claimed),
        achievement.number_of_steps,
    );
    let points_percent = achievement_progress_percent(
        achievement.achievement_points_earned,
        achievement.total_achievement_points,
    );

    body.push_str(&format!(
        r#"<article class="achievement-card{card_class}" id="achievement{}">"#,
        achievement.achievement_id
    ));
    body.push_str(r#"<header class="achievement-card-header">"#);
    body.push_str(&format!(
        r#"<div><h2 class="achievement-card-title">{}</h2><p class="achievement-card-description">{}</p></div>"#,
        escape_html(&achievement.title),
        escape_html(&achievement.description)
    ));
    if achievement.claimable {
        body.push_str(&render_achievement_claim_badge(achievement.achievement_id));
    } else if completed {
        body.push_str(r#"<span class="achievement-status-badge achievement-status-complete">Completed</span>"#);
    } else {
        body.push_str(r#"<span class="achievement-status-badge achievement-status-progress">In progress</span>"#);
    }
    body.push_str("</header>");

    render_achievement_progress(
        body,
        "Steps completed",
        i64::from(achievement.steps_claimed),
        achievement.number_of_steps,
        steps_percent,
    );
    render_achievement_progress(
        body,
        "Achievement points",
        achievement.achievement_points_earned,
        achievement.total_achievement_points,
        points_percent,
    );

    body.push_str(r#"<section class="achievement-rewards"><h3 class="achievement-section-title">Next reward</h3><ul class="achievement-reward-list">"#);
    body.push_str(&format!(
        r#"<li><span class="achievement-reward-label">Points</span><span class="achievement-reward-value">{}</span></li>"#,
        achievement.next_achievement_points
    ));
    if achievement.mining_queue_reward > 0 {
        body.push_str(&format!(
            r#"<li><span class="achievement-reward-label">Queue increase</span><span class="achievement-reward-value">+{}</span></li>"#,
            achievement.mining_queue_reward
        ));
    }
    if let Some(ore_name) = &achievement.ore_name {
        let new_ore_maximum = achievement
            .current_ore_maximum
            .max(achievement.max_ore_reward);
        body.push_str(&format!(
            r#"<li><span class="achievement-reward-label">{} ore maximum</span><span class="achievement-reward-value">{} → {}</span></li>"#,
            escape_html(ore_name),
            achievement.current_ore_maximum,
            new_ore_maximum
        ));
    }
    if i64::from(achievement.robot_reward) > robot_count {
        body.push_str(r#"<li><span class="achievement-reward-label">Robot</span><span class="achievement-reward-value">New robot</span></li>"#);
    }
    if let Some(mining_area_name) = &achievement.mining_area_name {
        body.push_str(&format!(
            r#"<li><span class="achievement-reward-label">Mining area</span><span class="achievement-reward-value">{}</span></li>"#,
            escape_html(mining_area_name)
        ));
    }
    body.push_str("</ul></section>");

    if !total_requirements.is_empty() || !score_requirements.is_empty() {
        body.push_str(r#"<section class="achievement-requirements"><h3 class="achievement-section-title">Requirements</h3><ul class="achievement-requirement-list">"#);
        for requirement in total_requirements {
            body.push_str(&format!(
                r#"<li><span>{} mined</span><span class="achievement-requirement-target">{}</span><span class="{}">({})</span></li>"#,
                escape_html(&requirement.ore_name),
                requirement.amount,
                if requirement.current_amount >= requirement.amount {
                    "sufficientbalance"
                } else {
                    "insufficientbalance"
                },
                requirement.current_amount
            ));
        }
        for requirement in score_requirements {
            body.push_str(&format!(
                r#"<li><span>Average {} score</span><span class="achievement-requirement-target">{:.1}</span><span class="{}">({:.1})</span></li>"#,
                escape_html(&requirement.area_name),
                requirement.minimum_score,
                if requirement.current_score >= requirement.minimum_score {
                    "sufficientbalance"
                } else {
                    "insufficientbalance"
                },
                requirement.current_score
            ));
        }
        body.push_str("</ul></section>");
    }

    body.push_str("</article>");
}

fn render_achievement_progress(
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
    body.push_str(r#"<div class="achievement-progress-track" aria-hidden="true">"#);
    body.push_str(&format!(
        r#"<div class="achievement-progress-bar" style="width: {percent:.1}%"></div>"#
    ));
    body.push_str("</div></div>");
}

fn render_achievement_claim_badge(achievement_id: i64) -> String {
    format!(
        r#"<form action="achievements" method="post" class="achievement-claim-badge-form"><input type="hidden" name="achievementId" value="{achievement_id}"/><button type="submit" class="achievement-status-badge achievement-status-claimable achievement-claim-badge">Claim</button></form>"#
    )
}

fn achievement_completed(achievement: &robominer_db::AchievementPageStateRecord) -> bool {
    i64::from(achievement.steps_claimed) >= achievement.number_of_steps
}

fn achievement_progress_percent(current: i64, total: i64) -> f64 {
    if total <= 0 {
        return 100.0;
    }
    ((current as f64 / total as f64) * 100.0).clamp(0.0, 100.0)
}

