//! SQLite implementation of [`HandoffRepository`].
//!
//! Persists the 1:1 session handoff and the activity-plan items in the
//! `handoffs` and `activity_plans` tables created by
//! [`crate::default_migration_set`].

use crate::error::TrackerError;
use crate::handoff::{ActivityPlan, Handoff, HandoffRepository};
use crate::session_repository::SqliteSessionRepository;
use sqlx::Row;

impl SqliteSessionRepository {
    pub(crate) fn row_to_handoff(row: sqlx::sqlite::SqliteRow) -> Handoff {
        Handoff {
            id: row.try_get::<i64, _>("id").ok(),
            session_id: row.try_get::<i64, _>("session_id").unwrap_or(0),
            content: row.try_get::<String, _>("content").unwrap_or_default(),
            updated_at: row.try_get::<String, _>("updated_at").unwrap_or_default(),
        }
    }

    pub(crate) fn row_to_activity_plan(row: sqlx::sqlite::SqliteRow) -> ActivityPlan {
        ActivityPlan {
            id: row.try_get::<i64, _>("id").ok(),
            session_id: row.try_get::<i64, _>("session_id").unwrap_or(0),
            position: row.try_get::<i64, _>("position").unwrap_or(0),
            description: row.try_get::<String, _>("description").unwrap_or_default(),
            created_at: row.try_get::<String, _>("created_at").unwrap_or_default(),
        }
    }
}

impl HandoffRepository for SqliteSessionRepository {
    fn get_handoff(&self, session_id: i64) -> Result<Option<Handoff>, TrackerError> {
        let pool = self.pool()?;
        let row = self
            .block_on(
                sqlx::query(
                    r#"SELECT id, session_id, content, updated_at
                       FROM handoffs WHERE session_id = ?1"#,
                )
                .bind(session_id)
                .fetch_optional(&pool),
            )
            .map_err(|e| TrackerError::Database(e.into()))?;
        Ok(row.map(Self::row_to_handoff))
    }

    fn upsert_handoff(&self, session_id: i64, content: &str) -> Result<Handoff, TrackerError> {
        let pool = self.pool()?;
        let now = chrono::Utc::now().to_rfc3339();
        let row = self
            .block_on(
                sqlx::query(
                    r#"INSERT INTO handoffs (session_id, content, updated_at)
                       VALUES (?1, ?2, ?3)
                       ON CONFLICT(session_id) DO UPDATE SET
                           content = excluded.content,
                           updated_at = excluded.updated_at
                       RETURNING id, session_id, content, updated_at"#,
                )
                .bind(session_id)
                .bind(content)
                .bind(&now)
                .fetch_one(&pool),
            )
            .map_err(|e| TrackerError::Database(e.into()))?;
        Ok(Self::row_to_handoff(row))
    }

    fn list_activity_plans(&self, session_id: i64) -> Result<Vec<ActivityPlan>, TrackerError> {
        let pool = self.pool()?;
        let rows = self
            .block_on(
                sqlx::query(
                    r#"SELECT id, session_id, position, description, created_at
                       FROM activity_plans WHERE session_id = ?1
                       ORDER BY position ASC"#,
                )
                .bind(session_id)
                .fetch_all(&pool),
            )
            .map_err(|e| TrackerError::Database(e.into()))?;
        Ok(rows.into_iter().map(Self::row_to_activity_plan).collect())
    }
}
