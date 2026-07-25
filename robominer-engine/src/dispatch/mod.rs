use anyhow::{Result, ensure};

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

pub(crate) use achievement::dispatch_achievement;
pub(crate) use activity::dispatch_activity;
pub(crate) use assets::dispatch_assets;
pub(crate) use leaderboard::dispatch_leaderboard;
pub(crate) use migrate::dispatch_migrate;
pub(crate) use mining::dispatch_mining;
pub(crate) use program::dispatch_program;
pub(crate) use rally::dispatch_rally;
pub(crate) use robot::dispatch_robot;
pub(crate) use shop::dispatch_shop;
pub(crate) use user::dispatch_user;

fn ensure_positive_user_id(user_id: i64) -> Result<()> {
    ensure!(user_id > 0, "--user-id must be greater than zero");
    Ok(())
}
