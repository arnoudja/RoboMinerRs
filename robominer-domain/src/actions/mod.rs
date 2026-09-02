//! Thin mutation wrappers shared by web and engine presentation layers.

mod account;
mod achievements;
mod mining_queue;
mod robot_config;
mod shop;

pub use account::{
    LogoutAllDevicesOutcome, UpdateUserAccountOutcome, logout_all_devices, update_user_account,
};
pub use achievements::{ClaimAchievementStepOutcome, claim_achievement_step};
pub use mining_queue::{
    CancelMiningQueueOutcome, EnqueueMiningOutcome, cancel_mining_queue, enqueue_mining,
};
pub use robot_config::{UpdateRobotConfigOutcome, update_robot_config};
pub use shop::{
    BuyRobotPartOutcome, SellAllUnassignedRobotPartsOutcome, SellRobotPartOutcome, buy_robot_part,
    sell_all_unassigned_robot_parts, sell_robot_part,
};
