use clap::Subcommand;

#[derive(Debug, Subcommand)]
pub(crate) enum MiningCommand {
    ClaimResults {
        #[arg(long)]
        user_id: i64,

        /// Confirm intentional wallet claim for a specific user.
        #[arg(long)]
        i_understand: bool,
    },
    ClaimAll {
        #[arg(long)]
        once: bool,

        #[arg(long)]
        loop_mode: bool,

        #[arg(long, default_value_t = 5)]
        sleep_seconds: u64,
    },
    Enqueue {
        #[arg(long)]
        user_id: i64,

        #[arg(long)]
        robot_id: i64,

        #[arg(long)]
        mining_area_id: i64,

        #[arg(long)]
        fill: bool,
    },
    CancelQueue {
        #[arg(long)]
        user_id: i64,

        #[arg(long)]
        mining_queue_id: i64,
    },
    QueueStates {
        #[arg(long)]
        user_id: i64,
    },
    QueuePageStates {
        #[arg(long)]
        user_id: i64,
    },
    AreaScores {
        #[arg(long)]
        user_id: i64,
    },
    ResultStates {
        #[arg(long)]
        user_id: i64,

        #[arg(long, default_value_t = 10)]
        max_results: i64,
    },
    AreaOverviewStates,
}
