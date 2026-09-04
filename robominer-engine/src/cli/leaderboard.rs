use clap::Subcommand;

#[derive(Debug, Subcommand)]
pub(crate) enum LeaderboardCommand {
    States {
        #[arg(long, default_value_t = 10)]
        max_entries: i64,
    },
}
