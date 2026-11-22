//! `list_sessions` — lista tutte le sessioni, ordinate per `started_at` discendente.

use crate::domain::Status;
use crate::reads::session::error::SessionReadError;
use crate::reads::session::view::SessionView;
use crate::repo::SqliteRepo;
use nxm_db::Db;
use sqlx::Row;

/// Lista tutte le sessioni di lavoro, ordinate per `started_at` discendente.
pub fn list_sessions(db: nxm_db::Db) -> Result<Vec<SessionView>, SessionReadError> {
    let repo = SqliteRepo::from_database(db);
    let pool = repo.pool()?;

    let rows = repo
        .block_on(
            sqlx::query(
                r#"SELECT id, name, started_at, ended_at, status, design_phase_complete, consumed_at, consumed_by
                   FROM session ORDER BY started_at DESC"#,
            )
            .fetch_all(&pool),
        )
        .map_err(|e| SessionReadError::Read(e.to_string()))?;

    let mut sessions = Vec::with_capacity(rows.len());
    for row in rows {
        let id: String = row
            .try_get("id")
            .map_err(|e| SessionReadError::Read(e.to_string()))?;
        let name: String = row
            .try_get("name")
            .map_err(|e| SessionReadError::Read(e.to_string()))?;
        let started_at: String = row
            .try_get("started_at")
            .map_err(|e| SessionReadError::Read(e.to_string()))?;
        let ended_at: Option<String> = row.try_get("ended_at").ok().flatten();
        let status_str: String = row
            .try_get("status")
            .map_err(|e| SessionReadError::Read(e.to_string()))?;
        let design_phase_complete: i32 = row
            .try_get("design_phase_complete")
            .map_err(|e| SessionReadError::Read(e.to_string()))?;
        let consumed_at: Option<String> = row.try_get("consumed_at").ok().flatten();
        let consumed_by: Option<String> = row.try_get("consumed_by").ok().flatten();

        let status = Status::try_from(status_str.as_str())
            .map_err(|e| SessionReadError::Read(e.to_string()))?;

        sessions.push(SessionView {
            id,
            name,
            started_at,
            ended_at,
            status,
            design_phase_complete: design_phase_complete != 0,
            consumed_at,
            consumed_by,
        });
    }

    Ok(sessions)
}
