//! Trait `LifecycleRepo` (sola lettura) e i suoi adapter.
//!
//! Definisce il seam del lifecycle come trait di sola lettura
//! (`states`, `allowed_transitions`) con due implementazioni:
//! un adapter SQLite che legge dalle tabelle del database e un fake
//! in-memory per i test dei caller. Nessuna scrittura, nessuna
//! validazione referenziale.

use crate::domain::{ParseStatusError, ParseTriggerError, Status, Transition};
use nxm_db::DbError;
use thiserror::Error;

/// Errore tipizzato del repository di lettura del lifecycle.
///
/// Un valore DB non mappabile (es. uno `status` o un `trigger` sconosciuto)
/// produce un errore tipizzato, mai un panic.
#[derive(Debug, Error)]
pub enum RepoError {
    /// Errore di I/O / configurazione del database sottostante.
    #[error("database error: {0}")]
    Db(#[from] DbError),

    /// Il campo `name` di una riga di `lifecycle_state` non si mappa a uno
    /// `Status` noto.
    #[error("failed to parse status '{0}': {1}")]
    ParseStatus(String, #[source] ParseStatusError),

    /// Il campo `trigger` di una riga di `lifecycle_transition` non si mappa
    /// a uno `Trigger` noto.
    #[error("failed to parse trigger '{0}': {1}")]
    ParseTrigger(String, #[source] ParseTriggerError),

    /// Errore di query di lettura (`prepare`/`query_map`/riga) sulle tabelle
    /// del lifecycle. Non è una migrazione: una SELECT non modifica lo schema.
    #[error("query error: {0}")]
    Query(String),
}

/// Seam di sola lettura del lifecycle: il caller chiede "quali stati
/// esistono" e "quali transizioni sono permesse" senza conoscere SQL né lo
/// schema.
///
/// Object-safe: usabile sia generico (`&impl LifecycleRepo`) che come
/// trait object (`//! trait object (`&dyn LifecycleRepo`).dyn LifecycleRepo`).
pub trait LifecycleRepo: Send + Sync {
    /// Restituisce tutti gli stati del lifecycle noti al repository.
    fn states(&self) -> Result<Vec<Status>, RepoError>;

    /// Restituisce tutte le transizioni permesse note al repository.
    fn allowed_transitions(&self) -> Result<Vec<Transition>, RepoError>;
}

pub mod memory;
pub mod sqlite;

pub use memory::MemoryRepo;
pub use sqlite::SqliteRepo;
