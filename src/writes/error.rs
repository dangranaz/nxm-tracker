//! Errore tipizzato per le scritture del lifecycle.
//!
//! Definisce `WriteError`, l'errore restituito dalle operazioni di scrittura
//! (es. `create_session`). Mai un panic, sempre un errore tipizzato che
//! nomina la causa.

use crate::domain::transition::Trigger;
use crate::domain::{Status, Transition};
use crate::repo::RepoError;
use nxm_db::DbError;
use thiserror::Error;

/// Errore del repository di scrittura del lifecycle.
///
/// Copre: id vuoto, id duplicato, errori di database sottostanti,
/// sessione inesistente, transizione non permessa, transizione terminale
/// che richiede signoff, campo obbligatorio vuoto.
#[derive(Debug, Error)]
pub enum WriteError {
    /// Lo slug è vuoto (o solo spazi) o non rispetta il formato kebab-case.
    #[error("invalid slug: {slug}")]
    InvalidSlug { slug: String },

    /// L'id della sessione è già presente nel database.
    #[error("session id already exists: {0}")]
    DuplicateId(String),

    /// La sessione richiesta non esiste.
    #[error("session not found: {0}")]
    SessionNotFound(String),

    /// La transizione richiesta non è nel known-set del dominio.
    #[error("transition not allowed: {from} -> {to} via {trigger}")]
    TransitionNotAllowed {
        from: Status,
        to: Status,
        trigger: Trigger,
    },

    /// La transizione verso uno stato terminale è bloccata perché la policy
    /// `require_human_signoff` è attiva e non esiste una firma umana valida
    /// per la sessione.
    #[error("terminal state requires human signoff: {to}")]
    SignoffRequired { to: Status },

    /// Un campo obbligatorio è vuoto (o solo spazi).
    #[error("field must not be empty: {field}")]
    EmptyField { field: &'static str },

    /// Errore di I/O o configurazione del database sottostante.
    #[error("database error: {0}")]
    Db(#[from] DbError),

    /// Errore del repository di lettura (es. sessione non trovata su read).
    #[error("repository error: {0}")]
    Repo(#[from] RepoError),

    /// Errore generico di scrittura (es. fallimento di transazione).
    #[error("write error: {0}")]
    Write(String),
}
