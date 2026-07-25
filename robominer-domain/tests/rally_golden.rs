mod support;

use robominer_domain::{completed_rally_record, run_rally_loadout_with_animation_seed};
use robominer_sim::{ANIMATION_PAYLOAD_VERSION, AnimationPayload};
use robominer_test_support::{
    load_fixture, round_golden_coord, round_golden_score, update_golden_enabled, write_fixture,
};
use serde::{Deserialize, Serialize};
use support::{RallyScenario, RallyScenarioId};

const MANIFEST_DIR: &str = env!("CARGO_MANIFEST_DIR");
const FIXTURE_SUBDIR: &str = "rally";
const UPDATE_ENV_VAR: &str = "UPDATE_RALLY_GOLDEN";

#[derive(Debug, Serialize, Deserialize, PartialEq)]
struct GoldenPosition {
    x: f64,
    y: f64,
    orientation: i32,
}

#[derive(Debug, Serialize, Deserialize, PartialEq)]
struct GoldenParticipant {
    player_number: usize,
    queue_id: Option<i64>,
    robot_id: i64,
    is_ai: bool,
    position: GoldenPosition,
    ore: Vec<i32>,
    score: f64,
    actions_done: Vec<i32>,
}

#[derive(Debug, Serialize, Deserialize, PartialEq)]
struct GoldenOreResult {
    ore_id: i64,
    amount: i32,
}

#[derive(Debug, Serialize, Deserialize, PartialEq)]
struct GoldenActionResult {
    action_type: i32,
    amount: i32,
}

#[derive(Debug, Serialize, Deserialize, PartialEq)]
struct GoldenCompletedParticipant {
    mining_queue_id: i64,
    robot_id: i64,
    mining_area_id: i64,
    player_number: i32,
    mining_end_seconds_from_now: i32,
    score: f64,
    ore_results: Vec<GoldenOreResult>,
    action_results: Vec<GoldenActionResult>,
}

#[derive(Debug, Serialize, Deserialize, PartialEq)]
struct GoldenRallyFixture {
    scenario: String,
    seed: u64,
    mining_area_id: i64,
    final_time: i32,
    participants: Vec<GoldenParticipant>,
    completed_participants: Vec<GoldenCompletedParticipant>,
}

struct BuiltRallyFixture {
    fixture: GoldenRallyFixture,
    animation_data: String,
}

fn build_fixture(scenario: &RallyScenario) -> BuiltRallyFixture {
    let run = run_rally_loadout_with_animation_seed(&scenario.loadout, scenario.seed)
        .expect("golden scenario animation rally should run");
    let outcome = &run.outcome;
    let record = completed_rally_record(&scenario.loadout, outcome, &run.result_data)
        .expect("golden scenario completed rally record should map");

    BuiltRallyFixture {
        fixture: GoldenRallyFixture {
            scenario: scenario.name.to_string(),
            seed: scenario.seed,
            mining_area_id: outcome.mining_area_id,
            final_time: outcome.final_time,
            participants: outcome
                .participants
                .iter()
                .map(|participant| GoldenParticipant {
                    player_number: participant.player_number,
                    queue_id: participant.queue_id,
                    robot_id: participant.robot_id,
                    is_ai: participant.is_ai,
                    position: GoldenPosition {
                        x: round_golden_coord(participant.position.x),
                        y: round_golden_coord(participant.position.y),
                        orientation: participant.position.orientation,
                    },
                    ore: participant.ore.to_vec(),
                    score: round_golden_score(participant.score),
                    actions_done: participant.actions_done.to_vec(),
                })
                .collect(),
            completed_participants: record
                .participants
                .iter()
                .map(|participant| GoldenCompletedParticipant {
                    mining_queue_id: participant.mining_queue_id,
                    robot_id: participant.robot_id,
                    mining_area_id: participant.mining_area_id,
                    player_number: participant.player_number,
                    mining_end_seconds_from_now: participant.mining_end_seconds_from_now,
                    score: round_golden_score(participant.score),
                    ore_results: participant
                        .ore_results
                        .iter()
                        .map(|ore| GoldenOreResult {
                            ore_id: ore.ore_id,
                            amount: ore.amount,
                        })
                        .collect(),
                    action_results: participant
                        .action_results
                        .iter()
                        .map(|action| GoldenActionResult {
                            action_type: action.action_type,
                            amount: action.amount,
                        })
                        .collect(),
                })
                .collect(),
        },
        animation_data: run.result_data,
    }
}

fn assert_animation_payload(scenario: &RallyScenario, animation_data: &str) {
    let payload = AnimationPayload::parse(animation_data).unwrap_or_else(|error| {
        panic!(
            "scenario {} animation payload should parse: {error}",
            scenario.name
        )
    });
    assert_eq!(
        payload.v, ANIMATION_PAYLOAD_VERSION,
        "scenario {} animation version",
        scenario.name
    );
    assert_eq!(
        payload.robots.robot.len(),
        4,
        "scenario {} should animate four rally slots",
        scenario.name
    );
    assert_eq!(
        payload.ground.size_x, scenario.loadout.mining_area.area.size_x as usize,
        "scenario {} ground sizeX",
        scenario.name
    );
    assert_eq!(
        payload.ground.size_y, scenario.loadout.mining_area.area.size_y as usize,
        "scenario {} ground sizeY",
        scenario.name
    );
    assert!(
        !payload.ore_types.is_empty() || scenario.loadout.mining_area.ore_supplies.is_empty(),
        "scenario {} oreTypes",
        scenario.name
    );

    let expects_depot = scenario
        .loadout
        .queue_entries
        .iter()
        .any(|entry| entry.robot.depot_capacity.iter().any(|&cap| cap > 0));
    if expects_depot {
        assert!(
            payload
                .robots
                .robot
                .iter()
                .any(|robot| robot.depot_max_a.is_some()),
            "scenario {} should include depotMax fields",
            scenario.name
        );
        assert!(
            payload.robots.robot.iter().any(|robot| robot
                .locations
                .iter()
                .any(|location| location.depot_a.is_some())),
            "scenario {} should include depot amount samples",
            scenario.name
        );
    }
}

#[test]
fn rally_outcomes_match_golden_fixtures() {
    if update_golden_enabled(UPDATE_ENV_VAR) {
        for id in RallyScenarioId::ALL {
            let scenario = id.build();
            write_fixture(
                MANIFEST_DIR,
                FIXTURE_SUBDIR,
                id.as_str(),
                &build_fixture(&scenario).fixture,
            );
        }
        return;
    }

    for id in RallyScenarioId::ALL {
        let scenario = id.build();
        let expected: GoldenRallyFixture = load_fixture(MANIFEST_DIR, FIXTURE_SUBDIR, id.as_str());
        let built = build_fixture(&scenario);
        let actual = built.fixture;

        assert_eq!(expected.scenario, actual.scenario);
        assert_eq!(expected.seed, actual.seed);
        assert_eq!(expected.mining_area_id, actual.mining_area_id);
        assert_eq!(expected.final_time, actual.final_time);
        assert_eq!(expected.participants, actual.participants);
        assert_eq!(
            expected.completed_participants,
            actual.completed_participants
        );
        assert_animation_payload(&scenario, &built.animation_data);
    }
}
