use clap::Subcommand;

#[derive(Debug, Subcommand)]
pub(crate) enum ActivityCommand {
    States {
        #[arg(long)]
        user_id: i64,

        #[arg(long, default_value_t = 5)]
        max_users: i64,

        #[arg(long, default_value_t = 10)]
        max_rallies: i64,
    },
    RallyViewState {
        #[arg(long)]
        user_id: i64,

        #[arg(long)]
        rally_result_id: i64,

        #[arg(long)]
        require_user_result: bool,
    },
}
