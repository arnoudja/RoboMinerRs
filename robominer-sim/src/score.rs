use std::array;

use crate::MAX_ORE_TYPES;

/// Score haul using area `score_ore_target` (T). Thresholds scale with T; full tier
/// points stay 900 / 90 / 9 / …. When T=30 this matches the legacy hardcoded formula.
/// Values of T below 1 are treated as 1.
pub fn calculate_score(ore: [i32; MAX_ORE_TYPES], score_ore_target: i32) -> f64 {
    let t = score_ore_target.max(1);
    let mid_cap = 3 * t;
    let low_cap = 12 * t;
    let band = 3 * t;

    let high_ore = ore[0];
    let mut med_ore = ore[1];
    let mut low_ore = ore[2];

    let mut score = high_ore.min(t) as f64 * (900.0 / f64::from(t));

    if high_ore > t {
        med_ore += (high_ore - t) * 2;
    }

    score += med_ore.min(mid_cap) as f64 * (90.0 / f64::from(mid_cap));

    if med_ore > mid_cap {
        low_ore += (med_ore - mid_cap) * 2;
    }

    score += low_ore.min(low_cap) as f64 * (9.0 / f64::from(low_cap));
    low_ore -= low_cap;

    let mut factor = 0.01;
    while low_ore > 0 {
        score += low_ore.min(band) as f64 * factor;
        low_ore -= band;
        factor /= 10.0;
    }

    score
}

pub fn ore_amounts(amounts: &[(usize, i32)]) -> [i32; MAX_ORE_TYPES] {
    let mut ore = array::from_fn(|_| 0);

    for (ore_type, amount) in amounts {
        ore[*ore_type] = *amount;
    }

    ore
}
