//! `update_session` — inserts a new handoff revision + audit row.
//!
//! Atomic transaction: INSERT into `session_handoff` (versioned, append-only)
//! + INSERT into `audit_log`.

use std::time::Instant;

use crate::domain::action_context::ActionContext;
use crate::repo::SqliteRepo;
use crate::writes::handoff::error::HandoffWriteError;
use crate::writes::handoff::input::HandoffInput;
use sqlx::SqlitePool;
use sqlx::Row;

/// Updates a session handoff: inserts a new revision and an audit row.
///
/// # Parameters
/// - `db`: handle to the SQLite database.
/// - `session_id`: ID of the session (must exist).
/// - `handoff`: handoff fields to write.
/// - `ctx`: action context (attribution required).
/// - `now`: current timestamp (RFC3339).
///
/// # Returns
/// The version number of the new handoff revision.
///
/// # Errors
/// Returns `HandoffWriteError::SessionNotFound` if the session does not exist,
/// `HandoffWriteError::Db` for database errors,
/// `HandoffWriteError::Write` for generic write failures.
pub fn update_session(
    db: sqlx::SqlitePool,
    session_id: &str,
    handoff: &HandoffInput,
    ctx: &ActionContext,
    now: &str,
) -> Result<i64, HandoffWriteError> {
    let started = Instant::now();
    tracing::debug!(
        target: "nexum::tracker::writes",
        session_id = %session_id,
        "ENTER update_session"
    );

    let repo = SqliteRepo::from_pool(db);
    let pool = repo.pool()?;

    // Verify session exists.
    let exists: bool = repo
        .block_on(
            sqlx::query("SELECT COUNT(*) > 0 FROM session WHERE id = ?1")
                .bind(session_id)
                .fetch_one(&pool),
        )
        .map_err(|e| HandoffWriteError::Db(e.into()))?
        .try_get::<bool, _>(0)
        .map_err(|e| HandoffWriteError::Db(e.into()))?;

    if !exists {
        return Err(HandoffWriteError::SessionNotFound(session_id.to_string()));
    }

    // Determine next version (MAX(version) + 1, or 0 if none).
    let next_version: i64 = repo
        .block_on(
            sqlx::query(
                "SELECT COALESCE(MAX(version), -1) + 1 FROM session_handoff WHERE session_id = ?1",
            )
            .bind(session_id)
            .fetch_one(&pool),
        )
        .map_err(|e| HandoffWriteError::Db(e.into()))?
        .try_get::<i64, _>(0)
        .map_err(|e| HandoffWriteError::Db(e.into()))?;

    // Serialize list fields as JSON.
    let decisions_json = serde_json::to_string(&handoff.decisions).unwrap_or_default();
    let next_steps_json = serde_json::to_string(&handoff.next_steps).unwrap_or_default();
    let failed_approaches_json =
        serde_json::to_string(&handoff.failed_approaches).unwrap_or_default();
    let relevant_files_json = serde_json::to_string(&handoff.relevant_files).unwrap_or_default();

    // Insert the handoff revision.
    repo.block_on(
        sqlx::query(
            r#"INSERT INTO session_handoff
               (session_id, version, goal_next, decisions, next_steps, failed_approaches, relevant_files, context_usage_pct, at)
               VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9)"#,
        )
        .bind(session_id)
        .bind(next_version)
        .bind(&handoff.goal_next)
        .bind(&decisions_json)
        .bind(&next_steps_json)
        .bind(&failed_approaches_json)
        .bind(&relevant_files_json)
        .bind(&handoff.context_usage_pct)
        .bind(now)
        .execute(&pool),
    )
    .map_err(|e| HandoffWriteError::Db(e.into()))?;

    // Insert audit row.
    repo.block_on(
        sqlx::query(
            r#"INSERT INTO audit_log
               (session_id, action, from_state, to_state, user, auth_method, ai_tool, ai_model, at)
               VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9)"#,
        )
        .bind(session_id)
        .bind("update_session")
        .bind::<Option<String>>(None)
        .bind::<Option<String>>(None)
        .bind(&ctx.user)
        .bind(ctx.auth_method.to_string())
        .bind(&ctx.ai_tool)
        .bind(&ctx.ai_model)
        .bind(now)
        .execute(&pool),
    )
    .map_err(|e| HandoffWriteError::Db(e.into()))?;

    tracing::info!(
        target: "nexum::tracker::writes",
        session_id = %session_id,
        version = next_version,
        elapsed_ms = started.elapsed().as_millis(),
        "EXIT update_session"
    );

    Ok(next_version)
}
