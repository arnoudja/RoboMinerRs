//! Domain layer: loadouts, simulation, compile-linked program writes, and shared
//! rejection copy. Persistence and typed mutation contracts live in `robominer-db`.
//! See `CONTRIBUTING.md` for the crate boundary.

mod rejection_messages;

mod constants;
mod error;
mod loadout;
mod robot_config;
mod simulation;

pub use error::{DomainError, RobotPartSlot};
pub use loadout::*;
pub use rejection_messages::*;
pub use robot_config::*;
pub use simulation::*;
