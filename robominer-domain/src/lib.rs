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

pub use rejection_messages::{
    cancel_mining_queue_rejection_cli_message, cancel_mining_queue_rejection_player_message,
    claim_achievement_step_rejection_message, create_user_rejection_cli_message,
    create_user_rejection_player_message, enqueue_mining_rejection_cli_message,
    enqueue_mining_rejection_player_message, format_program_source_apply_player_message,
    program_source_apply_warning_message, program_source_write_rejection_cli_message,
    program_source_write_rejection_player_message, robot_part_transaction_rejection_message,
    update_robot_config_rejection_cli_message, update_robot_config_rejection_player_message,
    update_user_account_rejection_cli_message, update_user_account_rejection_player_message,
    verify_login_rejection_cli_message,
};

pub use robot_config::{create_program_source, update_program_source};

pub use simulation::{
    persist_pool_rally_outcome, persist_rally_outcome, run_pool_loadout_with_seed,
    run_rally_loadout_with_animation_seed, run_rally_loadout_with_seed,
};
