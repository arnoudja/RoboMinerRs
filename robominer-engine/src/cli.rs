use clap::{Parser, Subcommand};
use std::path::PathBuf;

#[derive(Debug, Parser)]
#[command(name = "robominer-engine")]
#[command(about = "Rust migration engine for RoboMiner")]
pub(crate) struct Cli {
    #[arg(long)]
    pub(crate) database_url: Option<String>,

    #[arg(long)]
    pub(crate) config: Option<PathBuf>,

    #[command(subcommand)]
    pub(crate) command: Command,
}

#[derive(Debug, Subcommand)]
pub(crate) enum Command {
    Verify {
        program_source_id: i64,
    },
    VerifySource {
        source_file: PathBuf,
    },
    SimulateSource {
        source_file: Option<PathBuf>,

        #[arg(long)]
        robot: Vec<PathBuf>,

        #[arg(long, default_value_t = 10)]
        turns: i32,

        #[arg(long, default_value_t = 5)]
        size_x: usize,

        #[arg(long, default_value_t = 5)]
        size_y: usize,

        #[arg(long, default_value_t = 1)]
        ore_x: usize,

        #[arg(long, default_value_t = 1)]
        ore_y: usize,

        #[arg(long, default_value_t = 0)]
        ore_type: usize,

        #[arg(long, default_value_t = 8)]
        ore_amount: i32,

        #[arg(long, default_value_t = 4)]
        mining_speed: i32,

        #[arg(long, default_value_t = 1.5)]
        forward_speed: f64,

        #[arg(long, default_value_t = 1.0)]
        backward_speed: f64,

        #[arg(long, default_value_t = 90)]
        rotate_speed: i32,
    },
    ClaimResults {
        #[arg(long)]
        user_id: i64,
    },
    EnqueueMining {
        #[arg(long)]
        user_id: i64,

        #[arg(long)]
        robot_id: i64,

        #[arg(long)]
        mining_area_id: i64,

        #[arg(long)]
        fill: bool,
    },
    CancelMiningQueue {
        #[arg(long)]
        user_id: i64,

        #[arg(long)]
        mining_queue_id: i64,
    },
    MiningQueueStates {
        #[arg(long)]
        user_id: i64,
    },
    MiningQueuePageStates {
        #[arg(long)]
        user_id: i64,
    },
    ActivityStates {
        #[arg(long)]
        user_id: i64,

        #[arg(long, default_value_t = 5)]
        max_users: i64,

        #[arg(long, default_value_t = 10)]
        max_rallies: i64,
    },
    RallyViewState {
        #[arg(long)]
        user_id: i64,

        #[arg(long)]
        rally_result_id: i64,

        #[arg(long)]
        require_user_result: bool,
    },
    UserOreAssetStates {
        #[arg(long)]
        user_id: i64,
    },
    MiningAreaScores {
        #[arg(long)]
        user_id: i64,
    },
    MiningResultStates {
        #[arg(long)]
        user_id: i64,

        #[arg(long, default_value_t = 10)]
        max_results: i64,
    },
    MiningAreaOverviewStates,
    BuyRobotPart {
        #[arg(long)]
        user_id: i64,

        #[arg(long)]
        robot_part_id: i64,
    },
    SellRobotPart {
        #[arg(long)]
        user_id: i64,

        #[arg(long)]
        robot_part_id: i64,
    },
    ShopRobotPartStates {
        #[arg(long)]
        user_id: i64,
    },
    ShopCatalogStates,
    RobotConfigStates {
        #[arg(long)]
        user_id: i64,
    },
    UpdateRobotConfig {
        #[arg(long)]
        user_id: i64,

        #[arg(long)]
        robot_id: i64,

        #[arg(long)]
        robot_name: String,

        #[arg(long)]
        program_source_id: i64,

        #[arg(long)]
        ore_container_id: i64,

        #[arg(long)]
        mining_unit_id: i64,

        #[arg(long)]
        battery_id: i64,

        #[arg(long)]
        memory_module_id: i64,

        #[arg(long)]
        cpu_id: i64,

        #[arg(long)]
        engine_id: i64,

        #[arg(long)]
        ore_scanner_id: i64,
    },
    CreateProgramSource {
        #[arg(long)]
        user_id: i64,

        #[arg(long)]
        source_name: String,

        #[arg(long)]
        source_code: String,
    },
    UpdateProgramSource {
        #[arg(long)]
        user_id: i64,

        #[arg(long)]
        program_source_id: i64,

        #[arg(long)]
        source_name: String,

        #[arg(long)]
        source_code: String,
    },
    DeleteProgramSource {
        #[arg(long)]
        user_id: i64,

        #[arg(long)]
        program_source_id: i64,
    },
    ProgramSourceStates {
        #[arg(long)]
        user_id: i64,
    },
    AccountState {
        #[arg(long)]
        user_id: i64,
    },
    CreateUser {
        #[arg(long)]
        username: String,

        #[arg(long)]
        email: String,

        #[arg(long)]
        password: String,
    },
    UpdateUserAccount {
        #[arg(long)]
        user_id: i64,

        #[arg(long)]
        username: String,

        #[arg(long)]
        email: String,

        #[arg(long)]
        password: Option<String>,
    },
    VerifyLogin {
        #[arg(long)]
        login_name: String,

        #[arg(long)]
        password: String,
    },
    VerifyUserPassword {
        #[arg(long)]
        user_id: i64,

        #[arg(long)]
        password: String,
    },
    ClaimAchievementStep {
        #[arg(long)]
        user_id: i64,

        #[arg(long)]
        achievement_id: i64,
    },
    AchievementStates {
        #[arg(long)]
        user_id: i64,
    },
    AchievementPageStates {
        #[arg(long)]
        user_id: i64,
    },
    LeaderboardStates {
        #[arg(long, default_value_t = 10)]
        max_entries: i64,
    },
    RunRally {
        #[arg(long)]
        mining_area_id: i64,

        #[arg(long, default_value_t = 0)]
        seed: u64,

        #[arg(long)]
        persist: bool,

        #[arg(long)]
        result_data_file: Option<PathBuf>,
    },
    RunPool {
        #[arg(long)]
        pool_id: i64,

        #[arg(long, default_value_t = 0)]
        seed: u64,

        #[arg(long)]
        persist: bool,

        #[arg(long)]
        until_complete: bool,

        #[arg(long, default_value_t = 100)]
        max_rallies: u64,
    },
    RunRallies {
        #[arg(long)]
        once: bool,

        #[arg(long = "loop")]
        loop_mode: bool,

        #[arg(long, default_value_t = 5)]
        sleep_seconds: u64,

        #[arg(long, default_value_t = 0)]
        seed: u64,

        #[arg(long)]
        persist: bool,
    },
    /// Apply pending schema migrations (or baseline a current schema).
    Migrate,
    /// Show applied/pending schema migrations.
    MigrateStatus,
}
