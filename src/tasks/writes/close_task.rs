//! `close_task` — chiude un task (imposta stato `cancelled`).

use std::time::Instant;

use super::set_task_status;
use crate::domain::action_context::ActionContext;
use crate::domain::task_status::TaskStatus;
use crate::repo::SqliteRepo;
use crate::writes::error::WriteError;
use sqlx::Row;

/// Chiude un task (imposta stato `cancelled`).
///
/// # Parametri
/// - `db`: handle al database SQLite.
/// - `session_id`: id della sessione padre.
/// - `task_id`: id del task.
/// - `ctx`: contesto di azione.
/// - `now`: timestamp corrente (RFC3339).
///
/// # Errors
/// Restituisce `WriteError::SessionNotFound` o `WriteError::TaskNotFound`
/// se sessione o task non esistono.
pub fn close_task(
    db: sqlx::SqlitePool,
    session_id: &str,
    task_id: i64,
    ctx: &ActionContext,
    now: &str,
) -> Result<i64, WriteError> {
    set_task_status(db, session_id, task_id, TaskStatus::Cancelled, ctx, now)
}
