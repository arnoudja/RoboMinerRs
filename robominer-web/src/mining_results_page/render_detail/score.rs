use robominer_domain::{SCORE_TIER_COUNT, ScoreTierBreakdown, ore_amounts, score_breakdown};

use crate::help_pages;
use crate::html::EscapedHtml;

#[derive(Clone, Copy)]
struct ScoringSlot<'a> {
    name: Option<&'a str>,
    amount: i32,
}

pub(super) fn render_mining_result_score_breakdown(
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
    body.push_str(&help_pages::render_page_help_hint(
        "How is this calculated?",
        "helpMechanics#rally-score",
        "Rally score",
    ));
    body.push_str(&format!(
        r#"<p class="mining-results-score-target">Mining target: {} ore</p>"#,
        breakdown.ore_target
    ));
    body.push_str(
        r#"<div class="mining-results-score-table-wrap"><table class="mining-results-score-table"><thead><tr><th scope="col" class="mining-results-score-col-ore">Ore</th><th scope="col" class="mining-results-score-col-mined">Mined + Overflow</th><th scope="col" class="mining-results-score-col-counted">Counted</th><th scope="col" class="mining-results-score-col-points">Points</th><th scope="col" class="mining-results-score-col-overflow">Overflow</th></tr></thead><tbody>"#,
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
    body.push_str("</tbody></table></div></section>");
}

fn push_score_tier_row(
    body: &mut String,
    slot: &ScoringSlot<'_>,
    tier: &ScoreTierBreakdown,
    highest_value_ore: bool,
) {
    let ore_name = slot
        .name
        .map(|name| EscapedHtml::from(name).to_string())
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
        r#"<tr{class_attr}><td class="mining-results-score-col-ore">{}</td><td class="mining-results-score-num mining-results-score-start mining-results-score-col-mined">{}</td><td class="mining-results-score-num mining-results-score-start mining-results-score-col-counted">{}</td><td class="mining-results-score-num mining-results-score-col-points">{}</td><td class="mining-results-score-num mining-results-score-col-overflow">{}</td></tr>"#,
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
