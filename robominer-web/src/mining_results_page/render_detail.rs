use std::collections::HashMap;

use robominer_domain::{SCORE_TIER_COUNT, ScoreTierBreakdown, ore_amounts, score_breakdown};

use crate::html::{escape_html, format_utc_millis};
use crate::mining_results_page::MiningResultsPageState;

#[derive(Clone, Copy)]
struct ScoringSlot<'a> {
    name: Option<&'a str>,
    amount: i32,
}

pub(super) fn render_mining_results_detail_section(
    body: &mut String,
    state: &MiningResultsPageState,
    robot_names: &HashMap<i64, &str>,
    ore_result_map: &HashMap<i64, Vec<&robominer_db::MiningResultOreStateRecord>>,
    action_result_map: &HashMap<i64, Vec<&robominer_db::MiningResultActionStateRecord>>,
    area_ore_map: &HashMap<i64, Vec<&robominer_db::MiningResultAreaOreRecord>>,
) {
    body.push_str(
        r#"<div class="mining-results-detail-area"><div class="mining-results-detail-panels">"#,
    );
    for result in &state.results {
        let ore_results = ore_result_map
            .get(&result.mining_queue_id)
            .map(Vec::as_slice)
            .unwrap_or(&[]);
        let action_results = action_result_map
            .get(&result.mining_queue_id)
            .map(Vec::as_slice)
            .unwrap_or(&[]);
        let area_ores = area_ore_map
            .get(&result.mining_area_id)
            .map(Vec::as_slice)
            .unwrap_or(&[]);
        let robot_name = robot_names
            .get(&result.robot_id)
            .copied()
            .unwrap_or("Robot");
        render_mining_result_detail_panel(
            body,
            result,
            robot_name,
            ore_results,
            action_results,
            area_ores,
            Some(result.mining_queue_id) == state.selected_mining_queue_id,
        );
    }
    body.push_str("</div></div>");
}

