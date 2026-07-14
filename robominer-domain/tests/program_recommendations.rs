//! Rally program benchmarks against real mining-area layouts from `gameData.sql`.
//!
//! Use this whenever you want to compare robot programs before changing game balance,
//! recommending player strategies, or validating a new program idea.
//!
//! ```sh
//! cargo test -p robominer-domain benchmark_recommended_programs -- --nocapture
//! ```
//!
//! To benchmark custom programs or areas, edit the `programs`, `cases`, and robot
//! profile helpers in `benchmark_recommended_programs` below. Scores are printed
//! after tax; each case averages 20 seeds (0..19).

use robominer_db::{
    MiningAreaOreSupplyRecord, MiningAreaRecord, MiningRallyQueueRecord, RobotRecord,
};
use robominer_domain::{
    MiningAreaLoadout, RallyLoadout, RallyQueueEntry, RobotLoadout, RobotLoadoutParts,
    run_rally_loadout_with_seed,
};
use robominer_program::compatibility_fixture_source;
use robominer_test_support::{mining_area_record, ore_supply_record};

struct RobotProfile {
    robot: RobotRecord,
}

struct AreaProfile {
    name: &'static str,
    area: MiningAreaRecord,
    ore_supplies: Vec<MiningAreaOreSupplyRecord>,
}

struct ProgramCase {
    name: &'static str,
    source: &'static str,
}

fn starter_robot() -> RobotRecord {
    robot_with_stats("starter", 11, 2, 1, 12, 4, 1, 1.571, 0.714, 7, 1.38, 6, 1)
}

fn enhanced_cerbonium_robot() -> RobotRecord {
    robot_with_stats(
        "enhanced_cerbonium",
        20,
        5,
        2,
        15,
        6,
        2,
        1.448,
        0.621,
        8,
        1.45,
        6,
        1,
    )
}

fn cerbonium_max_robot() -> RobotRecord {
    robot_with_stats(
        "cerbonium_max",
        7,
        7,
        2,
        23,
        8,
        2,
        1.543,
        0.686,
        9,
        1.52,
        4,
        3,
    )
}

fn oxaria_mid_robot() -> RobotRecord {
    robot_with_stats(
        "oxaria_mid",
        10,
        10,
        3,
        25,
        12,
        3,
        1.535,
        1.047,
        9,
        1.56,
        3,
        4,
    )
}

fn robot_with_stats(
    name: &str,
    recharge_time: i32,
    max_ore: i32,
    mining_speed: i32,
    max_turns: i32,
    memory_size: i32,
    cpu_speed: i32,
    forward_speed: f64,
    backward_speed: f64,
    rotate_speed: i32,
    robot_size: f64,
    scan_time: i32,
    scan_distance: i32,
) -> RobotRecord {
    RobotRecord {
        id: 11,
        user_id: 3,
        robot_name: name.to_string(),
        source_code: String::new(),
        program_source_id: Some(1),
        ore_container_id: Some(101),
        mining_unit_id: Some(201),
        battery_id: Some(301),
        memory_module_id: Some(401),
        cpu_id: Some(501),
        engine_id: Some(601),
        ore_scanner_id: Some(701),
        recharge_time,
        max_ore,
        mining_speed,
        max_turns,
        memory_size,
        cpu_speed,
        forward_speed,
        backward_speed,
        rotate_speed,
        robot_size,
        scan_time,
        scan_distance,
        total_mining_runs: 0,
    }
}

fn cerbonium_mini() -> AreaProfile {
    let mut area = mining_area_record(1001, "Cerbonium-mini");
    area.ore_price_id = 10001;
    area.size_x = 10;
    area.size_y = 10;
    area.max_moves = 20;
    area.mining_time = 5;
    AreaProfile {
        name: "Cerbonium-mini (1001)",
        area,
        ore_supplies: vec![ore_supply_record(1, 1001, 1, 4, 4)],
    }
}

