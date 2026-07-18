use robominer_sim::{MAX_ORE_TYPES, Simulation};

use crate::constants::RALLY_SIZE;
use crate::error::DomainError;
use crate::loadout::{PoolItemOreOutcome, PoolItemOutcome, PoolLoadout, PoolRallyOutcome};

use super::legacy::legacy_ore_ids;
use super::robots::{scripted_robot_from_loadout, scripted_robot_from_loadout_source};

pub fn run_pool_loadout_with_seed(
    loadout: &PoolLoadout,
    seed: u64,
) -> Result<PoolRallyOutcome, DomainError> {
    validate_pool_loadout(loadout)?;

    let ground = loadout.mining_area.simulator_ground_with_seed(seed)?;
    let mut robots = Vec::with_capacity(RALLY_SIZE);

    for entry in &loadout.items {
        robots.push(scripted_robot_from_loadout_source(
            &entry.robot,
            Some(-1),
            entry.source_code(),
        )?);
    }

    for _ in loadout.items.len()..RALLY_SIZE {
        robots.push(scripted_robot_from_loadout(
            &loadout.mining_area.ai_robot,
            Some(-1),
        )?);
    }

    let ore_ids = legacy_ore_ids(
        loadout.mining_area.area.id,
        &loadout.mining_area.ore_supplies,
    )?;
    let mut simulation =
        Simulation::new_with_ore_ids(ground, loadout.mining_area.area.max_moves, robots, ore_ids);
    simulation.run();

    pool_rally_outcome(loadout, simulation)
}

pub(crate) fn validate_pool_outcome(
    loadout: &PoolLoadout,
    outcome: &PoolRallyOutcome,
) -> Result<(), DomainError> {
    validate_pool_loadout(loadout)?;

    if outcome.pool_id != loadout.pool.id
        || outcome.mining_area_id != loadout.mining_area.area.id
        || outcome.items.len() != loadout.items.len()
        || loadout.items.iter().any(|item| {
            !outcome
                .items
                .iter()
                .any(|outcome_item| outcome_item.pool_item_id == item.item.id)
        })
    {
        return Err(DomainError::PoolOutcomeMismatch {
            pool_id: loadout.pool.id,
        });
    }

    Ok(())
}

fn validate_pool_loadout(loadout: &PoolLoadout) -> Result<(), DomainError> {
    if loadout.items.is_empty() || loadout.items.len() > RALLY_SIZE {
        return Err(DomainError::InvalidPoolLoadout {
            pool_id: loadout.pool.id,
            items: loadout.items.len(),
        });
    }

    Ok(())
}

fn pool_rally_outcome(
    loadout: &PoolLoadout,
    simulation: Simulation,
) -> Result<PoolRallyOutcome, DomainError> {
    let final_time = simulation.time();
    let ore_ids = legacy_ore_ids(
        loadout.mining_area.area.id,
        &loadout.mining_area.ore_supplies,
    )?;
    let items = loadout
        .items
        .iter()
        .enumerate()
        .map(|(player_number, item)| {
            let robot = simulation.robot(player_number);
            PoolItemOutcome {
                player_number,
                pool_item_id: item.item.id,
                robot_id: item.robot.robot.id,
                score: robot.calculate_score(),
                ore_results: pool_item_ore_results(&ore_ids, &robot.result_ore()),
            }
        })
        .collect();

    Ok(PoolRallyOutcome {
        pool_id: loadout.pool.id,
        mining_area_id: loadout.mining_area.area.id,
        final_time,
        items,
    })
}

fn pool_item_ore_results(ore_ids: &[i64], ore: &[i32; MAX_ORE_TYPES]) -> Vec<PoolItemOreOutcome> {
    ore_ids
        .iter()
        .copied()
        .zip(ore.iter().copied())
        .filter_map(|(ore_id, amount)| {
            if amount > 0 {
                Some(PoolItemOreOutcome { ore_id, amount })
            } else {
                None
            }
        })
        .collect()
}
