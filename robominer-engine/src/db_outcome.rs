use anyhow::{Result, bail};

pub(crate) fn finish_db_outcome<T, R>(
    outcome: robominer_db::DbOutcome<T, R>,
    on_success: impl FnOnce(T) -> Result<()>,
    rejection_message: impl FnOnce(R) -> String,
) -> Result<()> {
    match outcome {
        robominer_db::DbOutcome::Success(value) => on_success(value),
        robominer_db::DbOutcome::Rejected(rejection) => {
            let message = rejection_message(rejection);
            eprintln!("{message}");
            bail!(message)
        }
    }
}
