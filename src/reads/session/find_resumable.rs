//! `find_resumable_session` — trova una sessione da riprendere.
//!
//! Restituisce la prima sessione con stato `active` o `onhold`, ordinata
//! per `updated_at` discendente.

use crate::domain::Status;
use crate::reads::session::error::SessionReadError;
use crate::reads::session::view::SessionView;
use crate::repo::SqliteRepo;
use nxm_db::Db;
use sqlx::Row;

/// Sessione ripristinabile trovata.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ResumableSession {
    pub view: SessionView,
}

/// Trova una sessione da riprendere (stato `active` o `onhold`).
pub fn find_resumable_session(
    db: nxm_db::Db,
) -> Result<Option<ResumableSession>, SessionReadError> {
    let repo = SqliteRepo::from_database(db);
    let pool = repo.pool()?;

    let row = repo
        .block_on(
            sqlx::query(
                r#"SELECT id, name, started_at, ended_at, status, design_phase_complete, consumed_at, consumed_by
                   FROM session WHERE status IN ('active', 'onhold')
                   ORDER BY updated_at DESC LIMIT 1"#,
            )
            .fetch_optional(&pool),
        )
        .map_err(|e| SessionReadError::Db(e.into()))?;

    let row = match row {
        Some(r) => r,
        None => return Ok(None),
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

    Ok(Some(ResumableSession {
        view: SessionView {
            id,
            name,
            started_at,
            ended_at,
            status,
            design_phase_complete: design_phase_complete != 0,
            consumed_at,
            consumed_by,
        },
    }))
}