fn render_mining_result_detail_panel(
    body: &mut String,
    result: &robominer_db::MiningResultStateRecord,
    robot_name: &str,
    ore_results: &[&robominer_db::MiningResultOreStateRecord],
    action_results: &[&robominer_db::MiningResultActionStateRecord],
    area_ores: &[&robominer_db::MiningResultAreaOreRecord],
    active: bool,
) {
    let active_class = if active {
        " mining-results-detail-panel-active"
    } else {
        ""
    };
    let hidden_attr = if active { "" } else { " hidden" };

    body.push_str(&format!(
        r#"<div class="mining-results-detail-panel{active_class}" id="miningResultDetails{}" data-run-id="{}" data-robot-id="{}" data-area-name="{}" data-sort-end="{}" data-sort-reward="{}" data-sort-score="{}"{hidden_attr}>"#,
        result.mining_queue_id,
        result.mining_queue_id,
        result.robot_id,
        escape_html(&result.mining_area_name),
        result.mining_end_time_millis,
        result.total_reward,
        result.score
    ));
    body.push_str(r#"<header class="mining-results-detail-header">"#);
    body.push_str(&format!(
        r#"<div><h2 class="mining-results-detail-title">{}</h2><p class="mining-results-detail-subtitle">{} · Ended {} · Score {:.1}</p></div>"#,
        escape_html(&result.mining_area_name),
        escape_html(robot_name),
        escape_html(&format_utc_millis(result.mining_end_time_millis)),
        result.score
    ));
    body.push_str(&render_mining_result_replay_action(result));
    body.push_str("</header>");
    render_mining_result_breakdown(body, result, ore_results, action_results, area_ores);
    body.push_str("</div>");
}

fn render_mining_result_replay_action(result: &robominer_db::MiningResultStateRecord) -> String {
    if let Some(rally_result_id) = result.rally_result_id {
        return format!(
            r#"<a class="mining-results-replay-link mining-results-replay-link-primary" href="miningResults?rallyResultId={rally_result_id}" data-rally-result-id="{rally_result_id}">Replay rally</a>"#
        );
    }
    r#"<span class="mining-results-replay-disabled" title="No animation stored for this run.">Replay unavailable</span>"#
        .to_string()
}

fn render_mining_result_breakdown(
    body: &mut String,
    result: &robominer_db::MiningResultStateRecord,
    ore_results: &[&robominer_db::MiningResultOreStateRecord],
    action_results: &[&robominer_db::MiningResultActionStateRecord],
    area_ores: &[&robominer_db::MiningResultAreaOreRecord],
) {
    body.push_str(r#"<div class="mining-results-run-breakdown">"#);
    body.push_str(r#"<section class="mining-results-breakdown-section"><h3 class="mining-results-breakdown-title">Payout</h3><dl class="mining-results-payout-list">"#);
    body.push_str(&format!(
        r#"<div class="mining-results-payout-item"><dt>Mined</dt><dd>{}</dd></div><div class="mining-results-payout-item"><dt><span class="mining-results-tax-label" title="Tax is deducted before ore is added to your wallet. Container tax applies to cargo still in the robot; depot tax applies to ore already banked in the depot.">Tax</span></dt><dd>{}</dd></div><div class="mining-results-payout-item"><dt>Net</dt><dd class="mining-results-payout-net">+{}</dd></div><div class="mining-results-payout-item"><dt>Score</dt><dd>{:.1}</dd></div>"#,
        result.total_ore_mined,
        result.total_tax,
        result.total_reward,
        result.score
    ));
    body.push_str("</dl></section>");

    if !ore_results.is_empty() {
        body.push_str(r#"<section class="mining-results-breakdown-section"><h3 class="mining-results-breakdown-title">Ore breakdown</h3><ul class="mining-results-ore-list">"#);
        let mut sorted_ores: Vec<_> = ore_results.to_vec();
        sorted_ores.sort_by_key(|ore_result| std::cmp::Reverse(ore_result.ore_id));
        for ore_result in sorted_ores {
            body.push_str(&format!(
                r#"<li><span class="mining-results-ore-name">{}</span><span class="mining-results-ore-values">{} mined · {} tax · +{} net</span></li>"#,
                escape_html(&ore_result.ore_name),
                ore_result.amount,
                ore_result.tax,
                ore_result.reward,
            ));
        }
        body.push_str("</ul></section>");
    }

    render_mining_result_score_breakdown(body, result, ore_results, area_ores);

    let total_actions: i32 = action_results.iter().map(|action| action.amount).sum();
    if !action_results.is_empty() {
        body.push_str(r#"<section class="mining-results-breakdown-section"><h3 class="mining-results-breakdown-title">Actions</h3><ul class="mining-results-action-list">"#);
        let mut sorted_actions: Vec<_> = action_results.to_vec();
        sorted_actions.sort_by_key(|action| std::cmp::Reverse(action.amount));
        for action in sorted_actions {
            let percentage = if total_actions == 0 {
                0.0
            } else {
                f64::from(action.amount) * 100.0 / f64::from(total_actions)
            };
            body.push_str(&format!(
                r#"<li><span class="mining-results-action-name">{}</span><span class="mining-results-action-values">{} · {:.1}%</span></li>"#,
                action_name(action.action_type),
                action.amount,
                percentage
            ));
        }
        body.push_str(&format!(
            r#"</ul><p class="mining-results-action-total">Total actions: {}</p></section>"#,
            total_actions
        ));
    }

    body.push_str(r#"<section class="mining-results-breakdown-section"><h3 class="mining-results-breakdown-title">Timeline</h3><ul class="mining-results-timeline-list">"#);
    body.push_str(&format!(
        r#"<li><span class="mining-results-timeline-label">Queued</span><span class="mining-results-timeline-value">{}</span></li><li><span class="mining-results-timeline-label">Mining end</span><span class="mining-results-timeline-value">{}</span></li></ul></section></div>"#,
        format_utc_millis(result.creation_time_millis),
        format_utc_millis(result.mining_end_time_millis)
    ));
}

fn render_mining_result_score_breakdown(
    body: &mut String,
    result: &robominer_db::MiningResultStateRecord,
    ore_results: &[&robominer_db::MiningResultOreStateRecord],
    area_ores: &[&robominer_db::MiningResultAreaOreRecord],
) {
    let slots = scoring_slots(area_ores, ore_results);
    let amounts = ore_amounts(&[
        (0, slots[0].amount),
        (1, slots[1].amount),
        (2, slots[2].amount),
    ]);
    let breakdown = score_breakdown(amounts, result.score_ore_target);
    let tiers = [
        (&breakdown.high, slots[0]),
        (&breakdown.mid, slots[1]),
        (&breakdown.low, slots[2]),
    ];

    body.push_str(
        r#"<section class="mining-results-breakdown-section"><h3 class="mining-results-breakdown-title">Score breakdown</h3>"#,
    );
    body.push_str(&format!(
        r#"<p class="mining-results-score-target">Mining target: {} ore</p>"#,
        breakdown.ore_target
    ));
    body.push_str(
        r#"<table class="mining-results-score-table"><thead><tr><th scope="col">Ore</th><th scope="col">Mined + Overflow</th><th scope="col">Counted</th><th scope="col">Points</th><th scope="col">Overflow</th></tr></thead><tbody>"#,
    );
    for (index, (tier, slot)) in tiers.into_iter().enumerate() {
        push_score_tier_row(body, &slot, tier, index == 0);
    }
    if breakdown.residual_ore > 0 {
        push_score_table_row(
            body,
            "mining-results-score-residual",
            "Residual",
            "",
            &breakdown.residual_ore.to_string(),
            &format!("{:.1}", breakdown.residual_points),
            "",
        );
    }
    push_score_table_row(
        body,
        "mining-results-score-total",
        "Total",
        "",
        "",
        &format!("{:.1}", breakdown.total),
        "",
    );
    body.push_str("</tbody></table></section>");
}

fn push_score_tier_row(
    body: &mut String,
    slot: &ScoringSlot<'_>,
    tier: &ScoreTierBreakdown,
    highest_value_ore: bool,
) {
    let ore_name = slot
        .name
        .map(escape_html)
        .unwrap_or_else(|| "—".to_string());
    let overflow = if tier.overflow_out > 0 {
        format!("{} × 2 = {}", tier.overflow_out, tier.overflow_converted)
    } else {
        String::new()
    };
    let mined = if highest_value_ore {
        slot.amount.to_string()
    } else {
        format!(
            "{} + {} = {}",
            slot.amount,
            tier.overflow_in,
            slot.amount + tier.overflow_in
        )
    };
    push_score_table_row(
        body,
        "",
        &ore_name,
        &mined,
        &format!("{} / {}", tier.counted, tier.cap),
        &format!("{:.1}", tier.points),
        &overflow,
    );
}

fn push_score_table_row(
    body: &mut String,
    row_class: &str,
    ore: &str,
    mined: &str,
    counted: &str,
    points: &str,
    overflow: &str,
) {
    let class_attr = if row_class.is_empty() {
        String::new()
    } else {
        format!(r#" class="{row_class}""#)
    };
    body.push_str(&format!(
        r#"<tr{class_attr}><td>{}</td><td class="mining-results-score-num mining-results-score-start">{}</td><td class="mining-results-score-num mining-results-score-start">{}</td><td class="mining-results-score-num">{}</td><td class="mining-results-score-num">{}</td></tr>"#,
        ore, mined, counted, points, overflow
    ));
}

fn scoring_slots<'a>(
    area_ores: &'a [&robominer_db::MiningResultAreaOreRecord],
    ore_results: &'a [&robominer_db::MiningResultOreStateRecord],
) -> [ScoringSlot<'a>; SCORE_TIER_COUNT] {
    let mut ordered: Vec<(i64, &'a str)> = Vec::new();
    let mut push_unique = |ore_id: i64, ore_name: &'a str| {
        if ordered.iter().any(|(id, _)| *id == ore_id) || ordered.len() == SCORE_TIER_COUNT {
            return;
        }
        ordered.push((ore_id, ore_name));
    };

    if !area_ores.is_empty() {
        let mut ores = area_ores.to_vec();
        ores.sort_by_key(|ore| std::cmp::Reverse(ore.ore_id));
        for ore in ores {
            push_unique(ore.ore_id, ore.ore_name.as_str());
        }
    } else {
        let mut ores = ore_results.to_vec();
        ores.sort_by_key(|ore| std::cmp::Reverse(ore.ore_id));
        for ore in ores {
            push_unique(ore.ore_id, ore.ore_name.as_str());
        }
    }

    let mut slots = [ScoringSlot {
        name: None,
        amount: 0,
    }; SCORE_TIER_COUNT];
    for (index, (ore_id, name)) in ordered.into_iter().enumerate() {
        let amount = ore_results
            .iter()
            .find(|result| result.ore_id == ore_id)
            .map(|result| result.amount)
            .unwrap_or(0);
        slots[index] = ScoringSlot {
            name: Some(name),
            amount,
        };
    }
    slots
}

fn action_name(action_type: i32) -> &'static str {
    match action_type {
        0 => "Scan",
        1 => "Wait on CPU",
        2 => "Move forward",
        3 => "Move backward",
        4 => "Rotate right",
        5 => "Rotate left",
        6 => "Mine",
        7 => "Dump",
        _ => "",
    }
}
