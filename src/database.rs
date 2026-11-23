//! Database access for `nxm-tracker`, built on [`nxm-db`].

use crate::error::TrackerError;
use nxm_db::{open, DbConfig, MigrationSet};

/// Wrapper around [`nxm_db::Db`] for `nxm-tracker`.
pub struct Database {
    pub inner: nxm_db::Db,
}

impl std::fmt::Debug for Database {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Database")
            .field("inner", &"nxm_db::Db")
            .finish()
    }
}

impl Database {
    /// Opens the default SQLite datasource (`~/.nxm/shared-db/data.db`).
    pub fn open() -> Result<Self, TrackerError> {
        let config = DbConfig::default();
        let inner = open(&config).map_err(TrackerError::Database)?;
        Ok(Self { inner })
    }

    /// Opens a datasource from an explicit [`DbConfig`].
    pub fn open_from(config: &DbConfig) -> Result<Self, TrackerError> {
        let inner = open(config).map_err(TrackerError::Database)?;
        Ok(Self { inner })
    }

    /// Returns the underlying [`nxm_db::Db`] handle.
    pub fn handle(&self) -> &nxm_db::Db {
        &self.inner
    }

    /// Runs a migration set against the database.
    pub fn migrate(&self, set: &MigrationSet) -> Result<(), TrackerError> {
        self.inner.migrate(set).map_err(TrackerError::Database)
    }
}

/// Default migration set for `nxm-tracker` session tracking.
///
/// Creates the `sessions` table used to track long, multi-day work sessions,
/// the `feature_sessions` table for integrated feature sessions created after
/// the design phase, the `handoffs` (1:1) and `activity_plans` (1:N) tables
/// attached to each session, and the `tasks` / `task_reports` tables for the
/// execution tasks created after session creation.
pub fn default_migration_set() -> Result<MigrationSet, TrackerError> {
    let migrations = vec![
        nxm_db::migrate::Migration::new(
            1,
            "create_sessions",
            r#"CREATE TABLE IF NOT EXISTS sessions (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                name TEXT NOT NULL,
                started_at TEXT NOT NULL,
                ended_at TEXT,
                status TEXT NOT NULL DEFAULT 'open',
                design_phase_complete INTEGER NOT NULL DEFAULT 0
            );"#,
        ),
        nxm_db::migrate::Migration::new(
            2,
            "create_feature_sessions",
            r#"CREATE TABLE IF NOT EXISTS feature_sessions (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                session_id INTEGER NOT NULL,
                name TEXT NOT NULL,
                description TEXT,
                status TEXT NOT NULL DEFAULT 'open',
                created_at TEXT NOT NULL,
                FOREIGN KEY (session_id) REFERENCES sessions(id) ON DELETE CASCADE
            );"#,
        ),
        nxm_db::migrate::Migration::new(
            3,
            "create_handoffs",
            r#"CREATE TABLE IF NOT EXISTS handoffs (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                session_id INTEGER NOT NULL UNIQUE,
                content TEXT NOT NULL,
                updated_at TEXT NOT NULL,
                FOREIGN KEY (session_id) REFERENCES sessions(id) ON DELETE CASCADE
            );"#,
        ),
        nxm_db::migrate::Migration::new(
            4,
            "create_activity_plans",
            r#"CREATE TABLE IF NOT EXISTS activity_plans (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                session_id INTEGER NOT NULL,
                position INTEGER NOT NULL,
                description TEXT NOT NULL,
                created_at TEXT NOT NULL,
                FOREIGN KEY (session_id) REFERENCES sessions(id) ON DELETE CASCADE
            );"#,
        ),
        nxm_db::migrate::Migration::new(
            5,
            "create_tasks",
            r#"CREATE TABLE IF NOT EXISTS tasks (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                session_id INTEGER NOT NULL,
                name TEXT NOT NULL,
                status TEXT NOT NULL DEFAULT 'open',
                note TEXT,
                created_at TEXT NOT NULL,
                closed_at TEXT,
                FOREIGN KEY (session_id) REFERENCES sessions(id) ON DELETE CASCADE
            );"#,
        ),
        nxm_db::migrate::Migration::new(
            6,
            "create_task_reports",
            r#"CREATE TABLE IF NOT EXISTS task_reports (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                task_id INTEGER NOT NULL UNIQUE,
                content TEXT NOT NULL,
                created_at TEXT NOT NULL,
                FOREIGN KEY (task_id) REFERENCES tasks(id) ON DELETE CASCADE
            );"#,
        ),
    ];
    MigrationSet::new(migrations).map_err(TrackerError::Database)
}
