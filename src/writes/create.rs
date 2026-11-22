//! `create_session` — creazione di una nuova sessione del lifecycle.
//!
//! Inserisce una riga in `session` con stato `active` e una riga in
//! `audit_log` (action = "create", from_state = NULL, to_state = "active"),
//! tutto in un'unica transazione atomica.
//!
//! Se `working_directory` è fornito, verifica che sia un git repository e
//! crea un branch `integration_<short>` prima di creare la sessione.

use std::time::Instant;

use crate::domain::action_context::ActionContext;
use crate::domain::Status;
use crate::git_workspace::{base_branch, create_integration_branch, is_git_repo};
use crate::repo::SqliteRepo;
use crate::writes::error::WriteError;
use sqlx::SqlitePool;
use sqlx::Row;

/// Errore restituito da `create_session` se il nome è vuoto.
const EMPTY_NAME_ERROR: &str = "name";

/// Crea una nuova sessione del lifecycle.
///
/// # Parametri
/// - `db`: handle al database SQLite.
/// - `session_id`: id univoco della sessione (slug-based, non vuoto).
/// - `name`: nome della sessione (non vuoto).
/// - `ctx`: contesto di azione (attribuzione obbligatoria).
/// - `working_directory`: percorso opzionale a un progetto git.
/// - `now`: timestamp corrente (RFC3339).
///
/// # Errors
/// Restituisce `WriteError::DuplicateId` se la sessione esiste già,
/// `WriteError::EmptyField` se nome o id sono vuoti,
/// `WriteError::Db` per errori di database.
pub fn create_session(
    db: sqlx::SqlitePool,
    session_id: &str,
    name: &str,
    ctx: &ActionContext,
    working_directory: Option<&str>,
    now: &str,
) -> Result<Status, WriteError> {
    let started = Instant::now();
    tracing::debug!(
        target: "nexum::tracker::writes",
        session_id = %session_id,
        "ENTER create_session"
    );

    // Valida input.
    if session_id.trim().is_empty() {
        return Err(WriteError::InvalidSlug {
            slug: session_id.to_string(),
        });
    }
    if name.trim().is_empty() {
        return Err(WriteError::EmptyField {
            field: EMPTY_NAME_ERROR,
        });
    }

    // Se c'è una working directory, verifica git e crea il branch di integrazione.
    if let Some(path) = working_directory {
        is_git_repo(path).map_err(|e| WriteError::Write(e.to_string()))?;
        let base = base_branch(path).map_err(|e| WriteError::Write(e.to_string()))?;
        create_integration_branch(path, &base, name)
            .map_err(|e| WriteError::Write(e.to_string()))?;
    }

    let repo = SqliteRepo::from_pool(db);
    let pool = repo.pool()?;

    // Verifica che la sessione non esista già.
    let exists: bool = repo
        .block_on(
            sqlx::query("SELECT COUNT(*) > 0 FROM session WHERE id = ?1")
                .bind(session_id)
                .fetch_one(&pool),
        )
        .map_err(|e| WriteError::Db(e.into()))?
        .try_get(0)
        .map_err(|e| WriteError::Db(e.into()))?;

    if exists {
        return Err(WriteError::DuplicateId(session_id.to_string()));
    }

    // Inserisce la sessione con stato active.
    let initial_status = Status::Active;
    repo.block_on(
        sqlx::query(
            r#"INSERT INTO session (id, status, created_at, updated_at)
               VALUES (?1, ?2, ?3, ?3)"#,
        )
        .bind(session_id)
        .bind(initial_status.as_str())
        .bind(now)
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
        .bind("create")
        .bind::<Option<String>>(None)
        .bind(initial_status.as_str())
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
        status = %initial_status,
        elapsed_ms = started.elapsed().as_millis(),
        "EXIT create_session"
    );

    Ok(initial_status)
}
