use robominer_db::MiningAreaOreSupplyRecord;

use robominer_domain::{
    MiningAreaLoadout, RallyLoadout, RallyQueueEntry, RobotLoadout, RobotLoadoutParts,
};
use robominer_test_support::{
    golden_mining_area_record, golden_robot_record, mining_rally_queue_record,
    ore_seeker_robot_record, ore_supply_record,
};
use robominer_program::compatibility_fixture_source;

pub struct RallyScenario {
    pub name: &'static str,
    pub seed: u64,
    pub loadout: RallyLoadout,
}

pub fn scenario(name: &str) -> RallyScenario {
    match name {
        "single_miner_seed0" => single_miner_seed0(),
        "dual_miner_seed17" => dual_miner_seed17(),
        "animation_seed0" => animation_seed0(),
        "seed_ai_1_seed42" => seed_ai_1_seed42(),
        "seed_ai_2_seed0" => seed_ai_2_seed0(),
        "seed_ai_3_seed14" => seed_ai_3_seed14(),
        "scan_then_mine_seed5" => scan_then_mine_seed5(),
        "do_while_mine_seed0" => do_while_mine_seed0(),
        "triple_queue_seed33" => triple_queue_seed33(),
        "quad_queue_seed33" => quad_queue_seed33(),
        "dual_ore_seed11" => dual_ore_seed11(),
        "ore_seeker_80x80_seed0" => ore_seeker_80x80_seed0(),
        other => panic!("unknown rally golden scenario: {other}"),
    }
}

fn single_miner_seed0() -> RallyScenario {
    let mut area = golden_mining_area_record(1001);
    area.max_moves = 3;
    let mining_area = MiningAreaLoadout::new(
        area,
        vec![ore_supply_record(1, 1001, 1, 10, 2)],
        RobotLoadout::new(
            golden_robot_record(1, "rotate(90);"),
            RobotLoadoutParts::empty(),
        ),
    );
    let queue_entries = vec![RallyQueueEntry::new(
        mining_rally_queue_record(10, 1001, 11, 9),
        RobotLoadout::new(
            golden_robot_record(11, "mine();"),
            RobotLoadoutParts::empty(),
        ),
    )];

    RallyScenario {
        name: "single_miner_seed0",
        seed: 0,
        loadout: RallyLoadout::new(mining_area, queue_entries),
    }
}

fn dual_miner_seed17() -> RallyScenario {
    let mut area = golden_mining_area_record(2002);
    area.max_moves = 5;
    let mining_area_id = area.id;
    let mining_area = MiningAreaLoadout::new(
        area,
        vec![ore_supply_record(1, mining_area_id, 1, 10, 2)],
        RobotLoadout::new(
            golden_robot_record(1, "rotate(90);"),
            RobotLoadoutParts::empty(),
        ),
    );
    let queue_entries = vec![
        RallyQueueEntry::new(
            mining_rally_queue_record(20, mining_area_id, 21, 12),
            RobotLoadout::new(
                golden_robot_record(21, "mine();"),
                RobotLoadoutParts::empty(),
            ),
        ),
        RallyQueueEntry::new(
            mining_rally_queue_record(21, mining_area_id, 22, 15),
            RobotLoadout::new(
                golden_robot_record(22, "mine();"),
                RobotLoadoutParts::empty(),
            ),
        ),
    ];

    RallyScenario {
        name: "dual_miner_seed17",
        seed: 17,
        loadout: RallyLoadout::new(mining_area, queue_entries),
    }
}

fn animation_seed0() -> RallyScenario {
    let mut area = golden_mining_area_record(3003);
    area.max_moves = 1;
    let mining_area_id = area.id;
    let mining_area = MiningAreaLoadout::new(
        area,
        vec![ore_supply_record(1, mining_area_id, 1, 10, 2)],
        RobotLoadout::new(
            golden_robot_record(1, "rotate(90);"),
            RobotLoadoutParts::empty(),
        ),
    );
    let queue_entries = vec![RallyQueueEntry::new(
        mining_rally_queue_record(30, mining_area_id, 31, 9),
        RobotLoadout::new(
            golden_robot_record(31, "mine();"),
            RobotLoadoutParts::empty(),
        ),
    )];

    RallyScenario {
        name: "animation_seed0",
        seed: 0,
        loadout: RallyLoadout::new(mining_area, queue_entries),
    }
}

fn seed_ai_1_seed42() -> RallyScenario {
    seeded_rally(
        "seed_ai_1_seed42",
        42,
        4004,
        25,
        vec![ore_supply_record(1, 4004, 1, 12, 2)],
        vec![(40, 41, 9, compatibility_fixture_source("seed_ai_1"))],
    )
}

