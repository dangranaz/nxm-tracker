//! Typed error for handoff writes.
//!
//! Defines `HandoffWriteError`, the error returned by handoff write
//! operations (`update_session`). Never a panic, always a typed error
//! naming the cause.

use crate::repo::RepoError;
use nxm_db::DbError;
use thiserror::Error;

/// Error of the handoff write repository.
///
/// Covers: session not found, underlying database errors,
/// generic write/transaction failures.
#[derive(Debug, Error)]
pub enum HandoffWriteError {
    /// The requested session does not exist.
    #[error("session not found: {0}")]
    SessionNotFound(String),

    /// Error of I/O / configuration of the underlying database.
    #[error("database error: {0}")]
    Db(#[from] DbError),

    /// Error from the read repository (e.g. session not found on read).
    #[error("repository error: {0}")]
    Repo(#[from] RepoError),

    /// Generic write error (e.g. transaction failure).
    #[error("write error: {0}")]
    Write(String),
}