fn cerbonium_starter() -> AreaProfile {
    AreaProfile {
        name: "Cerbonium-Starter (1002)",
        area: {
            let mut area = mining_area_record(1002, "Cerbonium-Starter");
            area.ore_price_id = 10002;
            area.size_x = 15;
            area.size_y = 15;
            area.max_moves = 30;
            area.mining_time = 10;
            area.tax_rate = 20;
            area.ai_robot_id = 2;
            area
        },
        ore_supplies: vec![
            ore_supply_record(1, 1002, 1, 6, 6),
            ore_supply_record(2, 1002, 1, 6, 4),
        ],
    }
}

fn cerbonium_advanced() -> AreaProfile {
    AreaProfile {
        name: "Cerbonium-Advanced (1003)",
        area: {
            let mut area = mining_area_record(1003, "Cerbonium-Advanced");
            area.ore_price_id = 10003;
            area.size_x = 20;
            area.size_y = 20;
            area.max_moves = 40;
            area.mining_time = 15;
            area.tax_rate = 0;
            area.ai_robot_id = 3;
            area
        },
        ore_supplies: vec![
            ore_supply_record(1, 1003, 1, 8, 7),
            ore_supply_record(2, 1003, 1, 6, 5),
        ],
    }
}

fn oxaria_light() -> AreaProfile {
    AreaProfile {
        name: "Oxaria-Light (1101)",
        area: {
            let mut area = mining_area_record(1101, "Oxaria-Light");
            area.ore_price_id = 11001;
            area.size_x = 20;
            area.size_y = 20;
            area.max_moves = 50;
            area.mining_time = 20;
            area.ai_robot_id = 2;
            area
        },
        ore_supplies: vec![
            ore_supply_record(1, 1101, 1, 12, 6),
            ore_supply_record(2, 1101, 2, 6, 4),
            ore_supply_record(3, 1101, 2, 6, 4),
        ],
    }
}

fn oxaria_2() -> AreaProfile {
    AreaProfile {
        name: "Oxaria-2 (1102)",
        area: {
            let mut area = mining_area_record(1102, "Oxaria-2");
            area.ore_price_id = 11002;
            area.size_x = 25;
            area.size_y = 25;
            area.max_moves = 75;
            area.mining_time = 30;
            area.tax_rate = 10;
            area.ai_robot_id = 3;
            area
        },
        ore_supplies: vec![
            ore_supply_record(1, 1102, 1, 20, 6),
            ore_supply_record(2, 1102, 2, 8, 4),
            ore_supply_record(3, 1102, 2, 6, 4),
            ore_supply_record(4, 1102, 2, 6, 4),
        ],
    }
}

fn ai_robot(id: i64, source: &str) -> RobotRecord {
    let mut robot = starter_robot();
    robot.id = id;
    robot.robot_name = format!("AI-{id}");
    robot.source_code = source.to_string();
    robot.max_ore = 50;
    robot.mining_speed = 2;
    robot.max_turns = match id {
        1 => 1500,
        2 => 3000,
        _ => 5000,
    };
    robot.cpu_speed = 99;
    robot.forward_speed = 2.0;
    robot.backward_speed = 2.0;
    robot.rotate_speed = 25;
    robot.robot_size = 1.5;
    robot
}

fn rally_loadout(
    area: &AreaProfile,
    robot: &RobotRecord,
    program: &str,
    ai_source: &str,
) -> RallyLoadout {
    let ai_id = area.area.ai_robot_id;
    let mining_area = MiningAreaLoadout::new(
        area.area.clone(),
        area.ore_supplies.clone(),
        RobotLoadout::new(ai_robot(ai_id, ai_source), RobotLoadoutParts::empty()),
    );

    let mut player = robot.clone();
    player.source_code = program.to_string();

    let queue = RallyQueueEntry::new(
        MiningRallyQueueRecord {
            queue: robominer_db::MiningQueueRecord {
                id: 100,
                mining_area_id: area.area.id,
                robot_id: player.id,
                rally_result_id: None,
                player_number: Some(1),
                score: None,
                claimed: false,
            },
            user_id: player.user_id,
            seconds_left: 0,
        },
        RobotLoadout::new(player, RobotLoadoutParts::empty()),
    );

    RallyLoadout::new(mining_area, vec![queue])
}

fn taxed_score(raw_score: f64, tax_rate: i32) -> f64 {
    raw_score * (100.0 - f64::from(tax_rate)) / 100.0
}

