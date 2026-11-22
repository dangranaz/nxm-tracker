//! `get_task` — legge una task revision per session_id e task_id.

use super::error::TaskReadError;
use super::view::TaskView;
use crate::domain::task_status::TaskStatus;
use crate::repo::SqliteRepo;
use nxm_db::Db;
use sqlx::Row;

/// Legge l'ultima revisione di un task.
pub fn get_task(db: nxm_db::Db, session_id: &str, task_id: i64) -> Result<TaskView, TaskReadError> {
    let repo = SqliteRepo::from_database(db);
    let pool = repo.pool()?;

    let row = repo
        .block_on(
            sqlx::query(
                r#"SELECT tr.id, tr.session_id, tr.task_id, tr.version, tr.status, tr.title,
                          tr.body, tr.exec_order, tr.result, tr.spawned_task_id, tr.blocked_by,
                          t.created_at, tr.at
                   FROM task_revision tr
                   JOIN task t ON tr.session_id = t.session_id AND tr.task_id = t.task_id
                   WHERE tr.session_id = ?1 AND tr.task_id = ?2 AND tr.version = (
                       SELECT MAX(version) FROM task_revision
                       WHERE session_id = ?1 AND task_id = ?2
                   )"#,
            )
            .bind(session_id)
            .bind(task_id)
            .fetch_optional(&pool),
        )
        .map_err(|e| TaskReadError::Db(e.into()))?;

    let row = match row {
        Some(r) => r,
        None => return Err(TaskReadError::TaskNotFound(session_id.to_string(), task_id)),
    };

    let status_str: String = row
        .try_get("status")
        .map_err(|e| TaskReadError::Db(e.into()))?;
    let status = TaskStatus::try_from(status_str.as_str())
        .map_err(|e| TaskReadError::Read(e.to_string()))?;

    Ok(TaskView {
        session_id: row
            .try_get("session_id")
            .map_err(|e| TaskReadError::Db(e.into()))?,
        task_id: row
            .try_get("task_id")
            .map_err(|e| TaskReadError::Db(e.into()))?,
        version: row
            .try_get("version")
            .map_err(|e| TaskReadError::Db(e.into()))?,
        status,
        title: row
            .try_get("title")
            .map_err(|e| TaskReadError::Db(e.into()))?,
        body: row.try_get("body").ok().flatten(),
        exec_order: row
            .try_get("exec_order")
            .map_err(|e| TaskReadError::Db(e.into()))?,
        result: row.try_get("result").ok().flatten(),
        spawned_task_id: row.try_get("spawned_task_id").ok().flatten(),
        blocked_by: row.try_get("blocked_by").ok().flatten(),
        created_at: row
            .try_get("created_at")
            .map_err(|e| TaskReadError::Db(e.into()))?,
        at: row.try_get("at").map_err(|e| TaskReadError::Db(e.into()))?,
    })
}
