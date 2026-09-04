use clap::Subcommand;

#[derive(Debug, Subcommand)]
pub(crate) enum UserCommand {
    AccountState {
        #[arg(long)]
        user_id: i64,
    },
    Create {
        #[arg(long)]
        username: String,

        #[arg(long)]
        email: String,

        #[arg(long)]
        password: String,
    },
    UpdateAccount {
        #[arg(long)]
        user_id: i64,

        #[arg(long)]
        username: String,

        #[arg(long)]
        email: String,

        #[arg(long)]
        password: Option<String>,

        /// Required when `--password` is set (resets credentials / invalidates sessions).
        #[arg(long)]
        i_understand: bool,
    },
    VerifyLogin {
        #[arg(long)]
        login_name: String,

        #[arg(long)]
        password: String,
    },
    VerifyPassword {
        #[arg(long)]
        user_id: i64,

        #[arg(long)]
        password: String,
    },
}
