#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ProgramSourceVerification {
    pub verified: bool,
    pub compiled_size: i32,
    pub error_description: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct UserRecord {
    pub id: i64,
    pub username: String,
    pub email: String,
    pub password_hash: String,
    pub achievement_points: i32,
    pub mining_queue_size: i32,
    pub session_version: i32,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ProgramSourceRecord {
    pub id: i64,
    pub user_id: i64,
    pub source_name: String,
    pub source_code: Option<String>,
    pub verified: bool,
    pub compiled_size: i32,
    pub error_description: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ProgramSourceStateRecord {
    pub source: ProgramSourceRecord,
    pub linked_robot_count: i64,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RobotPartTypeRecord {
    pub id: i64,
    pub type_name: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RobotPartRecord {
    pub id: i64,
    pub type_id: i64,
    pub tier_id: Option<i64>,
    pub part_name: String,
    pub ore_price_id: i64,
    pub ore_capacity: i32,
    pub mining_capacity: i32,
    pub battery_capacity: i32,
    pub memory_capacity: i32,
    pub cpu_capacity: i32,
    pub forward_capacity: i32,
    pub backward_capacity: i32,
    pub rotate_capacity: i32,
    pub recharge_time: i32,
    pub scan_time: i32,
    pub scan_distance: i32,
    pub weight: i32,
    pub volume: i32,
    pub power_usage: i32,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct UserOreAssetStateRecord {
    pub ore_id: i64,
    pub ore_name: String,
    pub amount: i32,
    pub max_allowed: i32,
    pub depot_max_allowed: i32,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct UserAssetSummaryRecord {
    pub username: String,
    pub achievement_points: i32,
    pub mining_queue_size: i32,
    pub robot_count: i64,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct OreRecord {
    pub id: i64,
    pub ore_name: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ShopRobotPartCatalogRecord {
    pub robot_part_id: i64,
    pub type_id: i64,
    pub tier_id: i64,
    pub tier_name: String,
    pub part_name: String,
    pub ore_capacity: i32,
    pub mining_capacity: i32,
    pub battery_capacity: i32,
    pub memory_capacity: i32,
    pub cpu_capacity: i32,
    pub forward_capacity: i32,
    pub backward_capacity: i32,
    pub rotate_capacity: i32,
    pub recharge_time: i32,
    pub scan_time: i32,
    pub scan_distance: i32,
    pub weight: i32,
    pub volume: i32,
    pub power_usage: i32,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ShopRobotPartCostRecord {
    pub robot_part_id: i64,
    pub ore_id: i64,
    pub ore_name: String,
    pub amount: i32,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ShopRobotPartStateRecord {
    pub robot_part_id: i64,
    pub total_owned: i32,
    pub assigned: i32,
    pub unassigned: i32,
    pub can_buy: bool,
    pub can_sell: bool,
}

#[derive(Debug, Clone, PartialEq)]
pub struct RobotConfigStateRecord {
    pub robot_id: i64,
    pub robot_name: String,
    pub program_source_id: i64,
    pub ore_container_id: i64,
    pub ore_container_name: String,
    pub mining_unit_id: i64,
    pub mining_unit_name: String,
    pub battery_id: i64,
    pub battery_name: String,
    pub memory_module_id: i64,
    pub memory_module_name: String,
    pub cpu_id: i64,
    pub cpu_name: String,
    pub engine_id: i64,
    pub engine_name: String,
    pub ore_scanner_id: i64,
    pub ore_scanner_name: String,
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
    pub change_pending: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RobotConfigPartAssetStateRecord {
    pub type_id: i64,
    pub robot_part_id: i64,
    pub part_name: String,
    pub memory_capacity: i32,
    pub unassigned: i32,
}

#[derive(Debug, Clone, PartialEq)]
pub struct RobotRecord {
    pub id: i64,
    pub user_id: i64,
    pub robot_name: String,
    pub source_code: String,
    pub program_source_id: Option<i64>,
    pub ore_container_id: Option<i64>,
    pub mining_unit_id: Option<i64>,
    pub battery_id: Option<i64>,
    pub memory_module_id: Option<i64>,
    pub cpu_id: Option<i64>,
    pub engine_id: Option<i64>,
    pub ore_scanner_id: Option<i64>,
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

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MiningAreaRecord {
    pub id: i64,
    pub area_name: String,
    pub ore_price_id: i64,
    pub size_x: i32,
    pub size_y: i32,
    pub max_moves: i32,
    pub mining_time: i32,
    pub tax_rate: i32,
    pub ai_robot_id: i64,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MiningAreaOreSupplyRecord {
    pub id: i64,
    pub mining_area_id: i64,
    pub ore_id: i64,
    pub supply: i32,
    pub radius: i32,
}

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

#[derive(Debug, Clone, PartialEq)]
pub struct RobotMiningAreaScoreRecord {
    pub robot_id: i64,
    pub mining_area_id: i64,
    pub score: f64,
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
    pub mining_time: i32,
    pub max_moves: i32,
    pub size_x: i32,
    pub size_y: i32,
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

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ActivityRecentUserRecord {
    pub user_id: i64,
    pub username: String,
    pub last_login_time_millis: i64,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ActivityRecentRallyRecord {
    pub mining_queue_id: i64,
    pub rally_result_id: Option<i64>,
    pub mining_area_id: i64,
    pub mining_area_name: String,
    pub robot_name: String,
    pub username: String,
    pub mining_end_time_millis: i64,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ActivityRallyAreaOption {
    pub mining_area_id: i64,
    pub area_name: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ActivityRecentRallyParticipantRecord {
    pub mining_queue_id: i64,
    pub player_number: i32,
    pub robot_name: String,
    pub username: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RallyViewStateRecord {
    pub result_data: String,
    pub ai_robot_name: String,
    pub ai_username: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RallyViewParticipantRecord {
    pub player_number: i32,
    pub robot_name: String,
    pub username: String,
}

#[derive(Debug, Clone, PartialEq)]
pub struct RallyViewMetadataRecord {
    pub mining_area_id: i64,
    pub mining_area_name: String,
    pub viewer_player_number: Option<i32>,
    pub viewer_robot_id: Option<i64>,
    pub viewer_robot_name: Option<String>,
    pub viewer_score: Option<f64>,
    pub viewer_total_ore_mined: Option<i32>,
    pub viewer_total_tax: Option<i32>,
    pub viewer_total_reward: Option<i32>,
    pub viewer_result_claimed: bool,
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

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PoolRecord {
    pub id: i64,
    pub mining_area_id: i64,
    pub required_runs: i32,
}

#[derive(Debug, Clone, PartialEq)]
pub struct PoolItemRecord {
    pub id: i64,
    pub pool_id: i64,
    pub robot_id: i64,
    pub source_code: String,
    pub total_score: f64,
    pub runs_done: i32,
}

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
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CompletedRallyActionRecord {
    pub action_type: i32,
    pub amount: i32,
}

#[derive(Debug, Clone, PartialEq)]
pub struct CompletedPoolRallyRecord {
    pub items: Vec<CompletedPoolItemRecord>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct CompletedPoolItemRecord {
    pub pool_item_id: i64,
    pub score: f64,
    pub ore_results: Vec<CompletedPoolItemOreRecord>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CompletedPoolItemOreRecord {
    pub ore_id: i64,
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

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AppShellHudRecord {
    pub ore_assets: Vec<UserOreAssetStateRecord>,
    pub queue_used: i64,
    pub queue_capacity: i64,
    pub claimable_achievements_count: u64,
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct ClaimedMiningQueueCleanupSummary {
    pub queues_deleted: u64,
    pub rally_results_deleted: u64,
}
