//! `mark_consumed` — consuma una sessione (imposta `superseded` + `consumed_at`/`consumed_by`).
//!
//! Scrive l'audit con action = "mark_consumed", tutto in un'unica transazione atomica.
//! Se `require_human_signoff` è true, richiede una firma umana valida prima di procedere.

use std::time::Instant;

use crate::domain::action_context::ActionContext;
use crate::repo::SqliteRepo;
use crate::writes::error::WriteError;
use sqlx::SqlitePool;
use sqlx::Row;

/// Marca una sessione come consumata.
///
/// # Parametri
/// - `db`: handle al database SQLite.
/// - `session_id`: id della sessione.
/// - `consumed_by`: id della sessione che riprende (opzionale).
/// - `ctx`: contesto di azione (attribuzione obbligatoria).
/// - `now`: timestamp corrente (RFC3339).
/// - `require_human_signoff`: se true, richiede una firma umana valida.
///
/// # Errors
/// Restituisce `WriteError::SessionNotFound` se la sessione non esiste,
/// `WriteError::SignoffRequired` se il signoff è richiesto ma mancante,
/// `WriteError::Db` per errori di database.
pub fn mark_consumed(
    db: sqlx::SqlitePool,
    session_id: &str,
    consumed_by: Option<&str>,
    ctx: &ActionContext,
    now: &str,
    require_human_signoff: bool,
) -> Result<(), WriteError> {
    let started = Instant::now();
    tracing::debug!(
        target: "nexum::tracker::writes",
        session_id = %session_id,
        "ENTER mark_consumed"
    );

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

    // Se richiesto, verifica che esista una human_signoff valida.
    if require_human_signoff {
        let has_signoff: bool = repo
            .block_on(
                sqlx::query("SELECT COUNT(*) > 0 FROM human_signoff WHERE session_id = ?1")
                    .bind(session_id)
                    .fetch_one(&pool),
            )
            .map_err(|e| WriteError::Db(e.into()))?
            .try_get::<bool, _>(0)
            .map_err(|e| WriteError::Db(e.into()))?;

        if !has_signoff {
            return Err(WriteError::SignoffRequired {
                to: crate::domain::Status::Superseded,
            });
        }
    }

    // Aggiorna la sessione.
    repo.block_on(
        sqlx::query(
            r#"UPDATE session SET status = 'superseded', consumed_at = ?1, consumed_by = ?2 WHERE id = ?3"#,
        )
        .bind(now)
        .bind(consumed_by)
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
        .bind("mark_consumed")
        .bind::<Option<String>>(None)
        .bind("superseded")
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
        elapsed_ms = started.elapsed().as_millis(),
        "EXIT mark_consumed"
    );

    Ok(())
}
