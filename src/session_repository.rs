//! SQLite implementation of [`SessionRepository`].
//!
//! Wraps an [`nxm_db::Db`] connection pool and persists sessions and feature
//! sessions in the SQLite tables created by [`crate::default_migration_set`].
//!
//! sqlx queries are asynchronous; this module runs them on a short-lived tokio
//! runtime so the [`SessionRepository`] trait stays synchronous.

use crate::database::Database;
use crate::error::TrackerError;
use crate::session::{
    FeatureSession, NewFeatureSession, NewSession, Session, SessionRepository, SessionStatus,
};
use nxm_db::Db;
use sqlx::Row;
use tokio::runtime::Runtime;

/// SQLite-backed session repository.
pub struct SqliteSessionRepository {
    db: Db,
    rt: Runtime,
}

impl SqliteSessionRepository {
    /// Builds a repository from an open [`crate::database::Database`].
    pub fn from_database(db: Database) -> Self {
        Self {
            db: db.inner,
            rt: Runtime::new().expect("tokio runtime should start"),
        }
    }

    /// Returns the underlying [`nxm_db::Db`] handle.
    pub fn db(&self) -> &Db {
        &self.db
    }

    fn pool(&self) -> Result<sqlx::SqlitePool, TrackerError> {
        self.db.sqlite_pool().map_err(TrackerError::Database)
    }

    /// Runs an async sqlx future on the repository's tokio runtime.
    fn block_on<F: std::future::Future>(&self, future: F) -> F::Output {
        self.rt.block_on(future)
    }

    fn status_to_str(status: SessionStatus) -> &'static str {
        match status {
            SessionStatus::Open => "open",
            SessionStatus::Paused => "paused",
            SessionStatus::Closed => "closed",
        }
    }

    fn row_to_session(row: sqlx::sqlite::SqliteRow) -> Session {
        Session {
            id: row.try_get::<i64, _>("id").ok(),
            name: row.try_get::<String, _>("name").unwrap_or_default(),
            started_at: row.try_get::<String, _>("started_at").unwrap_or_default(),
            ended_at: row.try_get::<Option<String>, _>("ended_at").ok().flatten(),
            status: SessionStatus::try_from(
                row.try_get::<String, _>("status")
                    .unwrap_or_default()
                    .as_str(),
            )
            .unwrap_or(SessionStatus::Open),
            design_phase_complete: row
                .try_get::<i32, _>("design_phase_complete")
                .map(|v| v != 0)
                .unwrap_or(false),
        }
    }

    fn row_to_feature_session(row: sqlx::sqlite::SqliteRow) -> FeatureSession {
        FeatureSession {
            id: row.try_get::<i64, _>("id").ok(),
            session_id: row.try_get::<i64, _>("session_id").unwrap_or(0),
            name: row.try_get::<String, _>("name").unwrap_or_default(),
            description: row
                .try_get::<Option<String>, _>("description")
                .ok()
                .flatten(),
            status: row.try_get::<String, _>("status").unwrap_or_default(),
            created_at: row.try_get::<String, _>("created_at").unwrap_or_default(),
        }
    }
}

impl SessionRepository for SqliteSessionRepository {
    fn create_session(&self, new: &NewSession) -> Result<Session, TrackerError> {
        let pool = self.pool()?;
        let row = self
            .block_on(
                sqlx::query(
                    r#"INSERT INTO sessions (name, started_at, status, design_phase_complete)
                   VALUES (?1, ?2, 'open', 0)
                   RETURNING id, name, started_at, ended_at, status, design_phase_complete"#,
                )
                .bind(&new.name)
                .bind(&new.started_at)
                .fetch_one(&pool),
            )
            .map_err(|e| TrackerError::Database(e.into()))?;
        Ok(Self::row_to_session(row))
    }

