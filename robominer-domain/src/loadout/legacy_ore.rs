//! Shared legacy ore-slot assignment (sort order + A/B/C indexing).

use robominer_db::MiningAreaOreSupplyRecord;
use robominer_sim::MAX_ORE_TYPES;

use crate::error::DomainError;

/// Sort supplies the way the legacy Java heap placer did (ore_id desc, then id asc).
pub(crate) fn sorted_legacy_ore_supplies(
    ore_supplies: &[MiningAreaOreSupplyRecord],
) -> Vec<MiningAreaOreSupplyRecord> {
    let mut supplies = ore_supplies.to_vec();
    supplies.sort_by(|left, right| {
        right
            .ore_id
            .cmp(&left.ore_id)
            .then_with(|| left.id.cmp(&right.id))
    });
    supplies
}

/// Map an ore_id onto a dense 0-based slot list, allocating a new slot when needed.
pub(crate) fn legacy_ore_slot(
    mining_area_id: i64,
    known_ore_ids: &mut Vec<i64>,
    ore_id: i64,
) -> Result<usize, DomainError> {
    if let Some(index) = known_ore_ids
        .iter()
        .position(|known_ore_id| *known_ore_id == ore_id)
    {
        return Ok(index);
    }

    if known_ore_ids.len() == MAX_ORE_TYPES {
        return Err(DomainError::TooManyMiningAreaOreTypes {
            mining_area_id,
            ore_type_count: known_ore_ids.len() + 1,
        });
    }

    let index = known_ore_ids.len();
    known_ore_ids.push(ore_id);
    Ok(index)
}