fn seed_ai_2_seed0() -> RallyScenario {
    seeded_rally(
        "seed_ai_2_seed0",
        0,
        5005,
        10,
        vec![ore_supply_record(1, 5005, 1, 10, 2)],
        vec![(50, 51, 9, compatibility_fixture_source("seed_ai_2"))],
    )
}

fn seed_ai_3_seed14() -> RallyScenario {
    seeded_rally(
        "seed_ai_3_seed14",
        14,
        6006,
        14,
        vec![ore_supply_record(1, 6006, 1, 20, 2)],
        vec![(60, 61, 9, compatibility_fixture_source("seed_ai_3"))],
    )
}

fn scan_then_mine_seed5() -> RallyScenario {
    seeded_rally(
        "scan_then_mine_seed5",
        5,
        7007,
        40,
        vec![ore_supply_record(1, 7007, 1, 15, 2)],
        vec![(70, 71, 9, compatibility_fixture_source("scan_then_mine"))],
    )
}

fn do_while_mine_seed0() -> RallyScenario {
    seeded_rally(
        "do_while_mine_seed0",
        0,
        8008,
        4,
        vec![ore_supply_record(1, 8008, 1, 10, 2)],
        vec![(80, 81, 9, compatibility_fixture_source("do_while_mine"))],
    )
}

fn triple_queue_seed33() -> RallyScenario {
    seeded_rally(
        "triple_queue_seed33",
        33,
        9009,
        5,
        vec![ore_supply_record(1, 9009, 1, 10, 2)],
        vec![
            (90, 91, 12, "mine();"),
            (91, 92, 15, "mine();"),
            (92, 93, 18, "mine();"),
        ],
    )
}

fn quad_queue_seed33() -> RallyScenario {
    seeded_rally(
        "quad_queue_seed33",
        33,
        9010,
        5,
        vec![ore_supply_record(1, 9010, 1, 12, 2)],
        vec![
            (93, 94, 12, "mine();"),
            (94, 95, 15, "mine();"),
            (95, 96, 18, "mine();"),
            (96, 97, 21, "mine();"),
        ],
    )
}

fn dual_ore_seed11() -> RallyScenario {
    seeded_rally(
        "dual_ore_seed11",
        11,
        10010,
        6,
        vec![
            ore_supply_record(1, 10010, 1, 8, 2),
            ore_supply_record(2, 10010, 2, 6, 1),
        ],
        vec![(100, 101, 9, "mine(); dump(1); mine();")],
    )
}

fn ore_seeker_80x80_seed0() -> RallyScenario {
    let mining_area_id = 11080;
    let mut area = golden_mining_area_record(mining_area_id);
    area.size_x = 80;
    area.size_y = 80;
    area.max_moves = 800;
    let mining_area = MiningAreaLoadout::new(
        area,
        vec![
            ore_supply_record(1, mining_area_id, 1, 15, 12),
            ore_supply_record(2, mining_area_id, 2, 12, 10),
            ore_supply_record(3, mining_area_id, 3, 10, 8),
        ],
        RobotLoadout::new(
            golden_robot_record(1, "rotate(90);"),
            RobotLoadoutParts::empty(),
        ),
    );
    let queue_entries = vec![RallyQueueEntry::new(
        mining_rally_queue_record(110, mining_area_id, 111, 9),
        RobotLoadout::new(
            ore_seeker_robot_record(111, compatibility_fixture_source("ore_seeker_80x80")),
            RobotLoadoutParts::empty(),
        ),
    )];

    RallyScenario {
        name: "ore_seeker_80x80_seed0",
        seed: 0,
        loadout: RallyLoadout::new(mining_area, queue_entries),
    }
}

fn seeded_rally(
    name: &'static str,
    seed: u64,
    mining_area_id: i64,
    max_moves: i32,
    ore_supplies: Vec<MiningAreaOreSupplyRecord>,
    queues: Vec<(i64, i64, i32, &'static str)>,
) -> RallyScenario {
    let mut area = golden_mining_area_record(mining_area_id);
    area.max_moves = max_moves;
    let mining_area = MiningAreaLoadout::new(
        area,
        ore_supplies,
        RobotLoadout::new(
            golden_robot_record(1, "rotate(90);"),
            RobotLoadoutParts::empty(),
        ),
    );
    let queue_entries = queues
        .into_iter()
        .map(|(queue_id, robot_id, seconds_left, source)| {
            RallyQueueEntry::new(
                mining_rally_queue_record(queue_id, mining_area_id, robot_id, seconds_left),
                RobotLoadout::new(
                    golden_robot_record(robot_id, source),
                    RobotLoadoutParts::empty(),
                ),
            )
        })
        .collect();

    RallyScenario {
        name,
        seed,
        loadout: RallyLoadout::new(mining_area, queue_entries),
    }
}
