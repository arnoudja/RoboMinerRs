//! Domain layer: loadouts, simulation, compile-linked program writes, and shared
//! rejection copy. Persistence and typed mutation contracts live in `robominer-db`.
//! See `CONTRIBUTING.md` for the crate boundary.

mod constants;
mod error;
pub mod loadout;
pub mod rejection_messages;
pub mod robot_config;
pub mod simulation;

pub use error::{DomainError, RobotPartSlot};

pub use loadout::{
    load_mining_area_loadout, load_next_pool_rally_loadout, load_next_rally_loadout,
    load_pool_loadout, load_robot_loadout, mining_rally_queue_is_ready,
};

pub use robot_config::{create_program_source, update_program_source};

pub use simulation::{
    persist_pool_rally_outcome, persist_rally_outcome, run_pool_loadout_with_seed,
    run_rally_loadout_with_animation_seed, run_rally_loadout_with_seed,
};
