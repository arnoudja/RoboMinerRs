mod ground;
mod load;
mod types;

#[cfg(test)]
mod tests;

pub(crate) use ground::validate_ore_supply;
pub use ground::{mining_area_to_ground, robot_record_to_spec};
pub use load::{
    load_mining_area_loadout, load_next_pool_rally_loadout, load_next_rally_loadout,
    load_pool_loadout, load_robot_loadout, mining_rally_queue_is_ready,
};
pub use types::{
    MiningAreaLoadout, PoolItemLoadout, PoolItemOreOutcome, PoolItemOutcome, PoolLoadout,
    PoolRallyOutcome, RallyLoadout, RallyOutcome, RallyParticipantOutcome, RallyQueueEntry,
    RallyRun, RobotLoadout, RobotLoadoutParts,
};
