mod ground;
mod legacy_ore;
mod load;
mod types;

#[cfg(test)]
mod tests;

pub(crate) use ground::validate_ore_supply;
#[cfg(test)]
pub(crate) use ground::{mining_area_to_ground, robot_record_to_spec};
pub(crate) use legacy_ore::{legacy_ore_slot, sorted_legacy_ore_supplies};
pub use load::{
    load_mining_area_loadout, load_next_pool_rally_loadout, load_next_rally_loadout,
    load_next_rally_loadout_with_claim, load_pool_loadout, load_robot_loadout,
    mining_rally_queue_is_ready,
};
pub use types::{
    MiningAreaLoadout, PoolItemLoadout, PoolLoadout, RallyLoadout, RallyQueueEntry, RobotLoadout,
    RobotLoadoutParts,
};
