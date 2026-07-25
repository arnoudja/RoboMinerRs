use robominer_db::MiningAreaOreSupplyRecord;
use robominer_sim::OreAnimationData;

use crate::error::DomainError;
use crate::loadout::{legacy_ore_slot, sorted_legacy_ore_supplies, validate_ore_supply};

pub(crate) fn legacy_ore_ids(
    mining_area_id: i64,
    ore_supplies: &[MiningAreaOreSupplyRecord],
) -> Result<Vec<i64>, DomainError> {
    Ok(legacy_ore_animation_data(mining_area_id, ore_supplies)?
        .into_iter()
        .map(|ore| ore.ore_id)
        .collect())
}

pub(crate) fn legacy_ore_animation_data(
    mining_area_id: i64,
    ore_supplies: &[MiningAreaOreSupplyRecord],
) -> Result<Vec<OreAnimationData>, DomainError> {
    let mut ore_data = Vec::new();
    let mut ore_ids = Vec::new();
    let supplies = sorted_legacy_ore_supplies(ore_supplies);

    for supply in &supplies {
        validate_ore_supply(supply)?;
        let ore_type = legacy_ore_slot(mining_area_id, &mut ore_ids, supply.ore_id)?;
        if ore_type == ore_data.len() {
            ore_data.push(OreAnimationData {
                ore_id: supply.ore_id,
                max_amount: 0,
            });
        }
        if supply.supply > ore_data[ore_type].max_amount {
            ore_data[ore_type].max_amount = supply.supply;
        }
    }

    Ok(ore_data)
}
