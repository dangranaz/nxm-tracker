//! SQLite implementation of [`TaskRepository`].
//!
//! Persists execution tasks in the `tasks` table and their final reports in
//! the 1:1 `task_reports` table, both created by
//! [`crate::default_migration_set`]. `close_task` runs in a transaction so the
//! status check, the report insert, and the status update are atomic.

use crate::error::TrackerError;
use crate::session_repository::SqliteSessionRepository;
use crate::task::{NewTask, Task, TaskReport, TaskRepository, TaskStatus};
use sqlx::Row;

impl SqliteSessionRepository {
    pub(crate) fn row_to_task(row: sqlx::sqlite::SqliteRow) -> Task {
        Task {
            id: row.try_get::<i64, _>("id").ok(),
            session_id: row.try_get::<i64, _>("session_id").unwrap_or(0),
            name: row.try_get::<String, _>("name").unwrap_or_default(),
            status: TaskStatus::try_from(
                row.try_get::<String, _>("status")
                    .unwrap_or_default()
                    .as_str(),
            )
            .unwrap_or(TaskStatus::Open),
            note: row.try_get::<Option<String>, _>("note").ok().flatten(),
            created_at: row.try_get::<String, _>("created_at").unwrap_or_default(),
            closed_at: row.try_get::<Option<String>, _>("closed_at").ok().flatten(),
        }
    }

    pub(crate) fn row_to_task_report(row: sqlx::sqlite::SqliteRow) -> TaskReport {
        TaskReport {
            id: row.try_get::<i64, _>("id").ok(),
            task_id: row.try_get::<i64, _>("task_id").unwrap_or(0),
            content: row.try_get::<String, _>("content").unwrap_or_default(),
            created_at: row.try_get::<String, _>("created_at").unwrap_or_default(),
        }
    }

    fn task_status_to_str(status: TaskStatus) -> &'static str {
        match status {
            TaskStatus::Open => "open",
            TaskStatus::Closed => "closed",
            TaskStatus::Invalid => "invalid",
        }
    }
}

impl TaskRepository for SqliteSessionRepository {
    fn create_task(&self, new: &NewTask) -> Result<Task, TrackerError> {
        let pool = self.pool()?;
        let now = chrono::Utc::now().to_rfc3339();
        let row = self
            .block_on(
                sqlx::query(
                    r#"INSERT INTO tasks (session_id, name, status, created_at)
                       VALUES (?1, ?2, 'open', ?3)
                       RETURNING id, session_id, name, status, note, created_at, closed_at"#,
                )
                .bind(new.session_id)
                .bind(&new.name)
                .bind(&now)
                .fetch_one(&pool),
            )
            .map_err(|e| TrackerError::Database(e.into()))?;
        Ok(Self::row_to_task(row))
    }

    fn get_task(&self, id: i64) -> Result<Option<Task>, TrackerError> {
        let pool = self.pool()?;
        let row = self
            .block_on(
                sqlx::query(
                    r#"SELECT id, session_id, name, status, note, created_at, closed_at
                       FROM tasks WHERE id = ?1"#,
                )
                .bind(id)
                .fetch_optional(&pool),
            )
            .map_err(|e| TrackerError::Database(e.into()))?;
        Ok(row.map(Self::row_to_task))
    }

    fn list_tasks(&self, session_id: i64) -> Result<Vec<Task>, TrackerError> {
        let pool = self.pool()?;
        let rows = self
            .block_on(
                sqlx::query(
                    r#"SELECT id, session_id, name, status, note, created_at, closed_at
                       FROM tasks WHERE session_id = ?1
                       ORDER BY created_at DESC"#,
                )
                .bind(session_id)
                .fetch_all(&pool),
            )
            .map_err(|e| TrackerError::Database(e.into()))?;
        Ok(rows.into_iter().map(Self::row_to_task).collect())
    }

