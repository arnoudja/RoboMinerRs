use std::collections::HashMap;

pub(crate) fn area_costs_affordable(
    costs: &[&robominer_db::MiningQueuePageAreaCostRecord],
    ore_amount_map: &HashMap<i64, i32>,
) -> bool {
    let pairs: Vec<_> = costs
        .iter()
        .map(|cost| (cost.ore_id, cost.amount))
        .collect();
    crate::html::ore_costs_affordable(&pairs, ore_amount_map)
}

pub(crate) fn render_area_entry_costs(
    costs: &[&robominer_db::MiningQueuePageAreaCostRecord],
    ore_amount_map: &HashMap<i64, i32>,
) -> String {
    let triples: Vec<_> = costs
        .iter()
        .map(|cost| (cost.ore_id, cost.amount, cost.ore_name.as_str()))
        .collect();
    crate::html::render_ore_entry_costs(
        &triples,
        ore_amount_map,
        "mining-area-atlas-cost-affordable",
        "mining-area-atlas-cost-unaffordable",
    )
}
