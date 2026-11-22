//! `list_session_tasks` — lista tutte le task revision di una sessione.

use crate::domain::task_status::TaskStatus;
use crate::repo::SqliteRepo;
use crate::tasks::reads::error::TaskReadError;
use crate::tasks::reads::view::TaskView;
use sqlx::SqlitePool;
use sqlx::Row;

/// Lista tutte le revisioni di task di una sessione, ordinate per task_id, versione discendente.
pub fn list_session_tasks(
    db: sqlx::SqlitePool,
    session_id: &str,
) -> Result<Vec<TaskView>, TaskReadError> {
    let repo = SqliteRepo::from_pool(db);
    let pool = repo.pool()?;

    let rows = repo
        .block_on(
            sqlx::query(
                r#"SELECT tr.id, tr.session_id, tr.task_id, tr.version, tr.status, tr.title,
                          tr.body, tr.exec_order, tr.result, tr.spawned_task_id, tr.blocked_by,
                          t.created_at, tr.at
                   FROM task_revision tr
                   JOIN task t ON tr.session_id = t.session_id AND tr.task_id = t.task_id
                   WHERE tr.session_id = ?1 AND tr.version = (
                       SELECT MAX(version) FROM task_revision tr2
                       WHERE tr2.session_id = tr.session_id AND tr2.task_id = tr.task_id
                   )
                   ORDER BY tr.task_id, tr.version DESC"#,
            )
            .bind(session_id)
            .fetch_all(&pool),
        )
        .map_err(|e| TaskReadError::Db(e.into()))?;

    let mut tasks = Vec::with_capacity(rows.len());
    for row in rows {
        let status_str: String = row
            .try_get("status")
            .map_err(|e| TaskReadError::Db(e.into()))?;
        let status = TaskStatus::try_from(status_str.as_str())
            .map_err(|e| TaskReadError::Read(e.to_string()))?;

        tasks.push(TaskView {
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
        });
    }

    Ok(tasks)
}
