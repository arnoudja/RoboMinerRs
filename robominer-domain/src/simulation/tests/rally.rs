use robominer_db::{CompletedRallyActionRecord, CompletedRallyOreRecord};
use robominer_sim::{MAX_ORE_TYPES, Position};

use crate::error::DomainError;
use crate::loadout::{
    MiningAreaLoadout, RallyLoadout, RallyOutcome, RallyParticipantOutcome, RallyQueueEntry,
    RobotLoadout, RobotLoadoutParts,
};
use crate::simulation::{
    completed_rally_record, run_rally_loadout_with_animation_seed, run_rally_loadout_with_seed,
};
use robominer_test_support::{
    mining_rally_queue_record, ore_supply_record, robot_record, unit_test_mining_area_record,
    unit_test_robot_record,
};

#[test]
fn run_rally_loadout_returns_outcomes_for_queued_and_ai_robots() {
    let mut area = unit_test_mining_area_record(1001);
    area.max_moves = 3;
    let mining_area = MiningAreaLoadout::new(
        area,
        vec![ore_supply_record(1, 1001, 1, 10, 2)],
        RobotLoadout::new(
            unit_test_robot_record(1, "rotate(90);"),
            RobotLoadoutParts::empty(),
        ),
    );
    let queue_entries = vec![RallyQueueEntry::new(
        mining_rally_queue_record(10, 1001, 11, 9),
        RobotLoadout::new(
            unit_test_robot_record(11, "mine();"),
            RobotLoadoutParts::empty(),
        ),
    )];
    let rally = RallyLoadout::new(mining_area, queue_entries);

    let outcome = run_rally_loadout_with_seed(&rally, 0).expect("rally should run");

    assert_eq!(outcome.mining_area_id, 1001);
    assert_eq!(outcome.final_time, 3);
    assert_eq!(outcome.participants.len(), 4);
    assert_eq!(outcome.participants[0].player_number, 0);
    assert_eq!(outcome.participants[0].queue_id, Some(10));
    assert_eq!(outcome.participants[0].robot_id, 11);
    assert!(!outcome.participants[0].is_ai);
    assert!(outcome.participants[0].ore[0] > 0);
    assert!(outcome.participants[0].score > 0.0);
    assert_eq!(outcome.participants[0].actions_done[6], 3);

    assert_eq!(outcome.participants[1].queue_id, None);
    assert_eq!(outcome.participants[1].robot_id, 1);
    assert!(outcome.participants[1].is_ai);
}

#[test]
fn run_rally_loadout_with_animation_returns_versioned_json_result_data() {
    let mut area = unit_test_mining_area_record(1001);
    area.max_moves = 1;
    let mining_area = MiningAreaLoadout::new(
        area,
        vec![ore_supply_record(1, 1001, 1, 10, 2)],
        RobotLoadout::new(
            unit_test_robot_record(1, "rotate(90);"),
            RobotLoadoutParts::empty(),
        ),
    );
    let queue_entries = vec![RallyQueueEntry::new(
        mining_rally_queue_record(10, 1001, 11, 9),
        RobotLoadout::new(
            unit_test_robot_record(11, "mine();"),
            RobotLoadoutParts::empty(),
        ),
    )];
    let rally = RallyLoadout::new(mining_area, queue_entries);

    let run = run_rally_loadout_with_animation_seed(&rally, 0).expect("rally should run");

    assert_eq!(run.outcome.mining_area_id, 1001);
    let payload: serde_json::Value =
        serde_json::from_str(&run.result_data).expect("result data should be JSON");
    assert_eq!(payload["v"], 1);
    assert!(
        payload["robots"]["robot"]
            .as_array()
            .is_some_and(|r| !r.is_empty())
    );
    assert_eq!(payload["ground"]["sizeX"], 4);
    assert_eq!(payload["ground"]["sizeY"], 4);
    assert_eq!(payload["oreTypes"]["A"]["id"], 1);
    assert_eq!(payload["oreTypes"]["A"]["max"], 10);
}

