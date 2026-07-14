use crate::error::DomainError;
use crate::loadout::{
    MiningAreaLoadout, PoolItemLoadout, PoolItemOreOutcome, PoolItemOutcome, PoolLoadout,
    PoolRallyOutcome, RobotLoadout, RobotLoadoutParts,
};
use crate::simulation::{completed_pool_rally_record, run_pool_loadout_with_seed};
use robominer_test_support::{
    ore_supply_record, pool_item_record, pool_record, robot_record, unit_test_mining_area_record,
    unit_test_robot_record,
};
use robominer_db::CompletedPoolItemOreRecord;

#[test]
fn pool_loadout_carries_pool_area_and_item_robot_data() {
    let mining_area = MiningAreaLoadout::new(
        unit_test_mining_area_record(1001),
        vec![ore_supply_record(1, 1001, 1, 10, 1)],
        RobotLoadout::new(robot_record(1), RobotLoadoutParts::empty()),
    );
    let item = PoolItemLoadout::new(
        pool_item_record(50, 900, 11, "mine();", 42.5, 1),
        RobotLoadout::new(robot_record(11), RobotLoadoutParts::empty()),
    );
    let loadout = PoolLoadout::new(pool_record(900, 1001, 3), mining_area, vec![item]);

    assert_eq!(loadout.pool.required_runs, 3);
    assert_eq!(loadout.mining_area.area.id, 1001);
    assert_eq!(loadout.items[0].item.id, 50);
    assert_eq!(loadout.items[0].source_code(), "mine();");
    assert_eq!(
        loadout.items[0].robot.simulator_spec().unwrap().robot_id,
        11
    );
}

#[test]
fn pool_loadout_reports_completion_from_item_runs() {
    let mining_area = MiningAreaLoadout::new(
        unit_test_mining_area_record(1001),
        vec![ore_supply_record(1, 1001, 1, 10, 1)],
        RobotLoadout::new(robot_record(1), RobotLoadoutParts::empty()),
    );
    let complete = PoolLoadout::new(
        pool_record(900, 1001, 2),
        mining_area.clone(),
        vec![
            PoolItemLoadout::new(
                pool_item_record(50, 900, 11, "mine();", 42.5, 2),
                RobotLoadout::new(robot_record(11), RobotLoadoutParts::empty()),
            ),
            PoolItemLoadout::new(
                pool_item_record(51, 900, 12, "move();", 40.0, 3),
                RobotLoadout::new(robot_record(12), RobotLoadoutParts::empty()),
            ),
        ],
    );
    let incomplete = PoolLoadout::new(
        pool_record(900, 1001, 2),
        mining_area,
        vec![PoolItemLoadout::new(
            pool_item_record(52, 900, 13, "rotate(90);", 30.0, 1),
            RobotLoadout::new(robot_record(13), RobotLoadoutParts::empty()),
        )],
    );

    assert!(complete.is_complete());
    assert!(!incomplete.is_complete());
}

#[test]
fn run_pool_loadout_uses_pool_item_sources_and_returns_item_outcomes() {
    let mut area = unit_test_mining_area_record(1001);
    area.max_moves = 3;
    let mining_area = MiningAreaLoadout::new(
        area,
        vec![ore_supply_record(1, 1001, 7, 10, 2)],
        RobotLoadout::new(
            unit_test_robot_record(1, "rotate(90);"),
            RobotLoadoutParts::empty(),
        ),
    );
    let loadout = PoolLoadout::new(
        pool_record(900, 1001, 3),
        mining_area,
        vec![PoolItemLoadout::new(
            pool_item_record(50, 900, 11, "mine();", 0.0, 0),
            RobotLoadout::new(
                unit_test_robot_record(11, "mine("),
                RobotLoadoutParts::empty(),
            ),
        )],
    );

    let outcome = run_pool_loadout_with_seed(&loadout, 0).expect("pool rally should run");

    assert_eq!(outcome.pool_id, 900);
    assert_eq!(outcome.mining_area_id, 1001);
    assert_eq!(outcome.final_time, 3);
    assert_eq!(outcome.items.len(), 1);
    assert_eq!(outcome.items[0].player_number, 0);
    assert_eq!(outcome.items[0].pool_item_id, 50);
    assert_eq!(outcome.items[0].robot_id, 11);
    assert!(outcome.items[0].score > 0.0);
    assert_eq!(outcome.items[0].ore_results[0].ore_id, 7);
    assert!(outcome.items[0].ore_results[0].amount > 0);
}

