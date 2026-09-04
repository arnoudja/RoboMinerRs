#![cfg_attr(test, allow(clippy::unwrap_used, clippy::expect_used))]
//! CLI and background worker for RoboMiner (rally polling, shop/user/robot ops,
//! migrate). Shares domain façades with `robominer-web`. See `CONTRIBUTING.md`.

mod achievement;
mod activity;
mod assets;
mod cli;
mod database;
mod db_outcome;
mod dispatch;
mod leaderboard;
mod migrate;
mod mining;
mod output;
mod program;
mod rally;
mod robot;
mod shop;
mod shutdown;
mod user;
mod verify;

use anyhow::Result;
use clap::Parser;

use cli::{Cli, Command};

pub async fn run() -> Result<()> {
    let cli = Cli::parse();

    let Cli {
        database_url,
        config,
        command,
    } = cli;

    match command {
        Command::Mining(cmd) => dispatch::dispatch_mining(database_url, config, cmd).await,
        Command::Activity(cmd) => dispatch::dispatch_activity(database_url, config, cmd).await,
        Command::Shop(cmd) => dispatch::dispatch_shop(database_url, config, cmd).await,
        Command::Robot(cmd) => dispatch::dispatch_robot(database_url, config, cmd).await,
        Command::Program(cmd) => dispatch::dispatch_program(database_url, config, cmd).await,
        Command::User(cmd) => dispatch::dispatch_user(database_url, config, cmd).await,
        Command::Achievement(cmd) => {
            dispatch::dispatch_achievement(database_url, config, cmd).await
        }
        Command::Rally(cmd) => dispatch::dispatch_rally(database_url, config, cmd).await,
        Command::Migrate(cmd) => dispatch::dispatch_migrate(database_url, config, cmd).await,
        Command::Leaderboard(cmd) => {
            dispatch::dispatch_leaderboard(database_url, config, cmd).await
        }
        Command::Assets(cmd) => dispatch::dispatch_assets(database_url, config, cmd).await,
    }
}
