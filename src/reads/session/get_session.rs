//! `get_session` — lettura di una sessione per ID.

use crate::domain::Status;
use crate::reads::session::error::SessionReadError;
use crate::reads::session::view::SessionView;
use crate::repo::SqliteRepo;
use sqlx::SqlitePool;
use sqlx::Row;

/// Legge una sessione per ID.
pub fn get_session(db: sqlx::SqlitePool, session_id: &str) -> Result<SessionView, SessionReadError> {
    let repo = SqliteRepo::from_pool(db);
    let pool = repo.pool()?;

    let row = repo
        .block_on(
            sqlx::query(
                r#"SELECT id, name, started_at, ended_at, status, design_phase_complete, consumed_at, consumed_by
                   FROM session WHERE id = ?1"#,
            )
            .bind(session_id)
            .fetch_optional(&pool),
        )
        .map_err(|e| SessionReadError::Db(e.into()))?;

    let row = match row {
        Some(r) => r,
        None => return Err(SessionReadError::SessionNotFound(session_id.to_string())),
    };

    let id: String = row
        .try_get("id")
        .map_err(|e| SessionReadError::Db(e.into()))?;
    let name: String = row
        .try_get("name")
        .map_err(|e| SessionReadError::Db(e.into()))?;
    let started_at: String = row
        .try_get("started_at")
        .map_err(|e| SessionReadError::Db(e.into()))?;
    let ended_at: Option<String> = row.try_get("ended_at").ok().flatten();
    let status_str: String = row
        .try_get("status")
        .map_err(|e| SessionReadError::Db(e.into()))?;
    let design_phase_complete: i32 = row
        .try_get("design_phase_complete")
        .map_err(|e| SessionReadError::Db(e.into()))?;
    let consumed_at: Option<String> = row.try_get("consumed_at").ok().flatten();
    let consumed_by: Option<String> = row.try_get("consumed_by").ok().flatten();

    let status =
        Status::try_from(status_str.as_str()).map_err(|e| SessionReadError::Read(e.to_string()))?;

    Ok(SessionView {
        id,
        name,
        started_at,
        ended_at,
        status,
        design_phase_complete: design_phase_complete != 0,
        consumed_at,
        consumed_by,
    })
}
