//! `register_prompt` — registrazione di un riferimento a un brief di delega
//! (prompt registry, metadato puro, nessun audit).
//!
//! Upsert per (session_id, task_no, lang): ri-registrare la stessa tripla
//! aggiorna slug/path in place (idempotente).

use std::time::Instant;

use crate::repo::SqliteRepo;
use sqlx::SqlitePool;
use sqlx::Row;

/// Errore tipizzato del prompt registry.
#[derive(Debug, thiserror::Error)]
pub enum PromptError {
    /// La sessione referenziata non esiste.
    #[error("session not found: {0}")]
    SessionNotFound(String),

    /// Il valore di `lang` non è uno dei due attesi (`ita`/`eng`).
    #[error("invalid lang: {lang}")]
    InvalidLang { lang: String },

    /// Errore di query sul database.
    #[error("query error: {0}")]
    Query(String),

    /// Errore generico di scrittura.
    #[error("write error: {0}")]
    Write(String),
}

/// Valori validi per `lang` nel prompt registry.
const VALID_LANGS: &[&str] = &["ita", "eng"];

/// Registra (o aggiorna, upsert) il riferimento a un brief di delega.
///
/// L'upsert avviene per `(session_id, task_no, lang)`: ri-registrare la
// stessa tripla aggiorna `slug` e `path` in place (idempotente).
///
/// Non scrive `audit_log` e non richiede `ActionContext` (metadato leggero).
pub fn register_prompt(
    db: sqlx::SqlitePool,
    session_id: &str,
    task_no: &str,
    slug: &str,
    lang: &str,
    path: &str,
) -> Result<(), PromptError> {
    let started = Instant::now();
    tracing::debug!(
        target: "nexum::tracker::writes",
        session_id = %session_id,
        task_no = %task_no,
        lang = %lang,
        "ENTER register_prompt"
    );

    if !VALID_LANGS.contains(&lang) {
        return Err(PromptError::InvalidLang {
            lang: lang.to_string(),
        });
    }

    let repo = SqliteRepo::from_pool(db);
    let pool = repo.pool().map_err(|e| PromptError::Query(e.to_string()))?;

    // Verifica che la sessione esista.
    let exists: bool = repo
        .block_on(
            sqlx::query("SELECT COUNT(*) > 0 FROM session WHERE id = ?1")
                .bind(session_id)
                .fetch_one(&pool),
        )
        .map_err(|e| PromptError::Query(e.to_string()))?
        .try_get::<bool, _>(0)
        .map_err(|e| PromptError::Query(e.to_string()))?;

    if !exists {
        return Err(PromptError::SessionNotFound(session_id.to_string()));
    }

    // Upsert per (session_id, task_no, lang).
    repo.block_on(
        sqlx::query(
            "INSERT INTO prompt (session_id, task_no, slug, lang, path)
             VALUES (?1, ?2, ?3, ?4, ?5)
             ON CONFLICT(session_id, task_no, lang) DO UPDATE SET
               slug = excluded.slug, path = excluded.path",
        )
        .bind(session_id)
        .bind(task_no)
        .bind(slug)
        .bind(lang)
        .bind(path)
        .execute(&pool),
    )
    .map_err(|e| PromptError::Write(e.to_string()))?;

    tracing::info!(
        target: "nexum::tracker::writes",
        session_id = %session_id,
        task_no = %task_no,
        lang = %lang,
        elapsed_ms = started.elapsed().as_millis(),
        "EXIT register_prompt"
    );

    Ok(())
}
