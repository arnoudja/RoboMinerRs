use crate::db_outcome::finish_db_outcome;
use crate::output::escape_state_field;
use anyhow::{Context, Result, anyhow};

pub(crate) async fn create_user(
    pool: &robominer_db::MySqlPool,
    request: robominer_db::CreateUserRequest,
) -> Result<()> {
    finish_db_outcome(
        robominer_db::create_user(pool, request)
            .await
            .context("failed to create user")?,
        |created| {
            println!("{}", created.user_id);
            Ok(())
        },
        |rejection| {
            format!(
                "unable to create user: {}",
                robominer_domain::rejection_messages::create_user_rejection_cli_message(rejection)
            )
        },
    )
}

pub(crate) async fn update_user_account(
    pool: &robominer_db::MySqlPool,
    request: robominer_db::UpdateUserAccountRequest,
) -> Result<()> {
    finish_db_outcome(
        robominer_db::update_user_account(pool, request)
            .await
            .context("failed to update user account")?,
        |updated| {
            println!("Updated user account {}", updated.user_id);
            Ok(())
        },
        |rejection| {
            format!(
                "unable to update user account: {}",
                robominer_domain::rejection_messages::update_user_account_rejection_cli_message(
                    rejection
                )
            )
        },
    )
}

pub(crate) async fn account_state(pool: &robominer_db::MySqlPool, user_id: i64) -> Result<()> {
    let user = robominer_db::get_user_by_id(pool, user_id)
        .await
        .context("failed to load account state")?
        .ok_or_else(|| anyhow!("unknown user"))?;

    println!(
        "U\t{}\t{}",
        escape_state_field(&user.username),
        escape_state_field(&user.email)
    );

    Ok(())
}

pub(crate) async fn verify_login(
    pool: &robominer_db::MySqlPool,
    request: robominer_db::VerifyLoginRequest,
) -> Result<()> {
    finish_db_outcome(
        robominer_db::verify_login(pool, request)
            .await
            .context("failed to verify login")?,
        |verified| {
            println!("{}", verified.user_id);
            Ok(())
        },
        |rejection| {
            format!(
                "unable to verify login: {}",
                robominer_domain::rejection_messages::verify_login_rejection_cli_message(rejection)
            )
        },
    )
}

pub(crate) async fn verify_user_password(
    pool: &robominer_db::MySqlPool,
    request: robominer_db::VerifyUserPasswordRequest,
) -> Result<()> {
    finish_db_outcome(
        robominer_db::verify_user_password(pool, request)
            .await
            .context("failed to verify user password")?,
        |verified| {
            println!("{}", verified.user_id);
            Ok(())
        },
        |rejection| {
            format!(
                "unable to verify user password: {}",
                robominer_domain::rejection_messages::verify_login_rejection_cli_message(rejection)
            )
        },
    )
}
