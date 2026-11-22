//! `create_task` — crea una nuova task revision (versione 0).

use std::time::Instant;

use crate::domain::action_context::ActionContext;
use crate::domain::task_status::TaskStatus;
use crate::repo::SqliteRepo;
use crate::writes::error::WriteError;
use sqlx::SqlitePool;
use sqlx::Row;

/// Crea una nuova task con la prima revisione (versione 0).
///
/// # Parametri
/// - `db`: handle al database SQLite.
/// - `session_id`: id della sessione padre (deve esistere).
/// - `task_id`: id progressivo del task nella sessione.
/// - `title`: titolo del task (non vuoto).
/// - `body`: corpo del task (opzionale).
/// - `exec_order`: ordine di esecuzione.
/// - `ctx`: contesto di azione (attribuzione obbligatoria).
/// - `now`: timestamp corrente (RFC3339).
///
/// # Errors
/// Restituisce `WriteError::SessionNotFound` se la sessione non esiste,
/// `WriteError::EmptyField` se il titolo è vuoto,
/// `WriteError::Db` per errori di database.
pub fn create_task(
    db: sqlx::SqlitePool,
    session_id: &str,
    task_id: i64,
    title: &str,
    body: Option<&str>,
    exec_order: i64,
    ctx: &ActionContext,
    now: &str,
) -> Result<i64, WriteError> {
    let started = std::time::Instant::now();
    tracing::debug!(
        target: "nexum::tracker::writes",
        session_id = %session_id,
        task_id = task_id,
        "ENTER create_task"
    );

    if title.trim().is_empty() {
        return Err(WriteError::EmptyField { field: "title" });
    }

    let repo = SqliteRepo::from_pool(db);
    let pool = repo.pool()?;

    // Verifica che la sessione esista.
    let exists: bool = repo
        .block_on(
            sqlx::query("SELECT COUNT(*) > 0 FROM session WHERE id = ?1")
                .bind(session_id)
                .fetch_one(&pool),
        )
        .map_err(|e| WriteError::Db(e.into()))?
        .try_get::<bool, _>(0)
        .map_err(|e| WriteError::Db(e.into()))?;

    if !exists {
        return Err(WriteError::SessionNotFound(session_id.to_string()));
    }

    // Inserisce la task.
    repo.block_on(
        sqlx::query(
            r#"INSERT INTO task (session_id, task_id, created_at)
               VALUES (?1, ?2, ?3)"#,
        )
        .bind(session_id)
        .bind(task_id)
        .bind(now)
        .execute(&pool),
    )
    .map_err(|e| WriteError::Db(e.into()))?;

    // Inserisce la prima revisione (version = 0).
    repo.block_on(
        sqlx::query(
            r#"INSERT INTO task_revision
               (session_id, task_id, version, status, title, body, exec_order, result,
                spawned_task_id, blocked_by, at)
               VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11)"#,
        )
        .bind(session_id)
        .bind(task_id)
        .bind(0i64)
        .bind(TaskStatus::Open.as_str())
        .bind(title)
        .bind(body)
        .bind(exec_order)
        .bind::<Option<String>>(None)
        .bind::<Option<i64>>(None)
        .bind::<Option<i64>>(None)
        .bind(now)
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
        .bind("create")
        .bind::<Option<String>>(None)
        .bind(TaskStatus::Open.as_str())
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
        elapsed_ms = started.elapsed().as_millis(),
        "EXIT create_task"
    );

    Ok(task_id)
}
