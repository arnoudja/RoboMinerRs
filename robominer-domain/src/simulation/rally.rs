use robominer_sim::Simulation;

use crate::constants::RALLY_SIZE;
use crate::error::DomainError;
use crate::loadout::{RallyLoadout, RallyOutcome, RallyParticipantOutcome, RallyRun};

use super::legacy::{legacy_ore_animation_data, legacy_ore_ids};
use super::robots::scripted_robot_from_loadout;

pub fn run_rally_loadout_with_seed(
    loadout: &RallyLoadout,
    seed: u64,
) -> Result<RallyOutcome, DomainError> {
    Ok(run_rally_simulation(loadout, seed, false)?.outcome)
}

pub fn run_rally_loadout_with_animation_seed(
    loadout: &RallyLoadout,
    seed: u64,
) -> Result<RallyRun, DomainError> {
    run_rally_simulation(loadout, seed, true)
}

fn run_rally_simulation(
    loadout: &RallyLoadout,
    seed: u64,
    record_animation: bool,
) -> Result<RallyRun, DomainError> {
    validate_rally_loadout(loadout)?;

    let ground = loadout.mining_area.simulator_ground_with_seed(seed)?;
    let mut participants = Vec::with_capacity(RALLY_SIZE);
    let mut robots = Vec::with_capacity(RALLY_SIZE);

    for entry in &loadout.queue_entries {
        participants.push(RallyParticipant {
            queue_id: Some(entry.queue.queue.id),
            robot_id: entry.robot.robot.id,
            is_ai: false,
        });
        robots.push(scripted_robot_from_loadout(&entry.robot, None)?);
    }

    for _ in loadout.queue_entries.len()..RALLY_SIZE {
        participants.push(RallyParticipant {
            queue_id: None,
            robot_id: loadout.mining_area.ai_robot.robot.id,
            is_ai: true,
        });
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
    let result_data = if record_animation {
        let ore_data = legacy_ore_animation_data(
            loadout.mining_area.area.id,
            &loadout.mining_area.ore_supplies,
        )?;

        simulation.run_with_animation(&ore_data)
    } else {
        simulation.run();
        String::new()
    };

    Ok(RallyRun {
        outcome: rally_outcome(loadout.mining_area.area.id, simulation, participants),
        result_data,
    })
}

pub(crate) fn validate_rally_outcome(
    loadout: &RallyLoadout,
    outcome: &RallyOutcome,
) -> Result<(), DomainError> {
    validate_rally_loadout(loadout)?;

    if outcome.mining_area_id != loadout.mining_area.area.id
        || outcome.participants.len() != RALLY_SIZE
        || loadout.queue_entries.iter().any(|entry| {
            !outcome
                .participants
                .iter()
                .any(|participant| participant.queue_id == Some(entry.queue.queue.id))
        })
    {
        return Err(DomainError::RallyOutcomeMismatch {
            mining_area_id: loadout.mining_area.area.id,
        });
    }

    Ok(())
}

fn validate_rally_loadout(loadout: &RallyLoadout) -> Result<(), DomainError> {
    if loadout.queue_entries.is_empty() || loadout.queue_entries.len() > RALLY_SIZE {
        return Err(DomainError::InvalidRallyLoadout {
            mining_area_id: loadout.mining_area.area.id,
            queue_entries: loadout.queue_entries.len(),
        });
    }

    Ok(())
}

#[derive(Clone, Debug, PartialEq, Eq)]
struct RallyParticipant {
    queue_id: Option<i64>,
    robot_id: i64,
    is_ai: bool,
}

fn rally_outcome(
    mining_area_id: i64,
    simulation: Simulation,
    participants: Vec<RallyParticipant>,
) -> RallyOutcome {
    let final_time = simulation.time();
    let participants = participants
        .into_iter()
        .enumerate()
        .map(|(player_number, participant)| {
            let robot = simulation.robot(player_number);
            RallyParticipantOutcome {
                player_number,
                queue_id: participant.queue_id,
                robot_id: participant.robot_id,
                is_ai: participant.is_ai,
                position: robot.position(),
                ore: *robot.ore(),
                score: robot.calculate_score(),
                actions_done: *robot.actions_done(),
            }
        })
        .collect();

    RallyOutcome {
        mining_area_id,
        final_time,
        participants,
    }
}