    fn get_session(&self, id: i64) -> Result<Option<Session>, TrackerError> {
        let pool = self.pool()?;
        let row = self
            .block_on(
                sqlx::query(
                    r#"SELECT id, name, started_at, ended_at, status, design_phase_complete
                   FROM sessions WHERE id = ?1"#,
                )
                .bind(id)
                .fetch_optional(&pool),
            )
            .map_err(|e| TrackerError::Database(e.into()))?;
        Ok(row.map(Self::row_to_session))
    }

    fn list_sessions(&self) -> Result<Vec<Session>, TrackerError> {
        let pool = self.pool()?;
        let rows = self
            .block_on(
                sqlx::query(
                    r#"SELECT id, name, started_at, ended_at, status, design_phase_complete
                   FROM sessions ORDER BY started_at DESC"#,
                )
                .fetch_all(&pool),
            )
            .map_err(|e| TrackerError::Database(e.into()))?;
        Ok(rows.into_iter().map(Self::row_to_session).collect())
    }

    fn set_session_status(&self, id: i64, status: SessionStatus) -> Result<bool, TrackerError> {
        let pool = self.pool()?;
        let status_str = Self::status_to_str(status);
        let result = self
            .block_on(
                sqlx::query(r#"UPDATE sessions SET status = ?1 WHERE id = ?2"#)
                    .bind(status_str)
                    .bind(id)
                    .execute(&pool),
            )
            .map_err(|e| TrackerError::Database(e.into()))?;
        Ok(result.rows_affected() > 0)
    }

    fn close_session(&self, id: i64) -> Result<bool, TrackerError> {
        let pool = self.pool()?;
        let now = chrono::Utc::now().to_rfc3339();
        let result = self
            .block_on(
                sqlx::query(
                    r#"UPDATE sessions SET status = 'closed', ended_at = ?1 WHERE id = ?2"#,
                )
                .bind(&now)
                .bind(id)
                .execute(&pool),
            )
            .map_err(|e| TrackerError::Database(e.into()))?;
        Ok(result.rows_affected() > 0)
    }

    fn set_design_phase_complete(&self, id: i64, complete: bool) -> Result<bool, TrackerError> {
        let pool = self.pool()?;
        let value: i32 = if complete { 1 } else { 0 };
        let result = self
            .block_on(
                sqlx::query(r#"UPDATE sessions SET design_phase_complete = ?1 WHERE id = ?2"#)
                    .bind(value)
                    .bind(id)
                    .execute(&pool),
            )
            .map_err(|e| TrackerError::Database(e.into()))?;
        Ok(result.rows_affected() > 0)
    }

    fn create_feature_session(
        &self,
        new: &NewFeatureSession,
    ) -> Result<FeatureSession, TrackerError> {
        let pool = self.pool()?;
        let now = chrono::Utc::now().to_rfc3339();
        let row = self.block_on(
            sqlx::query(
                r#"INSERT INTO feature_sessions (session_id, name, description, status, created_at)
                   VALUES (?1, ?2, ?3, 'open', ?4)
                   RETURNING id, session_id, name, description, status, created_at"#,
            )
            .bind(new.session_id)
            .bind(&new.name)
            .bind(&new.description)
            .bind(&now)
            .fetch_one(&pool),
        ).map_err(|e| TrackerError::Database(e.into()))?;
        Ok(Self::row_to_feature_session(row))
    }

    fn list_feature_sessions(&self, session_id: i64) -> Result<Vec<FeatureSession>, TrackerError> {
        let pool = self.pool()?;
        let rows = self
            .block_on(
                sqlx::query(
                    r#"SELECT id, session_id, name, description, status, created_at
                   FROM feature_sessions WHERE session_id = ?1
                   ORDER BY created_at DESC"#,
                )
                .bind(session_id)
                .fetch_all(&pool),
            )
            .map_err(|e| TrackerError::Database(e.into()))?;
        Ok(rows.into_iter().map(Self::row_to_feature_session).collect())
    }
}
