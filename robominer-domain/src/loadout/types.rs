use robominer_db::{
    MiningAreaOreSupplyRecord, MiningAreaRecord, MiningRallyQueueRecord, PoolItemRecord,
    PoolRecord, RobotPartRecord, RobotRecord,
};
use robominer_sim::{Ground, MAX_ORE_TYPES, Position, RobotSpec};

use crate::constants::RALLY_SIZE;
use crate::error::DomainError;

use super::ground::{mining_area_to_ground, robot_record_to_spec};

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct RobotLoadoutParts {
    pub ore_container: Option<RobotPartRecord>,
    pub mining_unit: Option<RobotPartRecord>,
    pub battery: Option<RobotPartRecord>,
    pub memory_module: Option<RobotPartRecord>,
    pub cpu: Option<RobotPartRecord>,
    pub engine: Option<RobotPartRecord>,
    pub ore_scanner: Option<RobotPartRecord>,
}

impl RobotLoadoutParts {
    pub fn empty() -> Self {
        Self {
            ore_container: None,
            mining_unit: None,
            battery: None,
            memory_module: None,
            cpu: None,
            engine: None,
            ore_scanner: None,
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct RobotLoadout {
    pub robot: RobotRecord,
    pub parts: RobotLoadoutParts,
    /// Per area ore-type slot depot capacity for this robot's owner (zeros for AI).
    pub depot_capacity: [i32; MAX_ORE_TYPES],
}

impl RobotLoadout {
    pub fn new(robot: RobotRecord, parts: RobotLoadoutParts) -> Self {
        Self {
            robot,
            parts,
            depot_capacity: [0; MAX_ORE_TYPES],
        }
    }

    pub fn with_depot_capacity(mut self, depot_capacity: [i32; MAX_ORE_TYPES]) -> Self {
        self.depot_capacity = depot_capacity;
        self
    }

    pub fn simulator_spec(&self) -> Result<RobotSpec, DomainError> {
        robot_record_to_spec(&self.robot)
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct MiningAreaLoadout {
    pub area: MiningAreaRecord,
    pub ore_supplies: Vec<MiningAreaOreSupplyRecord>,
    pub ai_robot: RobotLoadout,
}

impl MiningAreaLoadout {
    pub fn new(
        area: MiningAreaRecord,
        ore_supplies: Vec<MiningAreaOreSupplyRecord>,
        ai_robot: RobotLoadout,
    ) -> Self {
        Self {
            area,
            ore_supplies,
            ai_robot,
        }
    }

    pub fn simulator_ground_with_seed(&self, seed: u64) -> Result<Ground, DomainError> {
        mining_area_to_ground(&self.area, &self.ore_supplies, seed)
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct RallyQueueEntry {
    pub queue: MiningRallyQueueRecord,
    pub robot: RobotLoadout,
}

impl RallyQueueEntry {
    pub fn new(queue: MiningRallyQueueRecord, robot: RobotLoadout) -> Self {
        Self { queue, robot }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct RallyLoadout {
    pub mining_area: MiningAreaLoadout,
    pub queue_entries: Vec<RallyQueueEntry>,
}

impl RallyLoadout {
    pub fn new(mining_area: MiningAreaLoadout, queue_entries: Vec<RallyQueueEntry>) -> Self {
        Self {
            mining_area,
            queue_entries,
        }
    }

    pub fn ai_robot_count(&self) -> usize {
        RALLY_SIZE.saturating_sub(self.queue_entries.len())
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct PoolItemLoadout {
    pub item: PoolItemRecord,
    pub robot: RobotLoadout,
}

impl PoolItemLoadout {
    pub fn new(item: PoolItemRecord, robot: RobotLoadout) -> Self {
        Self { item, robot }
    }

    pub fn source_code(&self) -> &str {
        &self.item.source_code
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct PoolLoadout {
    pub pool: PoolRecord,
    pub mining_area: MiningAreaLoadout,
    pub items: Vec<PoolItemLoadout>,
}

impl PoolLoadout {
    pub fn new(
        pool: PoolRecord,
        mining_area: MiningAreaLoadout,
        items: Vec<PoolItemLoadout>,
    ) -> Self {
        Self {
            pool,
            mining_area,
            items,
        }
    }

    pub fn is_complete(&self) -> bool {
        self.items
            .iter()
            .all(|item| item.item.runs_done >= self.pool.required_runs)
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct RallyOutcome {
    pub mining_area_id: i64,
    pub final_time: i32,
    pub participants: Vec<RallyParticipantOutcome>,
}

#[derive(Clone, Debug, PartialEq)]
pub struct RallyRun {
    pub outcome: RallyOutcome,
    pub result_data: String,
}

#[derive(Clone, Debug, PartialEq)]
pub struct RallyParticipantOutcome {
    pub player_number: usize,
    pub queue_id: Option<i64>,
    pub robot_id: i64,
    pub is_ai: bool,
    pub position: Position,
    pub ore: [i32; MAX_ORE_TYPES],
    pub score: f64,
    pub actions_done: [i32; 8],
}

#[derive(Clone, Debug, PartialEq)]
pub struct PoolRallyOutcome {
    pub pool_id: i64,
    pub mining_area_id: i64,
    pub final_time: i32,
    pub items: Vec<PoolItemOutcome>,
}

#[derive(Clone, Debug, PartialEq)]
pub struct PoolItemOutcome {
    pub player_number: usize,
    pub pool_item_id: i64,
    pub robot_id: i64,
    pub score: f64,
    pub ore_results: Vec<PoolItemOreOutcome>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PoolItemOreOutcome {
    pub ore_id: i64,
    pub amount: i32,
}
