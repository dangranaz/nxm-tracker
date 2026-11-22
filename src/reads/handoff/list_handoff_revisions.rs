//! `list_handoff_revisions` — lista tutte le revisioni di handoff di una sessione.

use crate::reads::handoff::error::HandoffReadError;
use crate::reads::handoff::view::HandoffView;
use crate::repo::SqliteRepo;
use nxm_db::Db;
use sqlx::Row;

/// Lista tutte le revisioni di handoff di una sessione, ordinate per versione discendente.
pub fn list_handoff_revisions(
    db: nxm_db::Db,
    session_id: &str,
) -> Result<Vec<HandoffView>, HandoffReadError> {
    let repo = SqliteRepo::from_database(db);
    let pool = repo.pool()?;

    let rows = repo
        .block_on(
            sqlx::query(
                r#"SELECT id, session_id, version, goal_next, decisions, next_steps, failed_approaches, relevant_files, context_usage_pct, at
                   FROM session_handoff
                   WHERE session_id = ?1
                   ORDER BY version DESC"#,
            )
            .bind(session_id)
            .fetch_all(&pool),
        )
        .map_err(|e| HandoffReadError::Db(e.into()))?;

    let mut revisions = Vec::with_capacity(rows.len());
    for row in rows {
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

        revisions.push(HandoffView {
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
        });
    }

    Ok(revisions)
}
