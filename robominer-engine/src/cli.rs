use clap::{Parser, Subcommand};
use std::path::PathBuf;

#[derive(Debug, Parser)]
#[command(name = "robominer-engine")]
#[command(about = "RoboMiner engine CLI and rally worker")]
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
    /// Mining queue and read model commands.
    #[command(subcommand)]
    Mining(MiningCommand),
    /// Activity feed read model commands.
    #[command(subcommand)]
    Activity(ActivityCommand),
    /// Shop catalog and purchase commands.
    #[command(subcommand)]
    Shop(ShopCommand),
    /// Robot configuration commands.
    #[command(subcommand)]
    Robot(RobotCommand),
    /// Program source and verification commands.
    #[command(subcommand)]
    Program(ProgramCommand),
    /// User account commands.
    #[command(subcommand)]
    User(UserCommand),
    /// Achievement progress commands.
    #[command(subcommand)]
    Achievement(AchievementCommand),
    /// Rally simulation commands.
    #[command(subcommand)]
    Rally(RallyCommand),
    /// Schema migration commands.
    #[command(subcommand)]
    Migrate(MigrateCommand),
    /// Leaderboard read model commands.
    #[command(subcommand)]
    Leaderboard(LeaderboardCommand),
    /// User asset read model commands.
    #[command(subcommand)]
    Assets(AssetsCommand),
}

#[derive(Debug, Subcommand)]
pub(crate) enum MiningCommand {
    ClaimResults {
        #[arg(long)]
        user_id: i64,

        /// Confirm intentional wallet claim for a specific user.
        #[arg(long)]
        i_understand: bool,
    },
    ClaimAll {
        #[arg(long)]
        once: bool,

        #[arg(long)]
        loop_mode: bool,

        #[arg(long, default_value_t = 5)]
        sleep_seconds: u64,
    },
    Enqueue {
        #[arg(long)]
        user_id: i64,

        #[arg(long)]
        robot_id: i64,

        #[arg(long)]
        mining_area_id: i64,

        #[arg(long)]
        fill: bool,
    },
    CancelQueue {
        #[arg(long)]
        user_id: i64,

        #[arg(long)]
        mining_queue_id: i64,
    },
    QueueStates {
        #[arg(long)]
        user_id: i64,
    },
    QueuePageStates {
        #[arg(long)]
        user_id: i64,
    },
    AreaScores {
        #[arg(long)]
        user_id: i64,
    },
    ResultStates {
        #[arg(long)]
        user_id: i64,

        #[arg(long, default_value_t = 10)]
        max_results: i64,
    },
    AreaOverviewStates,
}

#[derive(Debug, Subcommand)]
pub(crate) enum ActivityCommand {
    States {
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
}

#[derive(Debug, Subcommand)]
pub(crate) enum ShopCommand {
    Buy {
        #[arg(long)]
        user_id: i64,

        #[arg(long)]
        robot_part_id: i64,

        /// Confirm intentional shop mutation for a specific user.
        #[arg(long)]
        i_understand: bool,
    },
    Sell {
        #[arg(long)]
        user_id: i64,

        #[arg(long)]
        robot_part_id: i64,

        /// Confirm intentional shop mutation for a specific user.
        #[arg(long)]
        i_understand: bool,
    },
    RobotPartStates {
        #[arg(long)]
        user_id: i64,
    },
    CatalogStates,
}

#[derive(Debug, Subcommand)]
pub(crate) enum RobotCommand {
    ConfigStates {
        #[arg(long)]
        user_id: i64,
    },
    UpdateConfig {
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
}

#[derive(Debug, Subcommand)]
pub(crate) enum ProgramCommand {
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
    CreateSource {
        #[arg(long)]
        user_id: i64,

        #[arg(long)]
        source_name: String,

        #[arg(long)]
        source_code: String,
    },
    UpdateSource {
        #[arg(long)]
        user_id: i64,

        #[arg(long)]
        program_source_id: i64,

        #[arg(long)]
        source_name: String,

        #[arg(long)]
        source_code: String,
    },
    DeleteSource {
        #[arg(long)]
        user_id: i64,

        #[arg(long)]
        program_source_id: i64,
    },
    SourceStates {
        #[arg(long)]
        user_id: i64,
    },
}

#[derive(Debug, Subcommand)]
pub(crate) enum UserCommand {
    AccountState {
        #[arg(long)]
        user_id: i64,
    },
    Create {
        #[arg(long)]
        username: String,

        #[arg(long)]
        email: String,

        #[arg(long)]
        password: String,
    },
    UpdateAccount {
        #[arg(long)]
        user_id: i64,

        #[arg(long)]
        username: String,

        #[arg(long)]
        email: String,

        #[arg(long)]
        password: Option<String>,

        /// Required when `--password` is set (resets credentials / invalidates sessions).
        #[arg(long)]
        i_understand: bool,
    },
    VerifyLogin {
        #[arg(long)]
        login_name: String,

        #[arg(long)]
        password: String,
    },
    VerifyPassword {
        #[arg(long)]
        user_id: i64,

        #[arg(long)]
        password: String,
    },
}

#[derive(Debug, Subcommand)]
pub(crate) enum AchievementCommand {
    ClaimStep {
        #[arg(long)]
        user_id: i64,

        #[arg(long)]
        achievement_id: i64,

        /// Confirm intentional achievement claim for a specific user.
        #[arg(long)]
        i_understand: bool,
    },
    States {
        #[arg(long)]
        user_id: i64,
    },
    PageStates {
        #[arg(long)]
        user_id: i64,
    },
}

#[derive(Debug, Subcommand)]
pub(crate) enum RallyCommand {
    Run {
        #[arg(long)]
        mining_area_id: i64,

        #[arg(long, default_value_t = 0)]
        seed: u64,

        #[arg(long)]
        persist: bool,

        #[arg(long)]
        result_data_file: Option<PathBuf>,
    },
    Pool {
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
    Rallies {
        #[arg(long)]
        once: bool,

        #[arg(long = "loop")]
        loop_mode: bool,

        /// Maximum seconds between poll cycles; shortened when the next claimable rally is sooner.
        #[arg(long, default_value_t = 5)]
        sleep_seconds: u64,

        #[arg(long, default_value_t = 0)]
        seed: u64,

        #[arg(long)]
        persist: bool,
    },
}

#[derive(Debug, Subcommand)]
pub(crate) enum MigrateCommand {
    /// Apply pending schema migrations (or baseline a current schema).
    Apply,
    /// Show applied/pending schema migrations.
    Status {
        /// Exit non-zero when any embedded migration is still pending.
        #[arg(long)]
        check: bool,
    },
}

#[derive(Debug, Subcommand)]
pub(crate) enum LeaderboardCommand {
    States {
        #[arg(long, default_value_t = 10)]
        max_entries: i64,
    },
}

#[derive(Debug, Subcommand)]
pub(crate) enum AssetsCommand {
    OreStates {
        #[arg(long)]
        user_id: i64,
    },
}
