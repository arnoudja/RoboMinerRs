use robominer_sim::{MAX_ORE_TYPES, Position};

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
    pub depot: [i32; MAX_ORE_TYPES],
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
