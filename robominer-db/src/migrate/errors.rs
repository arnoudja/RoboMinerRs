use thiserror::Error;

#[derive(Debug, Error)]
pub enum MigrateError {
    #[error(transparent)]
    Database(#[from] sqlx::Error),
    #[error("{0}")]
    InvalidMigration(String),
    #[error(transparent)]
    Io(#[from] std::io::Error),
}
