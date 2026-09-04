use clap::Subcommand;

#[derive(Debug, Subcommand)]
pub(crate) enum AssetsCommand {
    OreStates {
        #[arg(long)]
        user_id: i64,
    },
}
