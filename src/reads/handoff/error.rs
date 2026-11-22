//! Errori tipizzati per le letture di handoff.

use crate::repo::RepoError;
use nxm_db::DbError;
use thiserror::Error;

/// Errore tipizzato per le letture di handoff.
#[derive(Debug, Error)]
pub enum HandoffReadError {
    /// La sessione richiesta non esiste.
    #[error("session not found: {0}")]
    SessionNotFound(String),

    /// Nessuna handoff trovata per la sessione.
    #[error("no handoff found for session: {0}")]
    NoHandoff(String),

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
