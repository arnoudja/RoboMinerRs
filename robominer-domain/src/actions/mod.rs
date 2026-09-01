//! Thin mutation wrappers shared by web and engine presentation layers.

mod mining_queue;
mod shop;

pub use mining_queue::{
    CancelMiningQueueOutcome, EnqueueMiningOutcome, cancel_mining_queue, enqueue_mining,
};
pub use shop::{
    BuyRobotPartOutcome, SellAllUnassignedRobotPartsOutcome, SellRobotPartOutcome, buy_robot_part,
    sell_all_unassigned_robot_parts, sell_robot_part,
};
