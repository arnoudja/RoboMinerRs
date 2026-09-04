use clap::Subcommand;

#[derive(Debug, Subcommand)]
pub(crate) enum AchievementCommand {
    ClaimStep {
        #[arg(long)]
        user_id: i64,

        #[arg(long)]
        achievement_id: i64,

        /// Confirm intentional achievement claim for a specific user.
        #[arg(long)]
        i_understand: bool,
    },
    States {
        #[arg(long)]
        user_id: i64,
    },
    PageStates {
        #[arg(long)]
        user_id: i64,
    },
}
