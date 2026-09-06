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

        /// Prefer env `ROBOMINER_USER_PASSWORD` or a TTY prompt over argv.
        #[arg(long, env = "ROBOMINER_USER_PASSWORD")]
        password: Option<String>,
    },
    UpdateAccount {
        #[arg(long)]
        user_id: i64,

        #[arg(long)]
        username: String,

        #[arg(long)]
        email: String,

        /// Prefer env `ROBOMINER_USER_PASSWORD` or a TTY prompt over argv.
        #[arg(long, env = "ROBOMINER_USER_PASSWORD")]
        password: Option<String>,

        /// Required when `--password` is set (resets credentials / invalidates sessions).
        #[arg(long)]
        i_understand: bool,
    },
    VerifyLogin {
        #[arg(long)]
        login_name: String,

        /// Prefer env `ROBOMINER_USER_PASSWORD` or a TTY prompt over argv.
        #[arg(long, env = "ROBOMINER_USER_PASSWORD")]
        password: Option<String>,
    },
    VerifyPassword {
        #[arg(long)]
        user_id: i64,

        /// Prefer env `ROBOMINER_USER_PASSWORD` or a TTY prompt over argv.
        #[arg(long, env = "ROBOMINER_USER_PASSWORD")]
        password: Option<String>,
    },
}
