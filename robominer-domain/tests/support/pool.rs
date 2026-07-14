use robominer_domain::{MiningAreaLoadout, PoolItemLoadout, PoolLoadout, RobotLoadout, RobotLoadoutParts};
use robominer_test_support::{
    golden_mining_area_record, golden_robot_record, ore_supply_record, pool_item_record,
    pool_record,
};

pub struct PoolScenario {
    pub name: &'static str,
    pub seed: u64,
    pub loadout: PoolLoadout,
}

pub fn scenario(name: &str) -> PoolScenario {
    match name {
        "single_miner_pool_seed0" => single_miner_pool_seed0(),
        "dual_item_pool_seed17" => dual_item_pool_seed17(),
        other => panic!("unknown pool golden scenario: {other}"),
    }
}

fn single_miner_pool_seed0() -> PoolScenario {
    let pool_id = 2001;
    let mining_area_id = 12001;
    let mut area = golden_mining_area_record(mining_area_id);
    area.max_moves = 3;
    let mining_area = MiningAreaLoadout::new(
        area,
        vec![ore_supply_record(1, mining_area_id, 7, 10, 2)],
        RobotLoadout::new(
            golden_robot_record(1, "rotate(90);"),
            RobotLoadoutParts::empty(),
        ),
    );
    let loadout = PoolLoadout::new(
        pool_record(pool_id, mining_area_id, 3),
        mining_area,
        vec![PoolItemLoadout::new(
            pool_item_record(50, pool_id, 11, "mine();", 0.0, 0),
            RobotLoadout::new(
                golden_robot_record(11, "mine();"),
                RobotLoadoutParts::empty(),
            ),
        )],
    );

    PoolScenario {
        name: "single_miner_pool_seed0",
        seed: 0,
        loadout,
    }
}

fn dual_item_pool_seed17() -> PoolScenario {
    let pool_id = 2002;
    let mining_area_id = 12002;
    let mut area = golden_mining_area_record(mining_area_id);
    area.max_moves = 5;
    let mining_area = MiningAreaLoadout::new(
        area,
        vec![ore_supply_record(1, mining_area_id, 1, 10, 2)],
        RobotLoadout::new(
            golden_robot_record(1, "rotate(90);"),
            RobotLoadoutParts::empty(),
        ),
    );
    let loadout = PoolLoadout::new(
        pool_record(pool_id, mining_area_id, 3),
        mining_area,
        vec![
            PoolItemLoadout::new(
                pool_item_record(60, pool_id, 21, "mine();", 0.0, 0),
                RobotLoadout::new(
                    golden_robot_record(21, "mine();"),
                    RobotLoadoutParts::empty(),
                ),
            ),
            PoolItemLoadout::new(
                pool_item_record(61, pool_id, 22, "mine(); dump(1); mine();", 0.0, 0),
                RobotLoadout::new(
                    golden_robot_record(22, "mine(); dump(1); mine();"),
                    RobotLoadoutParts::empty(),
                ),
            ),
        ],
    );

    PoolScenario {
        name: "dual_item_pool_seed17",
        seed: 17,
        loadout,
    }
}
