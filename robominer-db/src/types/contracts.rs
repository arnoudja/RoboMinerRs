#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct EnqueueMiningRequest {
    pub user_id: i64,
    pub robot_id: i64,
    pub mining_area_id: i64,
    pub fill: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct EnqueuedMining {
    pub inserted_queues: u64,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EnqueueMiningRejection {
    UnknownRobot,
    UnknownMiningArea,
    MiningAreaUnavailable,
    QueueFull,
    InsufficientFunds,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct CancelMiningQueueRequest {
    pub user_id: i64,
    pub mining_queue_id: i64,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct CanceledMiningQueue {
    pub mining_queue_id: i64,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CancelMiningQueueRejection {
    UnknownQueue,
    WrongOwner,
    NotCancelable,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct RobotPartTransactionRequest {
    pub user_id: i64,
    pub robot_part_id: i64,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct RobotPartTransaction {
    pub robot_part_id: i64,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SellAllUnassignedRobotPartsResult {
    pub sold_count: i32,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RobotPartTransactionRejection {
    UnknownUser,
    UnknownRobotPart,
    InsufficientFunds,
    NoUnassignedRobotPart,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct UpdateRobotConfigRequest {
    pub user_id: i64,
    pub robot_id: i64,
    pub robot_name: String,
    pub program_source_id: i64,
    pub ore_container_id: i64,
    pub mining_unit_id: i64,
    pub battery_id: i64,
    pub memory_module_id: i64,
    pub cpu_id: i64,
    pub engine_id: i64,
    pub ore_scanner_id: i64,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct UpdatedRobotConfig {
    pub robot_id: i64,
    pub pending: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum UpdateRobotConfigRejection {
    UnknownRobot,
    ChangeAlreadyPending,
    InvalidRobotName,
    UnknownProgramSource,
    UnknownRobotPart,
    ProgramTooLarge,
    NoUnassignedRobotPart,
    InvalidRobotPartConfiguration,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ProgramSourceWriteRequest {
    pub user_id: i64,
    pub program_source_id: i64,
    pub source_name: String,
    pub source_code: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CreateProgramSourceRequest {
    pub user_id: i64,
    pub source_name: String,
    pub source_code: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct CreatedProgramSource {
    pub program_source_id: i64,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AppliedProgramSource {
    pub applied_robots: u64,
    pub warnings: Vec<ProgramSourceApplyWarning>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ProgramSourceApplyWarning {
    pub robot_name: String,
    pub reason: ProgramSourceApplyWarningReason,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ProgramSourceApplyWarningReason {
    NotEnoughMemory,
    RobotBusy,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ProgramSourceWriteRejection {
    UnknownUser,
    UnknownProgramSource,
    SourceInUse,
    EmptySourceName,
    EmptySourceCode,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CreateUserRequest {
    pub username: String,
    pub email: String,
    pub password: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct CreatedUser {
    pub user_id: i64,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CreateUserRejection {
    InvalidUsername,
    InvalidEmail,
    InvalidPassword,
    DuplicateUsername,
    DuplicateEmail,
    InitialAchievementRejected(ClaimAchievementStepRejection),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct UpdateUserAccountRequest {
    pub user_id: i64,
    pub username: String,
    pub email: String,
    pub password: Option<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct UpdatedUserAccount {
    pub user_id: i64,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum UpdateUserAccountRejection {
    UnknownUser,
    InvalidUsername,
    InvalidEmail,
    InvalidPassword,
    DuplicateUsername,
    DuplicateEmail,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct VerifyLoginRequest {
    pub login_name: String,
    pub password: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct VerifiedLogin {
    pub user_id: i64,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum VerifyLoginRejection {
    UnknownUser,
    InvalidPassword,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct VerifyUserPasswordRequest {
    pub user_id: i64,
    pub password: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ClaimAchievementStepRequest {
    pub user_id: i64,
    pub achievement_id: i64,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ClaimedAchievementStep {
    pub achievement_id: i64,
    pub step: i32,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct AchievementClaimStateRecord {
    pub achievement_id: i64,
    pub claimable: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct UserOreMinedRecord {
    pub ore_id: i64,
    pub amount: i32,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct UserMiningAreaScoreRecord {
    pub mining_area_id: i64,
    pub score: f64,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct AchievementPagePointsSummaryRecord {
    pub points_earned: i64,
    pub points_achievable: i64,
}

#[derive(Debug, Clone, PartialEq)]
pub struct AchievementPageStateRecord {
    pub achievement_id: i64,
    pub title: String,
    pub description: String,
    pub steps_claimed: i32,
    pub number_of_steps: i64,
    pub achievement_points_earned: i64,
    pub total_achievement_points: i64,
    pub step: i32,
    pub next_achievement_points: i32,
    pub mining_queue_reward: i32,
    pub robot_reward: i32,
    pub ore_id: Option<i64>,
    pub ore_name: Option<String>,
    pub current_ore_maximum: i32,
    pub max_ore_reward: i32,
    pub mining_area_id: Option<i64>,
    pub mining_area_name: Option<String>,
    pub claimable: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AchievementPageTotalRequirementRecord {
    pub achievement_id: i64,
    pub ore_id: i64,
    pub ore_name: String,
    pub amount: i32,
    pub current_amount: i32,
}

#[derive(Debug, Clone, PartialEq)]
pub struct AchievementPageScoreRequirementRecord {
    pub achievement_id: i64,
    pub mining_area_id: i64,
    pub area_name: String,
    pub minimum_score: f64,
    pub current_score: f64,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LeaderboardMiningAreaRecord {
    pub id: i64,
    pub area_name: String,
}

#[derive(Debug, Clone, PartialEq)]
pub struct LeaderboardMiningAreaScoreRecord {
    pub mining_area_id: i64,
    pub robot_name: String,
    pub username: String,
    pub score: f64,
    pub total_runs: i32,
}

#[derive(Debug, Clone, PartialEq)]
pub struct LeaderboardTopRobotRecord {
    pub robot_name: String,
    pub username: String,
    pub ore_per_run: f64,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LeaderboardTopUserRecord {
    pub username: String,
    pub achievement_points: i32,
}

#[derive(Debug, Clone, PartialEq)]
pub struct LeaderboardViewerStandingRecord {
    pub achievement_points: i32,
    pub achievement_rank: i64,
    pub area_standings: Vec<LeaderboardViewerAreaStandingRecord>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct LeaderboardViewerAreaStandingRecord {
    pub mining_area_id: i64,
    pub area_name: String,
    pub robot_name: String,
    pub score: f64,
    pub rank: i64,
}

#[derive(Debug, Clone, PartialEq)]
pub struct MiningResultStateRecord {
    pub robot_id: i64,
    pub mining_queue_id: i64,
    pub mining_area_name: String,
    pub rally_result_id: Option<i64>,
    pub score: f64,
    pub total_ore_mined: i32,
    pub total_tax: i32,
    pub total_reward: i32,
    pub creation_time_millis: i64,
    pub mining_end_time_millis: i64,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MiningResultOreStateRecord {
    pub mining_queue_id: i64,
    pub ore_id: i64,
    pub ore_name: String,
    pub amount: i32,
    pub tax: i32,
    pub reward: i32,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct MiningResultActionStateRecord {
    pub mining_queue_id: i64,
    pub action_type: i32,
    pub amount: i32,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MiningAreaOverviewOreRecord {
    pub ore_id: i64,
    pub ore_name: String,
}

#[derive(Debug, Clone, PartialEq)]
pub struct MiningAreaOverviewAreaRecord {
    pub mining_area_id: i64,
    pub area_name: String,
    pub total_percentage: f64,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct MiningAreaOverviewPercentageRecord {
    pub mining_area_id: i64,
    pub ore_id: i64,
    pub percentage: f64,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ClaimAchievementStepRejection {
    UnknownUserAchievement,
    NoNextStep,
    RequirementsNotMet,
    MissingDefaultRobotPart,
    InvalidDefaultRobotConfiguration,
}
