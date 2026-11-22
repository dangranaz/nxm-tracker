//! `transition_session` — cambio di stato di una sessione.
//!
//! Aggiorna lo status di una sessione e inserisce una riga in `audit_log`,
//! tutto in un'unica transazione atomica. La transizione deve essere nel
//! known-set del dominio (`valid_transitions`).

use std::time::Instant;

use crate::domain::action_context::ActionContext;
use crate::domain::transition::{is_transition_allowed, Trigger};
use crate::domain::Status;
use crate::repo::SqliteRepo;
use crate::writes::error::WriteError;
use nxm_db::Db;
use sqlx::Row;

/// Cambia lo stato di una sessione esistente.
///
/// # Parametri
/// - `db`: handle al database SQLite.
/// - `session_id`: id della sessione.
/// - `from_state`: stato attuale atteso (opzionale, per verifica ottimistica).
/// - `to_state`: nuovo stato.
/// - `trigger`: trigger che causa la transizione.
/// - `ctx`: contesto di azione (attribuzione obbligatoria).
/// - `now`: timestamp corrente (RFC3339).
///
/// # Errors
/// Restituisce `WriteError::SessionNotFound` se la sessione non esiste,
/// `WriteError::TransitionNotAllowed` se la transizione non è permessa,
/// `WriteError::Db` per errori di database.
pub fn transition_session(
    db: nxm_db::Db,
    session_id: &str,
    from_state: Option<Status>,
    to_state: Status,
    trigger: Trigger,
    ctx: &ActionContext,
    now: &str,
) -> Result<Status, WriteError> {
    let started = Instant::now();
    tracing::debug!(
        target: "nexum::tracker::writes",
        session_id = %session_id,
        to = %to_state,
        trigger = %trigger,
        "ENTER transition_session"
    );

    let repo = SqliteRepo::from_database(db);
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

    // Determina lo stato attuale.
    let from = match from_state {
        Some(s) => s,
        None => {
            let current: String = repo
                .block_on(
                    sqlx::query("SELECT status FROM session WHERE id = ?1")
                        .bind(session_id)
                        .fetch_one(&pool),
                )
                .map_err(|e| WriteError::Db(e.into()))?
                .try_get("status")
                .map_err(|e| WriteError::Db(e.into()))?;
            Status::try_from(current.as_str()).map_err(|e| WriteError::Write(e.to_string()))?
        }
    };

    // Verifica che la transizione sia permessa.
    if !is_transition_allowed(from, to_state, trigger) {
        return Err(WriteError::TransitionNotAllowed {
            from,
            to: to_state,
            trigger,
        });
    }

    // Aggiorna lo stato della sessione.
    repo.block_on(
        sqlx::query(r#"UPDATE session SET status = ?1, updated_at = ?2 WHERE id = ?3"#)
            .bind(to_state.as_str())
            .bind(now)
            .bind(session_id)
            .execute(&pool),
    )
    .map_err(|e| WriteError::Db(e.into()))?;

    // Inserisce la riga di audit.
    repo.block_on(
        sqlx::query(
            r#"INSERT INTO audit_log
               (session_id, action, from_state, to_state, user, auth_method, ai_tool, ai_model, at)
               VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9)"#,
        )
        .bind(session_id)
        .bind("set_status")
        .bind(from.as_str())
        .bind(to_state.as_str())
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
        from = %from,
        to = %to_state,
        elapsed_ms = started.elapsed().as_millis(),
        "EXIT transition_session"
    );

    Ok(to_state)
}
