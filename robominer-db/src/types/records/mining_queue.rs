#[derive(Debug, Clone, PartialEq)]
pub struct MiningQueueRecord {
    pub id: i64,
    pub mining_area_id: i64,
    pub robot_id: i64,
    pub rally_result_id: Option<i64>,
    pub player_number: Option<i32>,
    pub score: Option<f64>,
    pub claimed: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MiningQueueStateRecord {
    pub mining_queue_id: i64,
    pub robot_id: i64,
    pub status: MiningQueueStatus,
    pub time_left_seconds: i64,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MiningQueuePageRobotRecord {
    pub robot_id: i64,
    pub robot_name: String,
    pub recharge_time: i32,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MiningQueuePageAreaRecord {
    pub mining_area_id: i64,
    pub area_name: String,
    pub tax_rate: i32,
    pub depot_tax_rate: i32,
    pub mining_time: i32,
    pub max_moves: i32,
    pub size_x: i32,
    pub size_y: i32,
    pub score_ore_target: i32,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MiningQueuePageAreaCostRecord {
    pub mining_area_id: i64,
    pub ore_id: i64,
    pub ore_name: String,
    pub amount: i32,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MiningQueuePageAreaSupplyRecord {
    pub mining_area_id: i64,
    pub ore_id: i64,
    pub ore_name: String,
    pub supply: i32,
    pub radius: i32,
}

#[derive(Debug, Clone, PartialEq)]
pub struct MiningQueuePageAreaYieldRecord {
    pub mining_area_id: i64,
    pub ore_id: i64,
    pub ore_name: String,
    pub percentage: f64,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MiningQueuePageItemRecord {
    pub mining_queue_id: i64,
    pub robot_id: i64,
    pub mining_area_id: i64,
    pub area_name: String,
    pub rally_result_id: Option<i64>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MiningQueueStatus {
    Mining,
    Recharging,
    Queued,
    Updating,
}

impl MiningQueueStatus {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Mining => "MINING",
            Self::Recharging => "RECHARGING",
            Self::Queued => "QUEUED",
            Self::Updating => "UPDATING",
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct MiningRallyQueueRecord {
    pub queue: MiningQueueRecord,
    pub user_id: i64,
    pub seconds_left: i32,
}

/// Queue-head candidate for predicting when a rally becomes claimable.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NextClaimRallyCandidate {
    pub mining_area_id: i64,
    pub user_id: i64,
    /// Seconds until the robot (and any processing lease) is free for claim.
    pub busy_seconds: i32,
    /// Engagement countdown used by claim readiness (`seconds_left`).
    pub seconds_left: i32,
}
