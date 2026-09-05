//! `rally` subcommands: single run, pool, and the long-running `rallies` worker.

use clap::Subcommand;
use std::path::PathBuf;

#[derive(Debug, Subcommand)]
pub(crate) enum RallyCommand {
    Run {
        #[arg(long)]
        mining_area_id: i64,

        #[arg(long, default_value_t = 0)]
        seed: u64,

        #[arg(long)]
        persist: bool,

        #[arg(long)]
        result_data_file: Option<PathBuf>,
    },
    Pool {
        #[arg(long)]
        pool_id: i64,

        #[arg(long, default_value_t = 0)]
        seed: u64,

        #[arg(long)]
        persist: bool,

        #[arg(long)]
        until_complete: bool,

        #[arg(long, default_value_t = 100)]
        max_rallies: u64,
    },
    Rallies {
        #[arg(long)]
        once: bool,

        #[arg(long = "loop")]
        loop_mode: bool,

        /// Maximum seconds between poll cycles; shortened when the next claimable rally is sooner.
        #[arg(long, default_value_t = 5)]
        sleep_seconds: u64,

        #[arg(long, default_value_t = 0)]
        seed: u64,

        #[arg(long)]
        persist: bool,
    },
}
