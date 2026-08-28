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
    /// When true, cancel only if the full area-cost refund fits without clamping to maxAllowed.
    pub require_refund_fits: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct CanceledMiningQueue {
    pub mining_queue_id: i64,
}

#[derive(Debug, Default, Clone, PartialEq, Eq)]
pub struct CancelMiningQueueBatchResult {
    pub cleared: usize,
    pub skipped: usize,
    pub failed: usize,
    pub last_rejection: Option<CancelMiningQueueRejection>,
    pub rejection_counts: std::collections::HashMap<CancelMiningQueueRejection, usize>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum CancelMiningQueueRejection {
    UnknownQueue,
    WrongOwner,
    NotCancelable,
    /// Refund would exceed wallet maxAllowed (only when `require_refund_fits` is set).
    RefundWouldClamp,
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
    pub session_version: i32,
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
    pub session_version: i32,
    pub password_changed: bool,
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
    pub session_version: i32,
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
pub enum ClaimAchievementStepRejection {
    UnknownUserAchievement,
    NoNextStep,
    RequirementsNotMet,
    MissingDefaultRobotPart,
    InvalidDefaultRobotConfiguration,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PersistRallyRejection {
    /// One or more queue rows were already finished or leased by another worker.
    QueueAlreadyFinished,
}