#[test]
fn run_pool_loadout_rejects_empty_or_oversized_item_sets() {
    let mining_area = MiningAreaLoadout::new(
        unit_test_mining_area_record(1001),
        vec![ore_supply_record(1, 1001, 1, 10, 1)],
        RobotLoadout::new(robot_record(1), RobotLoadoutParts::empty()),
    );
    let empty = PoolLoadout::new(pool_record(900, 1001, 3), mining_area.clone(), Vec::new());
    let too_many = PoolLoadout::new(
        pool_record(900, 1001, 3),
        mining_area,
        (0..5)
            .map(|index| {
                PoolItemLoadout::new(
                    pool_item_record(50 + index, 900, 11 + index, "move();", 0.0, 0),
                    RobotLoadout::new(robot_record(11 + index), RobotLoadoutParts::empty()),
                )
            })
            .collect(),
    );

    assert!(matches!(
        run_pool_loadout_with_seed(&empty, 0).unwrap_err(),
        DomainError::InvalidPoolLoadout { .. }
    ));
    assert!(matches!(
        run_pool_loadout_with_seed(&too_many, 0).unwrap_err(),
        DomainError::InvalidPoolLoadout { .. }
    ));
}

#[test]
fn run_pool_loadout_reports_pool_item_compile_errors() {
    let mining_area = MiningAreaLoadout::new(
        unit_test_mining_area_record(1001),
        vec![ore_supply_record(1, 1001, 1, 10, 1)],
        RobotLoadout::new(robot_record(1), RobotLoadoutParts::empty()),
    );
    let loadout = PoolLoadout::new(
        pool_record(900, 1001, 3),
        mining_area,
        vec![PoolItemLoadout::new(
            pool_item_record(50, 900, 11, "mine(", 0.0, 0),
            RobotLoadout::new(robot_record(11), RobotLoadoutParts::empty()),
        )],
    );

    let error = run_pool_loadout_with_seed(&loadout, 0).unwrap_err();

    assert!(matches!(
        error,
        DomainError::ProgramCompile { robot_id: 11, .. }
    ));
}

#[test]
fn completed_pool_rally_record_maps_outcome_to_write_rows() {
    let mining_area = MiningAreaLoadout::new(
        unit_test_mining_area_record(1001),
        vec![ore_supply_record(1, 1001, 7, 10, 1)],
        RobotLoadout::new(robot_record(1), RobotLoadoutParts::empty()),
    );
    let loadout = PoolLoadout::new(
        pool_record(900, 1001, 3),
        mining_area,
        vec![PoolItemLoadout::new(
            pool_item_record(50, 900, 11, "mine();", 0.0, 0),
            RobotLoadout::new(robot_record(11), RobotLoadoutParts::empty()),
        )],
    );
    let outcome = PoolRallyOutcome {
        pool_id: 900,
        mining_area_id: 1001,
        final_time: 3,
        items: vec![PoolItemOutcome {
            player_number: 0,
            pool_item_id: 50,
            robot_id: 11,
            score: 12.5,
            ore_results: vec![
                PoolItemOreOutcome {
                    ore_id: 7,
                    amount: 4,
                },
                PoolItemOreOutcome {
                    ore_id: 8,
                    amount: 0,
                },
            ],
        }],
    };

    let record = completed_pool_rally_record(&loadout, &outcome).expect("record should map");

    assert_eq!(record.items.len(), 1);
    assert_eq!(record.items[0].pool_item_id, 50);
    assert_eq!(record.items[0].score, 12.5);
    assert_eq!(
        record.items[0].ore_results,
        vec![CompletedPoolItemOreRecord {
            ore_id: 7,
            amount: 4,
        }]
    );
}

#[test]
fn completed_pool_rally_record_rejects_mismatched_outcomes() {
    let mining_area = MiningAreaLoadout::new(
        unit_test_mining_area_record(1001),
        vec![ore_supply_record(1, 1001, 7, 10, 1)],
        RobotLoadout::new(robot_record(1), RobotLoadoutParts::empty()),
    );
    let loadout = PoolLoadout::new(
        pool_record(900, 1001, 3),
        mining_area,
        vec![PoolItemLoadout::new(
            pool_item_record(50, 900, 11, "mine();", 0.0, 0),
            RobotLoadout::new(robot_record(11), RobotLoadoutParts::empty()),
        )],
    );
    let outcome = PoolRallyOutcome {
        pool_id: 901,
        mining_area_id: 1001,
        final_time: 3,
        items: vec![PoolItemOutcome {
            player_number: 0,
            pool_item_id: 50,
            robot_id: 11,
            score: 12.5,
            ore_results: Vec::new(),
        }],
    };

    let error = completed_pool_rally_record(&loadout, &outcome).unwrap_err();

    assert!(matches!(error, DomainError::PoolOutcomeMismatch { .. }));
}
