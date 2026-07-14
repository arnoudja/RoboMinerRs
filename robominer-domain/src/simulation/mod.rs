mod completed;
mod legacy;
mod pool;
mod rally;
mod robots;

#[cfg(test)]
mod tests;

pub use completed::{
    completed_pool_rally_record, completed_rally_record, persist_pool_rally_outcome,
    persist_rally_outcome,
};
pub use pool::run_pool_loadout_with_seed;
pub use rally::{run_rally_loadout_with_animation_seed, run_rally_loadout_with_seed};