#[test]
fn run_rally_loadout_with_depot_capacity_banks_home_dump_in_haul_and_animation() {
    let mut area = unit_test_mining_area_record(1001);
    area.max_moves = 8;
    let mining_area = MiningAreaLoadout::new(
        area,
        vec![ore_supply_record(1, 1001, 1, 10, 2)],
        RobotLoadout::new(
            unit_test_robot_record(1, "rotate(90);"),
            RobotLoadoutParts::empty(),
        ),
    );

    let mut depot_capacity = [0; MAX_ORE_TYPES];
    depot_capacity[0] = 10;
    let with_depot = RallyLoadout::new(
        mining_area.clone(),
        vec![RallyQueueEntry::new(
            mining_rally_queue_record(10, 1001, 11, 9),
            RobotLoadout::new(
                unit_test_robot_record(11, "mine(); dump(0);"),
                RobotLoadoutParts::empty(),
            )
            .with_depot_capacity(depot_capacity),
        )],
    );
    let without_depot = RallyLoadout::new(
        mining_area,
        vec![RallyQueueEntry::new(
            mining_rally_queue_record(10, 1001, 11, 9),
            RobotLoadout::new(
                unit_test_robot_record(11, "mine(); dump(0);"),
                RobotLoadoutParts::empty(),
            ),
        )],
    );

    let banked = run_rally_loadout_with_animation_seed(&with_depot, 0).expect("rally should run");
    let spilled = run_rally_loadout_with_seed(&without_depot, 0).expect("rally should run");

    assert!(
        banked.outcome.participants[0].ore[0] > 0,
        "home dump should bank ore in the depot haul"
    );
    assert!(banked.outcome.participants[0].score > 0.0);
    assert_eq!(
        spilled.participants[0].ore[0], 0,
        "without depot capacity, home dump should spill cargo onto the ground"
    );

    let payload: serde_json::Value =
        serde_json::from_str(&banked.result_data).expect("result data should be JSON");
    let player = &payload["robots"]["robot"][0];
    assert_eq!(player["depotMaxA"], 10);
    assert_eq!(player["depotMaxB"], 0);
    assert_eq!(player["depotMaxC"], 0);
    assert!(player.get("homeSize").is_some());
    let locations = player["locations"].as_array().expect("locations");
    let max_depot_a = locations
        .iter()
        .filter_map(|step| step["DA"].as_i64())
        .max()
        .unwrap_or(0);
    assert!(
        max_depot_a > 0,
        "animation should record depot fill after home dump"
    );
    assert!(
        payload["robots"]["robot"][1].get("depotMaxA").is_none(),
        "AI fill robots should not record depot capacity"
    );
}

#[test]
fn run_rally_loadout_rejects_empty_or_oversized_queue_entries() {
    let mining_area = MiningAreaLoadout::new(
        unit_test_mining_area_record(1001),
        vec![ore_supply_record(1, 1001, 1, 10, 1)],
        RobotLoadout::new(robot_record(1), RobotLoadoutParts::empty()),
    );
    let empty = RallyLoadout::new(mining_area.clone(), Vec::new());
    let too_many = RallyLoadout::new(
        mining_area,
        (0..5)
            .map(|index| {
                RallyQueueEntry::new(
                    mining_rally_queue_record(10 + index, 1001, 11 + index, 9),
                    RobotLoadout::new(robot_record(11 + index), RobotLoadoutParts::empty()),
                )
            })
            .collect(),
    );

    assert!(matches!(
        run_rally_loadout_with_seed(&empty, 0).unwrap_err(),
        DomainError::InvalidRallyLoadout { .. }
    ));
    assert!(matches!(
        run_rally_loadout_with_seed(&too_many, 0).unwrap_err(),
        DomainError::InvalidRallyLoadout { .. }
    ));
}

#[test]
fn run_rally_loadout_reports_program_compile_errors() {
    let mining_area = MiningAreaLoadout::new(
        unit_test_mining_area_record(1001),
        vec![ore_supply_record(1, 1001, 1, 10, 1)],
        RobotLoadout::new(robot_record(1), RobotLoadoutParts::empty()),
    );
    let rally = RallyLoadout::new(
        mining_area,
        vec![RallyQueueEntry::new(
            mining_rally_queue_record(10, 1001, 11, 9),
            RobotLoadout::new(
                unit_test_robot_record(11, "this is not valid;"),
                RobotLoadoutParts::empty(),
            ),
        )],
    );

    let error = run_rally_loadout_with_seed(&rally, 0).unwrap_err();

    assert!(matches!(
        error,
        DomainError::ProgramCompile { robot_id: 11, .. }
    ));
}

