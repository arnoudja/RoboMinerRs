use clap::{Parser, Subcommand};

mod achievement;
mod activity;
mod assets;
mod leaderboard;
mod migrate;
mod mining;
mod program;
mod rally;
mod robot;
mod shop;
mod user;

pub(crate) use achievement::AchievementCommand;
pub(crate) use activity::ActivityCommand;
pub(crate) use assets::AssetsCommand;
pub(crate) use leaderboard::LeaderboardCommand;
pub(crate) use migrate::MigrateCommand;
pub(crate) use mining::MiningCommand;
pub(crate) use program::ProgramCommand;
pub(crate) use rally::RallyCommand;
pub(crate) use robot::RobotCommand;
pub(crate) use shop::ShopCommand;
pub(crate) use user::UserCommand;

#[derive(Debug, Parser)]
#[command(name = "robominer-engine")]
#[command(about = "RoboMiner engine CLI and rally worker")]
pub(crate) struct Cli {
    #[arg(long)]
    pub(crate) database_url: Option<String>,

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
