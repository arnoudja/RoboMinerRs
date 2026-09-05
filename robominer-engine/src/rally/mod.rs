//! Rally simulation and the background `rally rallies` worker loop.
//!
//! - `run` / `pool` — one-shot or bounded pool simulations (optional persist).
//! - `rallies` — poll claimable mining runs, simulate, optionally persist; with
//!   `--persist` also claims finished runs into wallets between cycles.
//!
//! Prefer `robominer-engine rally …` over calling these modules from other crates.

mod cycle;
mod print;
mod run_single;

pub(crate) use cycle::{RunRalliesOptions, run_rallies, validate_run_rallies_options};
pub(crate) use run_single::{
    RunPoolOptions, RunRallyOptions, run_pool, run_rally, validate_run_pool_options,
};
