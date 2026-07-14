use robominer_db::MiningAreaOreSupplyRecord;
use robominer_sim::{MAX_ORE_TYPES, OreAnimationData};

use crate::error::DomainError;
use crate::loadout::validate_ore_supply;

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
    let mut supplies = ore_supplies.to_vec();

    supplies.sort_by(|left, right| {
        right
            .ore_id
            .cmp(&left.ore_id)
            .then_with(|| left.id.cmp(&right.id))
    });

    for supply in &supplies {
        validate_ore_supply(supply)?;
        let ore_type = legacy_ore_animation_type(mining_area_id, &mut ore_data, supply.ore_id)?;
        if supply.supply > ore_data[ore_type].max_amount {
            ore_data[ore_type].max_amount = supply.supply;
        }
    }

    Ok(ore_data)
}

fn legacy_ore_animation_type(
    mining_area_id: i64,
    ore_data: &mut Vec<OreAnimationData>,
    ore_id: i64,
) -> Result<usize, DomainError> {
    if let Some(index) = ore_data
        .iter()
        .position(|known_ore| known_ore.ore_id == ore_id)
    {
        return Ok(index);
    }

    if ore_data.len() == MAX_ORE_TYPES {
        return Err(DomainError::TooManyMiningAreaOreTypes {
            mining_area_id,
            ore_type_count: ore_data.len() + 1,
        });
    }

    let index = ore_data.len();
    ore_data.push(OreAnimationData {
        ore_id,
        max_amount: 0,
    });

    Ok(index)
}
