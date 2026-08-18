use robominer_domain::{
    SCORE_HIGH_TIER_POINTS, SCORE_LOW_TIER_POINTS, SCORE_MID_TIER_POINTS, SCORE_SLOT_COUNT,
    ScoreBreakdown, ScoreSlot, ScoreTier, calculate_score_breakdown_from_slots,
};

use crate::html::escape_html;

const SCORE_SLOT_LABELS: [&str; SCORE_SLOT_COUNT] = ["A", "B", "C"];

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) struct ScoreHaulSlot<'a> {
    pub(super) label: &'static str,
    pub(super) ore_name: Option<&'a str>,
    pub(super) amount: i32,
}

pub(super) fn score_haul_slots<'a>(
    area_slots: &[&'a robominer_db::MiningResultAreaOreSlotRecord],
    ore_results: &[&'a robominer_db::MiningResultOreStateRecord],
) -> Vec<ScoreHaulSlot<'a>> {
    let mut unique_slots: Vec<(i64, Option<&'a str>)> = Vec::new();

    if !area_slots.is_empty() {
        for slot in area_slots {
            if !unique_slots
                .iter()
                .any(|(ore_id, _)| *ore_id == slot.ore_id)
            {
                unique_slots.push((slot.ore_id, Some(slot.ore_name.as_str())));
            }
        }
    } else {
        let mut mined = ore_results.to_vec();
        mined.sort_by(|left, right| {
            right
                .ore_id
                .cmp(&left.ore_id)
                .then_with(|| left.ore_name.cmp(&right.ore_name))
        });
        for ore in mined {
            if !unique_slots.iter().any(|(ore_id, _)| *ore_id == ore.ore_id) {
                unique_slots.push((ore.ore_id, Some(ore.ore_name.as_str())));
            }
        }
    }

    unique_slots.truncate(SCORE_SLOT_COUNT);
    unique_slots
        .into_iter()
        .enumerate()
        .map(|(index, (ore_id, ore_name))| ScoreHaulSlot {
            label: SCORE_SLOT_LABELS[index],
            ore_name,
            amount: ore_results
                .iter()
                .filter(|ore| ore.ore_id == ore_id)
                .map(|ore| ore.amount)
                .sum(),
        })
        .collect()
}

pub(super) fn render_mining_result_score_section(
    result: &robominer_db::MiningResultStateRecord,
    ore_results: &[&robominer_db::MiningResultOreStateRecord],
    area_slots: &[&robominer_db::MiningResultAreaOreSlotRecord],
) -> String {
    let haul_slots = score_haul_slots(area_slots, ore_results);
    let high = haul_amount(&haul_slots, 0);
    let mid = haul_amount(&haul_slots, 1);
    let low = haul_amount(&haul_slots, 2);
    let breakdown = calculate_score_breakdown_from_slots(high, mid, low, result.score_ore_target);

    let mut body = String::from(
        r#"<section class="mining-results-breakdown-section"><h3 class="mining-results-breakdown-title">Score calculation</h3>"#,
    );
    body.push_str(&format!(
        r#"<p class="mining-results-score-intro">Score uses hauled ore (container + depot) before tax. Slot A is the area's highest-value ore. This area's ore target is <strong>{}</strong> — filling it with A ore scores {:.0} points.</p>"#,
        breakdown.score_ore_target, SCORE_HIGH_TIER_POINTS
    ));
    body.push_str(&render_score_haul_list(&haul_slots));
    body.push_str(&render_score_steps(&breakdown));
    body.push_str(&format!(
        r#"<p class="mining-results-score-total">Total score: {:.1}</p>"#,
        result.score
    ));
    body.push_str(&format!(
        r#"<p class="mining-results-score-hint">{}</p>"#,
        escape_html(&score_improvement_hint(&breakdown))
    ));
    body.push_str(
        r#"<p class="mining-results-score-help"><a href="helpMechanics#rally-score">How rally score works</a></p></section>"#,
    );
    body
}

fn haul_amount(slots: &[ScoreHaulSlot<'_>], index: usize) -> i32 {
    slots.get(index).map(|slot| slot.amount).unwrap_or(0)
}

fn render_score_haul_list(slots: &[ScoreHaulSlot<'_>]) -> String {
    if slots.is_empty() {
        return r#"<p class="mining-results-score-empty-haul">No scoring ore was hauled this run.</p>"#
            .to_string();
    }

    let mut body = String::from(r#"<ul class="mining-results-score-haul">"#);
    for slot in slots {
        let name = slot
            .ore_name
            .map(escape_html)
            .unwrap_or_else(|| "Unknown ore".to_string());
        body.push_str(&format!(
            r#"<li><span class="mining-results-score-slot">{label}</span><span class="mining-results-score-ore">{name}</span><span class="mining-results-score-amount">{amount} hauled</span></li>"#,
            label = slot.label,
            amount = slot.amount
        ));
    }
    body.push_str("</ul>");
    body
}

fn render_score_steps(breakdown: &ScoreBreakdown) -> String {
    let mut body = String::from(r#"<ol class="mining-results-score-steps">"#);
    body.push_str(&render_score_tier_step(
        "A tier",
        &breakdown.high,
        SCORE_HIGH_TIER_POINTS,
        overflow_note_high(&breakdown.high),
    ));
    body.push_str(&render_score_tier_step(
        "B tier",
        &breakdown.mid,
        SCORE_MID_TIER_POINTS,
        overflow_note_mid(&breakdown.mid),
    ));
    body.push_str(&render_score_tier_step(
        "C tier",
        &breakdown.low,
        SCORE_LOW_TIER_POINTS,
        overflow_note_low(&breakdown.low),
    ));
    for (index, residual) in breakdown.residuals.iter().enumerate() {
        body.push_str(&format!(
            r#"<li><span class="mining-results-score-tier-name">Residual {}</span><span class="mining-results-score-tier-math">{} / {} units × {} pts = {:.1}</span></li>"#,
            index + 1,
            residual.scored_units,
            residual.cap,
            format_compact_f64(residual.points_per_unit),
            residual.points
        ));
    }
    body.push_str("</ol>");
    body
}

fn render_score_tier_step(
    name: &str,
    tier: &ScoreTier,
    full_points: f64,
    overflow_note: Option<String>,
) -> String {
    let mut item = format!(
        r#"<li><span class="mining-results-score-tier-name">{name}</span><span class="mining-results-score-tier-math">{} / {} units × {} pts = {:.1}</span>"#,
        tier.scored_units,
        tier.cap,
        format_compact_f64(tier.points_per_unit),
        tier.points
    );
    if tier.scored_units == tier.cap {
        item.push_str(&format!(
            r#"<span class="mining-results-score-tier-full">Full ({:.0} points)</span>"#,
            full_points
        ));
    }
    if let Some(note) = overflow_note {
        item.push_str(&format!(
            r#"<span class="mining-results-score-tier-note">{}</span>"#,
            escape_html(&note)
        ));
    }
    item.push_str("</li>");
    item
}

fn overflow_note_high(tier: &ScoreTier) -> Option<String> {
    if tier.overflow_units <= 0 {
        return None;
    }
    Some(format!(
        "{} extra A converted to {} B-equivalent",
        tier.overflow_units, tier.converted_out
    ))
}

fn overflow_note_mid(tier: &ScoreTier) -> Option<String> {
    if tier.converted_in > 0 && tier.overflow_units > 0 {
        Some(format!(
            "Includes {} converted from A overflow; {} extra B-equivalent converted to {} C-equivalent",
            tier.converted_in, tier.overflow_units, tier.converted_out
        ))
    } else if tier.converted_in > 0 {
        Some(format!(
            "Includes {} converted from A overflow",
            tier.converted_in
        ))
    } else if tier.overflow_units > 0 {
        Some(format!(
            "{} extra B-equivalent converted to {} C-equivalent",
            tier.overflow_units, tier.converted_out
        ))
    } else {
        None
    }
}

fn overflow_note_low(tier: &ScoreTier) -> Option<String> {
    if tier.converted_in > 0 && tier.overflow_units > 0 {
        Some(format!(
            "Includes {} converted from B overflow; {} leftover C-equivalent scored in residual bands",
            tier.converted_in, tier.overflow_units
        ))
    } else if tier.converted_in > 0 {
        Some(format!(
            "Includes {} converted from B overflow",
            tier.converted_in
        ))
    } else if tier.overflow_units > 0 {
        Some(format!(
            "{} leftover C-equivalent scored in residual bands",
            tier.overflow_units
        ))
    } else {
        None
    }
}

fn score_improvement_hint(breakdown: &ScoreBreakdown) -> String {
    if breakdown.high.equivalent_units == 0
        && breakdown.mid.equivalent_units == 0
        && breakdown.low.equivalent_units == 0
    {
        return format!(
            "This run hauled no scoring ore. Mine A ore until you reach the ore target ({} units) — that fills the top tier for {:.0} points.",
            breakdown.high.cap, SCORE_HIGH_TIER_POINTS
        );
    }

    match breakdown.next_unfilled_tier() {
        Some(ScoreSlot::High) => {
            let remaining = breakdown.high.cap - breakdown.high.scored_units;
            let gain = remaining as f64 * breakdown.high.points_per_unit;
            format!(
                "To score higher: mine {remaining} more A ore to fill the top tier (+{gain:.1} points). Each A is worth {} points until then.",
                format_compact_f64(breakdown.high.points_per_unit)
            )
        }
        Some(ScoreSlot::Mid) => {
            let remaining = breakdown.mid.cap - breakdown.mid.scored_units;
            let gain = remaining as f64 * breakdown.mid.points_per_unit;
            format!(
                "The A tier is full ({:.0} points). Extra A still helps: each extra A becomes 2 B-equivalent. The middle tier has {remaining} units left (up to +{gain:.1} points).",
                SCORE_HIGH_TIER_POINTS
            )
        }
        Some(ScoreSlot::Low) => {
            let remaining = breakdown.low.cap - breakdown.low.scored_units;
            let gain = remaining as f64 * breakdown.low.points_per_unit;
            format!(
                "A and B tiers are full ({:.0} points). Extra B converts 2:1 into C-equivalent. The low tier has {remaining} units left (up to +{gain:.1} points).",
                SCORE_HIGH_TIER_POINTS + SCORE_MID_TIER_POINTS
            )
        }
        None => format!(
            "Main scoring tiers are full ({:.0} points). Extra ore only adds tiny residual points. Keep prioritizing A ore — it is still the most valuable per unit.",
            SCORE_HIGH_TIER_POINTS + SCORE_MID_TIER_POINTS + SCORE_LOW_TIER_POINTS
        ),
    }
}

fn format_compact_f64(value: f64) -> String {
    if (value - value.round()).abs() < 1e-9 {
        return format!("{}", value.round() as i64);
    }
    format!("{value:.4}")
        .trim_end_matches('0')
        .trim_end_matches('.')
        .to_string()
}

#[cfg(test)]
mod tests {
    use super::{score_haul_slots, score_improvement_hint};
    use robominer_domain::calculate_score_breakdown_from_slots;

    fn slot(
        mining_area_id: i64,
        ore_id: i64,
        ore_name: &str,
    ) -> robominer_db::MiningResultAreaOreSlotRecord {
        robominer_db::MiningResultAreaOreSlotRecord {
            mining_area_id,
            ore_id,
            ore_name: ore_name.to_string(),
        }
    }

    fn ore(
        mining_queue_id: i64,
        ore_id: i64,
        ore_name: &str,
        amount: i32,
    ) -> robominer_db::MiningResultOreStateRecord {
        robominer_db::MiningResultOreStateRecord {
            mining_queue_id,
            ore_id,
            ore_name: ore_name.to_string(),
            amount,
            tax: 0,
            reward: amount,
        }
    }

    #[test]
    fn haul_slots_keep_unmined_high_value_ore_as_a() {
        let area_slots = [
            slot(1, 30, "Cerbonium"),
            slot(1, 20, "Ironium"),
            slot(1, 10, "Dirt"),
        ];
        let ore_results = [ore(10, 20, "Ironium", 12), ore(10, 10, "Dirt", 4)];
        let slots = score_haul_slots(
            &area_slots.iter().collect::<Vec<_>>(),
            &ore_results.iter().collect::<Vec<_>>(),
        );

        assert_eq!(slots.len(), 3);
        assert_eq!(slots[0].label, "A");
        assert_eq!(slots[0].ore_name, Some("Cerbonium"));
        assert_eq!(slots[0].amount, 0);
        assert_eq!(slots[1].amount, 12);
        assert_eq!(slots[2].amount, 4);
    }

    #[test]
    fn haul_slots_fall_back_to_mined_ore_id_order() {
        let ore_results = [ore(10, 2, "B ore", 8), ore(10, 7, "A ore", 3)];
        let slots = score_haul_slots(&[], &ore_results.iter().collect::<Vec<_>>());
        assert_eq!(slots[0].ore_name, Some("A ore"));
        assert_eq!(slots[0].amount, 3);
        assert_eq!(slots[1].ore_name, Some("B ore"));
        assert_eq!(slots[1].amount, 8);
    }

    #[test]
    fn improvement_hint_points_at_unfilled_a_tier() {
        let breakdown = calculate_score_breakdown_from_slots(12, 5, 0, 30);
        let hint = score_improvement_hint(&breakdown);
        assert!(hint.contains("18 more A ore"));
        assert!(hint.contains("+540.0 points"));
    }
}
