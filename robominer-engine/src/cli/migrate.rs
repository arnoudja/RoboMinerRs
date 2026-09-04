use clap::Subcommand;

#[derive(Debug, Subcommand)]
pub(crate) enum MigrateCommand {
    /// Apply pending schema migrations (or baseline a current schema).
    Apply,
    /// Show applied/pending schema migrations.
    Status {
        /// Exit non-zero when any embedded migration is still pending.
        #[arg(long)]
        check: bool,
    },
}
