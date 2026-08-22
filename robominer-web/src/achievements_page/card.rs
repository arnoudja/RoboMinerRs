use super::render::{achievement_progress_percent, render_achievement_progress};
use crate::html::escape_html;

pub(super) fn render_achievement_card(
    body: &mut String,
    achievement: &robominer_db::AchievementPageStateRecord,
    robot_count: i64,
    total_requirements: &[&robominer_db::AchievementPageTotalRequirementRecord],
    score_requirements: &[&robominer_db::AchievementPageScoreRequirementRecord],
    depot_total_requirements: &[&robominer_db::AchievementPageDepotTotalRequirementRecord],
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
        if new_ore_maximum > achievement.current_ore_maximum {
            body.push_str(&format!(
                r#"<li><span class="achievement-reward-label">{} ore maximum</span><span class="achievement-reward-value">{} → {}</span></li>"#,
                escape_html(ore_name),
                achievement.current_ore_maximum,
                new_ore_maximum
            ));
        }
        let new_depot_maximum = achievement
            .current_depot_maximum
            .max(achievement.max_depot_reward);
        if new_depot_maximum > achievement.current_depot_maximum {
            body.push_str(&format!(
                r#"<li><span class="achievement-reward-label">{} depot maximum</span><span class="achievement-reward-value">{} → {}</span></li>"#,
                escape_html(ore_name),
                achievement.current_depot_maximum,
                new_depot_maximum
            ));
        }
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

    if !total_requirements.is_empty()
        || !score_requirements.is_empty()
        || !depot_total_requirements.is_empty()
    {
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
        for requirement in depot_total_requirements {
            body.push_str(&format!(
                r#"<li><span>{} dumped in depot</span><span class="achievement-requirement-target">{}</span><span class="{}">({})</span></li>"#,
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
                r#"<li><span>Average {} score</span><span class="achievement-requirement-target">{:.1}</span><span class="{}">{}</span></li>"#,
                escape_html(&requirement.area_name),
                requirement.minimum_score,
                if robominer_db::achievement_score_meets_requirement(
                    requirement.current_score,
                    requirement.minimum_score,
                ) {
                    "sufficientbalance"
                } else {
                    "insufficientbalance"
                },
                current_score_display(requirement, robot_count)
            ));
        }
        body.push_str("</ul></section>");
    }

    body.push_str("</article>");
}

fn render_achievement_claim_badge(achievement_id: i64) -> String {
    format!(
        r#"<form action="achievements" method="post" class="achievement-claim-badge-form"><input type="hidden" name="achievementId" value="{achievement_id}"/><button type="submit" class="achievement-status-badge achievement-status-claimable achievement-claim-badge">Claim</button></form>"#
    )
}

fn current_score_display(
    requirement: &robominer_db::AchievementPageScoreRequirementRecord,
    robot_count: i64,
) -> String {
    let score = format!("{:.1}", requirement.current_score);
    match requirement
        .current_score_robot_name
        .as_deref()
        .map(str::trim)
        .filter(|name| robot_count >= 2 && !name.is_empty())
    {
        Some(name) => format!("({}: {score})", escape_html(name)),
        None => format!("({score})"),
    }
}

fn achievement_completed(achievement: &robominer_db::AchievementPageStateRecord) -> bool {
    i64::from(achievement.steps_claimed) >= achievement.number_of_steps
}
