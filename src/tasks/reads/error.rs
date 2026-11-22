//! Errori tipizzati per le letture di task.

use crate::repo::RepoError;
use nxm_db::DbError;
use thiserror::Error;

/// Errore tipizzato per le letture di task.
#[derive(Debug, Error)]
pub enum TaskReadError {
    /// Il task richiesto non esiste.
    #[error("task not found: {0}:{1}")]
    TaskNotFound(String, i64),

    /// Errore di database sottostante.
    #[error("database error: {0}")]
    Db(#[from] DbError),

    /// Errore del repository di lettura.
    #[error("repository error: {0}")]
    Repo(#[from] RepoError),

    /// Errore generico di lettura.
    #[error("read error: {0}")]
    Read(String),
}
