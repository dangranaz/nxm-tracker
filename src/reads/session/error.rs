//! Errori tipizzati per le letture di sessione.

use crate::repo::RepoError;
use nxm_db::DbError;
use thiserror::Error;

/// Errore tipizzato per le letture di sessione.
#[derive(Debug, Error)]
pub enum SessionReadError {
    /// La sessione richiesta non esiste.
    #[error("session not found: {0}")]
    SessionNotFound(String),

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
