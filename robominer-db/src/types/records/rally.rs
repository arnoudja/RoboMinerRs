#[derive(Debug, Clone, PartialEq)]
pub struct CompletedRallyRecord {
    pub result_data: String,
    pub participants: Vec<CompletedRallyParticipantRecord>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct CompletedRallyParticipantRecord {
    pub mining_queue_id: i64,
    pub robot_id: i64,
    pub mining_area_id: i64,
    pub player_number: i32,
    pub mining_end_seconds_from_now: i32,
    pub score: f64,
    /// Program source that ran for this queue entry; private to the owner via MiningQueue.
    pub executed_source_code: Option<String>,
    pub ore_results: Vec<CompletedRallyOreRecord>,
    pub action_results: Vec<CompletedRallyActionRecord>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CompletedRallyOreRecord {
    pub ore_id: i64,
    pub amount: i32,
    pub depot_amount: i32,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CompletedRallyActionRecord {
    pub action_type: i32,
    pub amount: i32,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ClaimedOreRewardRecord {
    pub ore_id: i64,
    pub ore_name: String,
    pub reward: i32,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ClaimedUserResults {
    pub claimed_queues: u64,
    pub ore_rewards: Vec<ClaimedOreRewardRecord>,
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct ClaimedMiningQueueCleanupSummary {
    pub queues_deleted: u64,
    pub rally_results_deleted: u64,
}