    fn list_tasks_by_status(
        &self,
        session_id: i64,
        status: TaskStatus,
    ) -> Result<Vec<Task>, TrackerError> {
        let pool = self.pool()?;
        let status_str = Self::task_status_to_str(status);
        let rows = self
            .block_on(
                sqlx::query(
                    r#"SELECT id, session_id, name, status, note, created_at, closed_at
                       FROM tasks WHERE session_id = ?1 AND status = ?2
                       ORDER BY created_at DESC"#,
                )
                .bind(session_id)
                .bind(status_str)
                .fetch_all(&pool),
            )
            .map_err(|e| TrackerError::Database(e.into()))?;
        Ok(rows.into_iter().map(Self::row_to_task).collect())
    }

    fn close_task(&self, id: i64, report: &str) -> Result<Option<Task>, TrackerError> {
        let pool = self.pool()?;
        self.block_on(async {
            let mut tx = pool
                .begin()
                .await
                .map_err(|e| TrackerError::Database(e.into()))?;

            let row = sqlx::query(
                r#"SELECT id, session_id, name, status, note, created_at, closed_at
                                       FROM tasks WHERE id = ?1"#,
            )
            .bind(id)
            .fetch_optional(&mut *tx)
            .await
            .map_err(|e| TrackerError::Database(e.into()))?;
            let Some(row) = row else {
                return Ok(None);
            };
            let task = Self::row_to_task(row);
            if task.status != TaskStatus::Open {
                return Err(TrackerError::TaskNotOpen {
                    id,
                    status: task.status.to_string(),
                });
            }

            let now = chrono::Utc::now().to_rfc3339();
            sqlx::query(
                r#"INSERT INTO task_reports (task_id, content, created_at)
                   VALUES (?1, ?2, ?3)"#,
            )
            .bind(id)
            .bind(report)
            .bind(&now)
            .execute(&mut *tx)
            .await
            .map_err(|e| TrackerError::Database(e.into()))?;

            let row = sqlx::query(
                r#"UPDATE tasks SET status = 'closed', closed_at = ?1 WHERE id = ?2
                   RETURNING id, session_id, name, status, note, created_at, closed_at"#,
            )
            .bind(&now)
            .bind(id)
            .fetch_one(&mut *tx)
            .await
            .map_err(|e| TrackerError::Database(e.into()))?;

            tx.commit()
                .await
                .map_err(|e| TrackerError::Database(e.into()))?;
            Ok(Some(Self::row_to_task(row)))
        })
    }

    fn invalidate_task(&self, id: i64, reason: Option<&str>) -> Result<Option<Task>, TrackerError> {
        let pool = self.pool()?;
        self.block_on(async {
            let mut tx = pool
                .begin()
                .await
                .map_err(|e| TrackerError::Database(e.into()))?;

            let row = sqlx::query(
                r#"SELECT id, session_id, name, status, note, created_at, closed_at
                                       FROM tasks WHERE id = ?1"#,
            )
            .bind(id)
            .fetch_optional(&mut *tx)
            .await
            .map_err(|e| TrackerError::Database(e.into()))?;
            let Some(row) = row else {
                return Ok(None);
            };
            let task = Self::row_to_task(row);
            if task.status != TaskStatus::Open {
                return Err(TrackerError::TaskNotOpen {
                    id,
                    status: task.status.to_string(),
                });
            }

            let row = sqlx::query(
                r#"UPDATE tasks SET status = 'invalid', note = ?1 WHERE id = ?2
                   RETURNING id, session_id, name, status, note, created_at, closed_at"#,
            )
            .bind(reason)
            .bind(id)
            .fetch_one(&mut *tx)
            .await
            .map_err(|e| TrackerError::Database(e.into()))?;

            tx.commit()
                .await
                .map_err(|e| TrackerError::Database(e.into()))?;
            Ok(Some(Self::row_to_task(row)))
        })
    }

    fn get_task_report(&self, task_id: i64) -> Result<Option<TaskReport>, TrackerError> {
        let pool = self.pool()?;
        let row = self
            .block_on(
                sqlx::query(
                    r#"SELECT id, task_id, content, created_at
                       FROM task_reports WHERE task_id = ?1"#,
                )
                .bind(task_id)
                .fetch_optional(&pool),
            )
            .map_err(|e| TrackerError::Database(e.into()))?;
        Ok(row.map(Self::row_to_task_report))
    }
}
