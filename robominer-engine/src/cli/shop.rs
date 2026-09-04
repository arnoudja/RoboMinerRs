use clap::Subcommand;

#[derive(Debug, Subcommand)]
pub(crate) enum ShopCommand {
    Buy {
        #[arg(long)]
        user_id: i64,

        #[arg(long)]
        robot_part_id: i64,

        /// Confirm intentional shop mutation for a specific user.
        #[arg(long)]
        i_understand: bool,
    },
    Sell {
        #[arg(long)]
        user_id: i64,

        #[arg(long)]
        robot_part_id: i64,

        /// Confirm intentional shop mutation for a specific user.
        #[arg(long)]
        i_understand: bool,
    },
    RobotPartStates {
        #[arg(long)]
        user_id: i64,
    },
    CatalogStates,
}
