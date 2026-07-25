mod achievement;
mod activity;
mod assets;
mod cli;
mod database;
mod dispatch;
mod leaderboard;
mod migrate;
mod mining;
mod output;
mod program;
mod rally;
mod robot;
mod shop;
mod user;
mod verify;

use anyhow::Result;
use clap::Parser;

use cli::{Cli, Command};

pub async fn run() -> Result<()> {
    let cli = Cli::parse();

    match &cli.command {
        Command::ClaimResults { .. }
        | Command::EnqueueMining { .. }
        | Command::CancelMiningQueue { .. }
        | Command::MiningQueueStates { .. }
        | Command::MiningQueuePageStates { .. }
        | Command::MiningAreaScores { .. }
        | Command::MiningResultStates { .. }
        | Command::MiningAreaOverviewStates => dispatch::dispatch_mining(cli).await,

        Command::ActivityStates { .. } | Command::RallyViewState { .. } => {
            dispatch::dispatch_activity(cli).await
        }

        Command::BuyRobotPart { .. }
        | Command::SellRobotPart { .. }
        | Command::ShopRobotPartStates { .. }
        | Command::ShopCatalogStates => dispatch::dispatch_shop(cli).await,

        Command::RobotConfigStates { .. } | Command::UpdateRobotConfig { .. } => {
            dispatch::dispatch_robot(cli).await
        }

        Command::Verify { .. }
        | Command::VerifySource { .. }
        | Command::SimulateSource { .. }
        | Command::CreateProgramSource { .. }
        | Command::UpdateProgramSource { .. }
        | Command::DeleteProgramSource { .. }
        | Command::ProgramSourceStates { .. } => dispatch::dispatch_program(cli).await,

        Command::AccountState { .. }
        | Command::CreateUser { .. }
        | Command::UpdateUserAccount { .. }
        | Command::VerifyLogin { .. }
        | Command::VerifyUserPassword { .. } => dispatch::dispatch_user(cli).await,

        Command::ClaimAchievementStep { .. }
        | Command::AchievementStates { .. }
        | Command::AchievementPageStates { .. } => dispatch::dispatch_achievement(cli).await,

        Command::RunRally { .. } | Command::RunPool { .. } | Command::RunRallies { .. } => {
            dispatch::dispatch_rally(cli).await
        }

        Command::Migrate | Command::MigrateStatus { .. } => dispatch::dispatch_migrate(cli).await,

        Command::LeaderboardStates { .. } => dispatch::dispatch_leaderboard(cli).await,

        Command::UserOreAssetStates { .. } => dispatch::dispatch_assets(cli).await,
    }
}