#[test]
fn completed_rally_record_maps_outcome_to_legacy_write_rows() {
    let mining_area = MiningAreaLoadout::new(
        unit_test_mining_area_record(1001),
        vec![
            ore_supply_record(1, 1001, 1, 10, 1),
            ore_supply_record(2, 1001, 3, 30, 1),
        ],
        RobotLoadout::new(robot_record(1), RobotLoadoutParts::empty()),
    );
    let rally = RallyLoadout::new(
        mining_area,
        vec![RallyQueueEntry::new(
            mining_rally_queue_record(10, 1001, 11, 9),
            RobotLoadout::new(robot_record(11), RobotLoadoutParts::empty()),
        )],
    );
    let mut actions_done = [0; 8];
    actions_done[2] = 3;
    actions_done[6] = 1;
    let outcome = RallyOutcome {
        mining_area_id: 1001,
        final_time: 3,
        participants: vec![
            RallyParticipantOutcome {
                player_number: 0,
                queue_id: Some(10),
                robot_id: 11,
                is_ai: false,
                position: Position::default(),
                ore: robominer_sim::ore_amounts(&[(0, 7), (1, 5)]),
                score: 42.5,
                actions_done,
            },
            ai_participant_outcome(1),
            ai_participant_outcome(2),
            ai_participant_outcome(3),
        ],
    };

    let record =
        completed_rally_record(&rally, &outcome, "animation-data").expect("record should map");

    assert_eq!(record.result_data, "animation-data");
    assert_eq!(record.participants.len(), 1);
    let participant = &record.participants[0];
    assert_eq!(participant.mining_queue_id, 10);
    assert_eq!(participant.robot_id, 11);
    assert_eq!(participant.mining_area_id, 1001);
    assert_eq!(participant.player_number, 0);
    assert_eq!(participant.mining_end_seconds_from_now, 9);
    assert_eq!(participant.score, 42.5);
    assert_eq!(
        participant.ore_results,
        vec![
            CompletedRallyOreRecord {
                ore_id: 3,
                amount: 7
            },
            CompletedRallyOreRecord {
                ore_id: 1,
                amount: 5
            },
        ]
    );
    assert_eq!(
        participant.action_results,
        vec![
            CompletedRallyActionRecord {
                action_type: 2,
                amount: 3
            },
            CompletedRallyActionRecord {
                action_type: 6,
                amount: 1
            },
        ]
    );
}

#[test]
fn completed_rally_record_rejects_mismatched_outcomes() {
    let mining_area = MiningAreaLoadout::new(
        unit_test_mining_area_record(1001),
        vec![ore_supply_record(1, 1001, 1, 10, 1)],
        RobotLoadout::new(robot_record(1), RobotLoadoutParts::empty()),
    );
    let rally = RallyLoadout::new(
        mining_area,
        vec![RallyQueueEntry::new(
            mining_rally_queue_record(10, 1001, 11, 9),
            RobotLoadout::new(robot_record(11), RobotLoadoutParts::empty()),
        )],
    );
    let outcome = RallyOutcome {
        mining_area_id: 1002,
        final_time: 3,
        participants: vec![
            ai_participant_outcome(0),
            ai_participant_outcome(1),
            ai_participant_outcome(2),
            ai_participant_outcome(3),
        ],
    };

    let error = completed_rally_record(&rally, &outcome, "").unwrap_err();

    assert!(matches!(error, DomainError::RallyOutcomeMismatch { .. }));
}

fn ai_participant_outcome(player_number: usize) -> RallyParticipantOutcome {
    RallyParticipantOutcome {
        player_number,
        queue_id: None,
        robot_id: 1,
        is_ai: true,
        position: Position::default(),
        ore: [0; MAX_ORE_TYPES],
        score: 0.0,
        actions_done: [0; 8],
    }
}
