use anyhow::{Result, ensure};

use super::{ensure_destructive_confirmed, ensure_positive_user_id};
use crate::cli::UserCommand;
use crate::database::connect_database;
use crate::user::{
    account_state, create_user, update_user_account, verify_login, verify_user_password,
};

pub(crate) async fn dispatch_user(
    database_url: Option<String>,
    command: UserCommand,
) -> Result<()> {
    match command {
        UserCommand::AccountState { user_id } => {
            ensure_positive_user_id(user_id)?;
            let pool = connect_database(database_url).await?;
            account_state(&pool, user_id).await
        }
        UserCommand::Create {
            username,
            email,
            password,
        } => {
            ensure!(!username.is_empty(), "--username must not be empty");
            ensure!(!email.is_empty(), "--email must not be empty");
            ensure!(!password.is_empty(), "--password must not be empty");
            let pool = connect_database(database_url).await?;
            create_user(
                &pool,
                robominer_db::CreateUserRequest {
                    username,
                    email,
                    password,
                },
            )
            .await
        }
        UserCommand::UpdateAccount {
            user_id,
            username,
            email,
            password,
            i_understand,
        } => {
            ensure_positive_user_id(user_id)?;
            ensure!(!username.is_empty(), "--username must not be empty");
            ensure!(!email.is_empty(), "--email must not be empty");
            if let Some(password) = &password {
                ensure!(!password.is_empty(), "--password must not be empty");
                ensure_destructive_confirmed(i_understand, "user update-account --password")?;
            }
            let pool = connect_database(database_url).await?;
            update_user_account(
                &pool,
                robominer_db::UpdateUserAccountRequest {
                    user_id,
                    username,
                    email,
                    password,
                },
            )
            .await
        }
        UserCommand::VerifyLogin {
            login_name,
            password,
        } => {
            ensure!(!login_name.is_empty(), "--login-name must not be empty");
            ensure!(!password.is_empty(), "--password must not be empty");
            let pool = connect_database(database_url).await?;
            verify_login(
                &pool,
                robominer_db::VerifyLoginRequest {
                    login_name,
                    password,
                },
            )
            .await
        }
        UserCommand::VerifyPassword { user_id, password } => {
            ensure_positive_user_id(user_id)?;
            ensure!(!password.is_empty(), "--password must not be empty");
            let pool = connect_database(database_url).await?;
            verify_user_password(
                &pool,
                robominer_db::VerifyUserPasswordRequest { user_id, password },
            )
            .await
        }
    }
}
