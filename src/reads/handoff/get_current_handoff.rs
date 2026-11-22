//! `get_current_handoff` — legge l'handoff corrente (versione massima) di una sessione.

use crate::reads::handoff::error::HandoffReadError;
use crate::reads::handoff::view::HandoffView;
use crate::repo::SqliteRepo;
use nxm_db::Db;
use sqlx::Row;

/// Legge l'handoff corrente (versione massima) di una sessione.
pub fn get_current_handoff(
    db: nxm_db::Db,
    session_id: &str,
) -> Result<HandoffView, HandoffReadError> {
    let repo = SqliteRepo::from_database(db);
    let pool = repo.pool()?;

    // Verifica che la sessione esista.
    let exists: bool = repo
        .block_on(
            sqlx::query("SELECT COUNT(*) > 0 FROM session WHERE id = ?1")
                .bind(session_id)
                .fetch_one(&pool),
        )
        .map_err(|e| HandoffReadError::Db(e.into()))?
        .try_get::<bool, _>(0)
        .map_err(|e| HandoffReadError::Db(e.into()))?;

    if !exists {
        return Err(HandoffReadError::SessionNotFound(session_id.to_string()));
    }

    // Legge l'handoff corrente (MAX version).
    let row = repo
        .block_on(
            sqlx::query(
                r#"SELECT id, session_id, version, goal_next, decisions, next_steps, failed_approaches, relevant_files, context_usage_pct, at
                   FROM session_handoff
                   WHERE session_id = ?1 AND version = (SELECT MAX(version) FROM session_handoff WHERE session_id = ?1)"#,
            )
            .bind(session_id)
            .bind(session_id)
            .fetch_optional(&pool),
        )
        .map_err(|e| HandoffReadError::Db(e.into()))?;

    let row = match row {
        Some(r) => r,
        None => return Err(HandoffReadError::NoHandoff(session_id.to_string())),
    };

    let id: i64 = row
        .try_get("id")
        .map_err(|e| HandoffReadError::Db(e.into()))?;
    let session_id: String = row
        .try_get("session_id")
        .map_err(|e| HandoffReadError::Db(e.into()))?;
    let version: i64 = row
        .try_get("version")
        .map_err(|e| HandoffReadError::Db(e.into()))?;
    let goal_next: Option<String> = row.try_get("goal_next").ok().flatten();
    let decisions: Vec<String> = HandoffView::from_json_list(
        row.try_get::<Option<String>, _>("decisions")
            .ok()
            .flatten()
            .as_deref(),
    );
    let next_steps: Vec<String> = HandoffView::from_json_list(
        row.try_get::<Option<String>, _>("next_steps")
            .ok()
            .flatten()
            .as_deref(),
    );
    let failed_approaches: Vec<String> = HandoffView::from_json_list(
        row.try_get::<Option<String>, _>("failed_approaches")
            .ok()
            .flatten()
            .as_deref(),
    );
    let relevant_files: Vec<String> = HandoffView::from_json_list(
        row.try_get::<Option<String>, _>("relevant_files")
            .ok()
            .flatten()
            .as_deref(),
    );
    let context_usage_pct: Option<i64> = row.try_get("context_usage_pct").ok().flatten();
    let at: String = row
        .try_get("at")
        .map_err(|e| HandoffReadError::Db(e.into()))?;

    Ok(HandoffView {
        id,
        session_id,
        version,
        goal_next,
        decisions,
        next_steps,
        failed_approaches,
        relevant_files,
        context_usage_pct,
        at,
    })
}
