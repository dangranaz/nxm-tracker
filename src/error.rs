use thiserror::Error;

/// Errors produced by the nxm-tracker crate.
#[derive(Debug, Error)]
pub enum TrackerError {
    /// Wraps an [`nxm_config::ConfigError`] from configuration loading/saving.
    #[error("configuration error: {0}")]
    Config(#[from] nxm_config::ConfigError),
    /// Wraps an [`nxm_logging::LoggingError`] from logging initialization.
    #[error("logging error: {0}")]
    Logging(#[from] nxm_logging::LoggingError),
    /// Wraps an [`nxm_db::DbError`] from database access.
    #[error("database error: {0}")]
    Database(#[from] nxm_db::DbError),
    /// The session status string stored in the database is not recognized.
    #[error("invalid session status: {0}")]
    InvalidSessionStatus(String),
    /// The task status string stored in the database is not recognized.
    #[error("invalid task status: {0}")]
    InvalidTaskStatus(String),
    /// A task lifecycle operation was attempted on a task that is not `open`.
    #[error("task {id} is not open (status: {status})")]
    TaskNotOpen { id: i64, status: String },
}
