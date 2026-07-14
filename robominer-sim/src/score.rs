use std::array;

use crate::MAX_ORE_TYPES;

pub fn calculate_score(ore: [i32; MAX_ORE_TYPES]) -> f64 {
    let high_ore = ore[0];
    let mut med_ore = ore[1];
    let mut low_ore = ore[2];

    let mut score = high_ore.min(30) as f64 * 30.0;

    if high_ore > 30 {
        med_ore += (high_ore - 30) * 2;
    }

    score += med_ore.min(90) as f64;

    if med_ore > 90 {
        low_ore += (med_ore - 90) * 2;
    }

    score += low_ore.min(360) as f64 / 40.0;
    low_ore -= 360;

    let mut factor = 0.01;
    while low_ore > 0 {
        score += low_ore.min(90) as f64 * factor;
        low_ore -= 90;
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
