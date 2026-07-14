use robominer_db::{
    MiningAreaOreSupplyRecord, MiningAreaRecord, MiningRallyQueueRecord, PoolItemRecord,
    PoolRecord, RobotPartRecord, RobotRecord,
};

#[derive(Clone, Copy, Debug)]
pub struct RobotStats {
    pub recharge_time: i32,
    pub max_ore: i32,
    pub mining_speed: i32,
    pub max_turns: i32,
    pub memory_size: i32,
    pub cpu_speed: i32,
    pub forward_speed: f64,
    pub backward_speed: f64,
    pub rotate_speed: i32,
    pub robot_size: f64,
    pub scan_time: i32,
    pub scan_distance: i32,
    pub total_mining_runs: i32,
}

impl RobotStats {
    pub const fn golden_default() -> Self {
        Self {
            recharge_time: 20,
            max_ore: 75,
            mining_speed: 9,
            max_turns: 600,
            memory_size: 128,
            cpu_speed: 3,
            forward_speed: 1.25,
            backward_speed: 0.75,
            rotate_speed: 30,
            robot_size: 1.5,
            scan_time: 6,
            scan_distance: 5,
            total_mining_runs: 4,
        }
    }

    pub const fn unit_test_default() -> Self {
        Self::golden_default()
    }

    pub const fn ore_seeker() -> Self {
        Self {
            recharge_time: 20,
            max_ore: 300,
            mining_speed: 3,
            max_turns: 800,
            memory_size: 128,
            cpu_speed: 15,
            forward_speed: 7.0,
            backward_speed: 0.75,
            rotate_speed: 28,
            robot_size: 1.5,
            scan_time: 8,
            scan_distance: 13,
            total_mining_runs: 4,
        }
    }
}

pub fn robot_record_with_stats(
    id: i64,
    robot_name: &str,
    source_code: &str,
    stats: RobotStats,
) -> RobotRecord {
    RobotRecord {
        id,
        user_id: 3,
        robot_name: robot_name.to_string(),
        source_code: source_code.to_string(),
        program_source_id: Some(11),
        ore_container_id: Some(101),
        mining_unit_id: Some(201),
        battery_id: Some(301),
        memory_module_id: Some(401),
        cpu_id: Some(501),
        engine_id: Some(601),
        ore_scanner_id: Some(701),
        recharge_time: stats.recharge_time,
        max_ore: stats.max_ore,
        mining_speed: stats.mining_speed,
        max_turns: stats.max_turns,
        memory_size: stats.memory_size,
        cpu_speed: stats.cpu_speed,
        forward_speed: stats.forward_speed,
        backward_speed: stats.backward_speed,
        rotate_speed: stats.rotate_speed,
        robot_size: stats.robot_size,
        scan_time: stats.scan_time,
        scan_distance: stats.scan_distance,
        total_mining_runs: stats.total_mining_runs,
    }
}

pub fn golden_robot_record(id: i64, source_code: &str) -> RobotRecord {
    robot_record_with_stats(
        id,
        &format!("Robot {id}"),
        source_code,
        RobotStats::golden_default(),
    )
}

pub fn unit_test_robot_record(id: i64, source_code: &str) -> RobotRecord {
    robot_record_with_stats(
        id,
        "Test robot",
        source_code,
        RobotStats::unit_test_default(),
    )
}

pub fn ore_seeker_robot_record(id: i64, source_code: &str) -> RobotRecord {
    robot_record_with_stats(
        id,
        &format!("Robot {id}"),
        source_code,
        RobotStats::ore_seeker(),
    )
}

pub fn robot_record(id: i64) -> RobotRecord {
    unit_test_robot_record(id, "move();")
}

pub fn mining_area_record(id: i64, area_name: &str) -> MiningAreaRecord {
    MiningAreaRecord {
        id,
        area_name: area_name.to_string(),
        ore_price_id: 10001,
        size_x: 4,
        size_y: 4,
        max_moves: 150,
        mining_time: 30,
        tax_rate: 25,
        ai_robot_id: 1,
    }
}

pub fn golden_mining_area_record(id: i64) -> MiningAreaRecord {
    mining_area_record(id, &format!("Area {id}"))
}

pub fn unit_test_mining_area_record(id: i64) -> MiningAreaRecord {
    mining_area_record(id, "Cerbonium-1")
}

pub fn ore_supply_record(
    id: i64,
    mining_area_id: i64,
    ore_id: i64,
    supply: i32,
    radius: i32,
) -> MiningAreaOreSupplyRecord {
    MiningAreaOreSupplyRecord {
        id,
        mining_area_id,
        ore_id,
        supply,
        radius,
    }
}

pub fn mining_rally_queue_record(
    id: i64,
    mining_area_id: i64,
    robot_id: i64,
    seconds_left: i32,
) -> MiningRallyQueueRecord {
    MiningRallyQueueRecord {
        queue: robominer_db::MiningQueueRecord {
            id,
            mining_area_id,
            robot_id,
            rally_result_id: None,
            player_number: None,
            score: None,
            claimed: false,
        },
        user_id: robot_id,
        seconds_left,
    }
}

pub fn pool_record(id: i64, mining_area_id: i64, required_runs: i32) -> PoolRecord {
    PoolRecord {
        id,
        mining_area_id,
        required_runs,
    }
}

pub fn pool_item_record(
    id: i64,
    pool_id: i64,
    robot_id: i64,
    source_code: &str,
    total_score: f64,
    runs_done: i32,
) -> PoolItemRecord {
    PoolItemRecord {
        id,
        pool_id,
        robot_id,
        source_code: source_code.to_string(),
        total_score,
        runs_done,
    }
}

pub fn robot_part(id: i64, type_id: i64) -> RobotPartRecord {
    RobotPartRecord {
        id,
        type_id,
        tier_id: Some(1),
        part_name: format!("Part {id}"),
        ore_price_id: 12,
        ore_capacity: 1,
        mining_capacity: 2,
        battery_capacity: 3,
        memory_capacity: 4,
        cpu_capacity: 5,
        forward_capacity: 6,
        backward_capacity: 7,
        rotate_capacity: 8,
        recharge_time: 9,
        scan_time: 0,
        scan_distance: 0,
        weight: 10,
        volume: 11,
        power_usage: 12,
    }
}
