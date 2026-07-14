use robominer_db::{
    CompletedPoolItemOreRecord, CompletedPoolItemRecord, CompletedPoolRallyRecord,
    CompletedRallyActionRecord, CompletedRallyOreRecord, CompletedRallyParticipantRecord,
    CompletedRallyRecord, MySqlPool,
};
use robominer_sim::MAX_ORE_TYPES;

use crate::error::DomainError;
use crate::loadout::{PoolLoadout, PoolRallyOutcome, RallyLoadout, RallyOutcome};

use super::legacy::legacy_ore_ids;
use super::pool::validate_pool_outcome;
use super::rally::validate_rally_outcome;

pub async fn persist_rally_outcome(
    pool: &MySqlPool,
    loadout: &RallyLoadout,
    outcome: &RallyOutcome,
    result_data: &str,
) -> Result<i64, DomainError> {
    let record = completed_rally_record(loadout, outcome, result_data)?;

    robominer_db::persist_completed_rally(pool, &record)
        .await
        .map_err(DomainError::Database)
}

pub async fn persist_pool_rally_outcome(
    pool: &MySqlPool,
    loadout: &PoolLoadout,
    outcome: &PoolRallyOutcome,
) -> Result<(), DomainError> {
    let record = completed_pool_rally_record(loadout, outcome)?;

    robominer_db::persist_completed_pool_rally(pool, &record)
        .await
        .map_err(DomainError::Database)
}

pub fn completed_rally_record(
    loadout: &RallyLoadout,
    outcome: &RallyOutcome,
    result_data: &str,
) -> Result<CompletedRallyRecord, DomainError> {
    validate_rally_outcome(loadout, outcome)?;

    let ore_ids = legacy_ore_ids(
        loadout.mining_area.area.id,
        &loadout.mining_area.ore_supplies,
    )?;
    let mut participants = Vec::with_capacity(loadout.queue_entries.len());

    for entry in &loadout.queue_entries {
        let Some(outcome_participant) = outcome
            .participants
            .iter()
            .find(|participant| participant.queue_id == Some(entry.queue.queue.id))
        else {
            return Err(DomainError::RallyOutcomeMismatch {
                mining_area_id: loadout.mining_area.area.id,
            });
        };

        participants.push(CompletedRallyParticipantRecord {
            mining_queue_id: entry.queue.queue.id,
            robot_id: entry.robot.robot.id,
            mining_area_id: loadout.mining_area.area.id,
            player_number: i32::try_from(outcome_participant.player_number).map_err(|_| {
                DomainError::RallyOutcomeMismatch {
                    mining_area_id: loadout.mining_area.area.id,
                }
            })?,
            mining_end_seconds_from_now: entry.queue.seconds_left,
            score: outcome_participant.score,
            ore_results: completed_ore_results(&ore_ids, &outcome_participant.ore),
            action_results: completed_action_results(&outcome_participant.actions_done),
        });
    }

    Ok(CompletedRallyRecord {
        result_data: result_data.to_string(),
        participants,
    })
}

pub fn completed_pool_rally_record(
    loadout: &PoolLoadout,
    outcome: &PoolRallyOutcome,
) -> Result<CompletedPoolRallyRecord, DomainError> {
    validate_pool_outcome(loadout, outcome)?;

    let mut items = Vec::with_capacity(loadout.items.len());

    for item in &loadout.items {
        let Some(outcome_item) = outcome
            .items
            .iter()
            .find(|outcome_item| outcome_item.pool_item_id == item.item.id)
        else {
            return Err(DomainError::PoolOutcomeMismatch {
                pool_id: loadout.pool.id,
            });
        };

        items.push(CompletedPoolItemRecord {
            pool_item_id: item.item.id,
            score: outcome_item.score,
            ore_results: outcome_item
                .ore_results
                .iter()
                .filter(|ore_result| ore_result.amount > 0)
                .map(|ore_result| CompletedPoolItemOreRecord {
                    ore_id: ore_result.ore_id,
                    amount: ore_result.amount,
                })
                .collect(),
        });
    }

    Ok(CompletedPoolRallyRecord { items })
}

fn completed_ore_results(
    ore_ids: &[i64],
    ore: &[i32; MAX_ORE_TYPES],
) -> Vec<CompletedRallyOreRecord> {
    ore.iter()
        .enumerate()
        .filter_map(|(index, amount)| {
            if *amount > 0 {
                ore_ids.get(index).map(|ore_id| CompletedRallyOreRecord {
                    ore_id: *ore_id,
                    amount: *amount,
                })
            } else {
                None
            }
        })
        .collect()
}

fn completed_action_results(actions_done: &[i32; 8]) -> Vec<CompletedRallyActionRecord> {
    actions_done
        .iter()
        .enumerate()
        .filter_map(|(action_type, amount)| {
            if *amount > 0 {
                Some(CompletedRallyActionRecord {
                    action_type: action_type as i32,
                    amount: *amount,
                })
            } else {
                None
            }
        })
        .collect()
}