fn benchmark_case(
    area: &AreaProfile,
    robot: &RobotProfile,
    program: &ProgramCase,
    ai_source: &str,
    seeds: &[u64],
) -> (f64, f64, f64, Vec<i32>) {
    let mut scores = Vec::new();
    let mut ore_totals = Vec::new();

    for &seed in seeds {
        let loadout = rally_loadout(area, &robot.robot, program.source, ai_source);
        let outcome = run_rally_loadout_with_seed(&loadout, seed).expect("rally should run");
        let participant = &outcome.participants[0];
        scores.push(taxed_score(participant.score, area.area.tax_rate));
        let total_ore: i32 = participant.ore.iter().sum();
        ore_totals.push(total_ore);
    }

    let avg_score = scores.iter().sum::<f64>() / scores.len() as f64;
    let min_score = scores.iter().copied().fold(f64::INFINITY, f64::min);
    let max_score = scores.iter().copied().fold(f64::NEG_INFINITY, f64::max);
    (avg_score, min_score, max_score, ore_totals)
}

#[test]
fn benchmark_recommended_programs() {
    let seeds: Vec<u64> = (0..20).map(|seed| seed as u64).collect();

    let programs = [
        ProgramCase {
            name: "default",
            source: compatibility_fixture_source("default_program"),
        },
        ProgramCase {
            name: "prog_a",
            source: compatibility_fixture_source("seed_ai_1"),
        },
        ProgramCase {
            name: "prog_b",
            source: compatibility_fixture_source("scan_then_mine"),
        },
        ProgramCase {
            name: "prog_c_ai2",
            source: compatibility_fixture_source("seed_ai_2"),
        },
        ProgramCase {
            name: "prog_d_heap",
            source: "while (true) { if (mine()) { while (mine()); } else if (move(1.42) < 0.1) { rotate(160); } else { move(1.42); } }",
        },
        ProgramCase {
            name: "prog_e_ox",
            source: "while (true) { if (mine()) { while (mine()); } else { scan(); if (oreType() == 2) { while (mine()); } else { move(1); } } }",
        },
        ProgramCase {
            name: "prog_f_ox",
            source: "while (true) { if (mine()) { while (mine()); } else { scan(); if (oreType() == 2) { while (mine()); } else if (move(1.42) < 0.1) { rotate(160); } else { move(1.42); } } }",
        },
        ProgramCase {
            name: "ai3",
            source: compatibility_fixture_source("seed_ai_3"),
        },
    ];

    println!("\n=== Program benchmark (20 seeds each, player slot 0) ===\n");

    let cases = [
        (
            cerbonium_mini(),
            starter_robot(),
            compatibility_fixture_source("seed_ai_1"),
        ),
        (
            cerbonium_mini(),
            enhanced_cerbonium_robot(),
            compatibility_fixture_source("seed_ai_1"),
        ),
        (
            cerbonium_starter(),
            enhanced_cerbonium_robot(),
            compatibility_fixture_source("seed_ai_2"),
        ),
        (
            cerbonium_starter(),
            cerbonium_max_robot(),
            compatibility_fixture_source("seed_ai_2"),
        ),
        (
            cerbonium_advanced(),
            cerbonium_max_robot(),
            compatibility_fixture_source("seed_ai_3"),
        ),
        (
            oxaria_light(),
            oxaria_mid_robot(),
            compatibility_fixture_source("seed_ai_2"),
        ),
        (
            oxaria_2(),
            oxaria_mid_robot(),
            compatibility_fixture_source("seed_ai_3"),
        ),
    ];

    for (area, robot, ai_source) in cases {
        println!("--- {} / {} ---", area.name, robot.robot_name);
        println!(
            "{:<14} {:>8} {:>8} {:>8}  ore_total(sample)",
            "program", "avg", "min", "max"
        );

        let profile = RobotProfile { robot };

        for program in &programs {
            let (avg, min, max, ore_totals) =
                benchmark_case(&area, &profile, program, ai_source, &seeds);
            println!(
                "{:<14} {:>8.1} {:>8.1} {:>8.1}  {:?}",
                program.name,
                avg,
                min,
                max,
                &ore_totals[..5.min(ore_totals.len())]
            );
        }
        println!();
    }
}
