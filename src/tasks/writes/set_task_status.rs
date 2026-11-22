//! `set_task_status` — cambia lo stato di un task (nuova revisione).

use std::time::Instant;

use crate::domain::action_context::ActionContext;
use crate::domain::task_status::TaskStatus;
use crate::repo::SqliteRepo;
use crate::writes::error::WriteError;
use sqlx::SqlitePool;
use sqlx::Row;

/// Cambia lo stato di un task (crea una nuova revisione).
///
/// # Parametri
/// - `db`: handle al database SQLite.
/// - `session_id`: id della sessione padre.
/// - `task_id`: id del task.
/// - `status`: nuovo stato del task.
/// - `ctx`: contesto di azione.
/// - `now`: timestamp corrente (RFC3339).
///
/// # Errors
/// Restituisce `WriteError::SessionNotFound` o `WriteError::TaskNotFound`
/// se sessione o task non esistono.
pub fn set_task_status(
    db: sqlx::SqlitePool,
    session_id: &str,
    task_id: i64,
    status: TaskStatus,
    ctx: &ActionContext,
    now: &str,
) -> Result<i64, WriteError> {
    let started = std::time::Instant::now();
    tracing::debug!(
        target: "nexum::tracker::writes",
        session_id = %session_id,
        task_id = task_id,
        status = %status,
        "ENTER set_task_status"
    );

    let repo = SqliteRepo::from_pool(db);
    let pool = repo.pool()?;

    // Verifica che sessione e task esistano.
    let exists: bool = repo
        .block_on(
            sqlx::query("SELECT COUNT(*) > 0 FROM task WHERE session_id = ?1 AND task_id = ?2")
                .bind(session_id)
                .bind(task_id)
                .fetch_one(&pool),
        )
        .map_err(|e| WriteError::Db(e.into()))?
        .try_get::<bool, _>(0)
        .map_err(|e| WriteError::Db(e.into()))?;

    if !exists {
        return Err(WriteError::SessionNotFound(session_id.to_string()));
    }

    // Determina la prossima versione.
    let next_version: i64 = repo
        .block_on(
            sqlx::query("SELECT COALESCE(MAX(version), -1) + 1 FROM task_revision WHERE session_id = ?1 AND task_id = ?2")
                .bind(session_id)
                .bind(task_id)
                .fetch_one(&pool),
        )
        .map_err(|e| WriteError::Db(e.into()))?
        .try_get::<i64, _>(0)
        .map_err(|e| WriteError::Db(e.into()))?;

    // Legge lo stato precedente.
    let prev_status: String = repo
        .block_on(
            sqlx::query("SELECT status FROM task_revision WHERE session_id = ?1 AND task_id = ?2 AND version = (SELECT MAX(version) FROM task_revision WHERE session_id = ?1 AND task_id = ?2)")
                .bind(session_id)
                .bind(task_id)
                .bind(session_id)
                .bind(task_id)
                .fetch_optional(&pool),
        )
        .map_err(|e| WriteError::Db(e.into()))?
        .map(|r| r.try_get::<String, _>("status").unwrap_or_else(|_| "open".to_string()))
        .unwrap_or_else(|| TaskStatus::Open.as_str().to_string());

    // Inserisce la nuova revisione.
    repo.block_on(
        sqlx::query(
            r#"INSERT INTO task_revision
               (session_id, task_id, version, status, title, body, exec_order, result,
                spawned_task_id, blocked_by, at)
               SELECT session_id, task_id, ?1, ?2, title, body, exec_order, result,
                      spawned_task_id, blocked_by, ?3
               FROM task_revision
               WHERE session_id = ?4 AND task_id = ?5 AND version = (
                   SELECT MAX(version) FROM task_revision
                   WHERE session_id = ?4 AND task_id = ?5
               )"#,
        )
        .bind(next_version)
        .bind(status.as_str())
        .bind(now)
        .bind(session_id)
        .bind(task_id)
        .execute(&pool),
    )
    .map_err(|e| WriteError::Db(e.into()))?;

    // Inserisce la riga di audit.
    repo.block_on(
        sqlx::query(
            r#"INSERT INTO task_audit
               (session_id, task_id, action, from_status, to_status, user, auth_method, ai_tool, ai_model, at)
               VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10)"#,
        )
        .bind(session_id)
        .bind(task_id)
        .bind("set_status")
        .bind(prev_status.as_str())
        .bind(status.as_str())
        .bind(&ctx.user)
        .bind(ctx.auth_method.to_string())
        .bind(&ctx.ai_tool)
        .bind(&ctx.ai_model)
        .bind(now)
        .execute(&pool),
    )
    .map_err(|e| WriteError::Db(e.into()))?;

    tracing::info!(
        target: "nexum::tracker::writes",
        session_id = %session_id,
        task_id = task_id,
        version = next_version,
        elapsed_ms = started.elapsed().as_millis(),
        "EXIT set_task_status"
    );

    Ok(next_version)
}
