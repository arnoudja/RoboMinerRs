mod support;

use robominer_domain::{completed_rally_record, run_rally_loadout_with_animation_seed};
use robominer_test_support::{
    load_fixture, round_golden_coord, round_golden_score, update_golden_enabled, write_fixture,
};
use serde::{Deserialize, Serialize};
use support::RallyScenario;

const MANIFEST_DIR: &str = env!("CARGO_MANIFEST_DIR");
const FIXTURE_SUBDIR: &str = "rally";
const UPDATE_ENV_VAR: &str = "UPDATE_RALLY_GOLDEN";

const SCENARIOS: &[&str] = &[
    "single_miner_seed0",
    "dual_miner_seed17",
    "animation_seed0",
    "seed_ai_1_seed42",
    "seed_ai_2_seed0",
    "seed_ai_3_seed14",
    "scan_then_mine_seed5",
    "do_while_mine_seed0",
    "triple_queue_seed33",
    "quad_queue_seed33",
    "dual_ore_seed11",
    "ore_seeker_80x80_seed0",
    "depot_dump_cerbonium_advanced_seed0",
];

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
    animation_contains: Vec<String>,
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

    let mut animation_contains = vec![
        r#""v":1"#.to_string(),
        r#""robots":{"robot":["#.to_string(),
        format!(
            r#""ground":{{"sizeX":{},"sizeY":{},"positions":["#,
            scenario.loadout.mining_area.area.size_x, scenario.loadout.mining_area.area.size_y,
        ),
        r#""oreTypes":{"#.to_string(),
    ];
    if scenario
        .loadout
        .queue_entries
        .iter()
        .any(|entry| entry.robot.depot_capacity.iter().any(|&cap| cap > 0))
    {
        animation_contains.push(r#""depotMaxA":"#.to_string());
        animation_contains.push(r#""DA":"#.to_string());
    }

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
            animation_contains,
        },
        animation_data: run.result_data,
    }
}

#[test]
fn rally_outcomes_match_golden_fixtures() {
    if update_golden_enabled(UPDATE_ENV_VAR) {
        for name in SCENARIOS {
            let scenario = support::scenario(name);
            write_fixture(
                MANIFEST_DIR,
                FIXTURE_SUBDIR,
                name,
                &build_fixture(&scenario).fixture,
            );
        }
        return;
    }

    for name in SCENARIOS {
        let scenario = support::scenario(name);
        let expected: GoldenRallyFixture = load_fixture(MANIFEST_DIR, FIXTURE_SUBDIR, name);
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

        for marker in &expected.animation_contains {
            assert!(
                built.animation_data.contains(marker),
                "scenario {name} animation data missing marker: {marker}"
            );
        }
    }
}
