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
/// Creates the `sessions` table used to track long, multi-day work sessions
/// and the `feature_sessions` table for integrated feature sessions created
/// after the design phase.
///
/// Migration v3–v15 add the lifecycle catalog (from `nxm-mcp-tracker-ee`):
/// `lifecycle_state` (v3-v4), `lifecycle_transition` (v5-v6), `session` (v7),
/// `audit_log` (v9), `task`/`task_revision`/`task_audit` (v10-v12),
/// `session_handoff` (v13), `slug_counter` (v14), `prompt` (v15),
/// `retention_log` (v16).
///
/// Each migration is a single SQL statement because `nxm-db` applies them
/// via `sqlx::query().execute()` which does not support multiple statements.
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
            "create_lifecycle_state",
            r#"CREATE TABLE IF NOT EXISTS lifecycle_state (
                name          TEXT    PRIMARY KEY,
                description   TEXT    NOT NULL,
                is_visible    INTEGER NOT NULL,
                is_archivable INTEGER NOT NULL,
                is_reopenable INTEGER NOT NULL,
                sort_order    INTEGER NOT NULL
            );"#,
        ),
        nxm_db::migrate::Migration::new(
            4,
            "seed_lifecycle_state",
            r#"INSERT INTO lifecycle_state (name, description, is_visible, is_archivable, is_reopenable, sort_order) VALUES
                ('active',     'In progress / resumable — the live work queue.',                    1, 0, 1, 0),
                ('onhold',     'Parked for a collision between sessions on the same project.',       1, 0, 1, 1),
                ('abandoned',  'Dormant in place; not archived; hidden from the default listing.',   0, 0, 1, 2),
                ('done',       'Finished with value; archived immediately; not reopenable.',         0, 1, 0, 3),
                ('superseded', 'Replaced by a successor; archived but reopenable.',                  0, 1, 1, 4);"#,
        ),
        nxm_db::migrate::Migration::new(
            5,
            "create_lifecycle_transition",
            r#"CREATE TABLE IF NOT EXISTS lifecycle_transition (
                from_state  TEXT NOT NULL,
                to_state    TEXT NOT NULL,
                trigger     TEXT NOT NULL,
                description TEXT NOT NULL,
                PRIMARY KEY (from_state, to_state, trigger)
            );"#,
        ),
        nxm_db::migrate::Migration::new(
            6,
            "seed_lifecycle_transition",
            r#"INSERT INTO lifecycle_transition (from_state, to_state, trigger, description) VALUES
                ('active',     'onhold',     'set_status', 'Park for a collision between sessions on the same project.'),
                ('active',     'abandoned',  'set_status', 'Abandon: dormant in place, not archived.'),
                ('active',     'done',       'set_status', 'Finished with value: archived, not reopenable.'),
                ('active',     'superseded', 'set_status', 'Replaced by a successor: archived, reopenable.'),
                ('onhold',     'active',     'reopen',     'Resume from the parked state.'),
                ('abandoned',  'active',     'reopen',     'Resume in place.'),
                ('superseded', 'active',     'reopen',     'Resume with a physical return of files from archived/.');"#,
        ),
        nxm_db::migrate::Migration::new(
            7,
            "create_session",
            r#"CREATE TABLE IF NOT EXISTS session (
                id           TEXT    PRIMARY KEY,
                status       TEXT    NOT NULL,
                created_at   TEXT    NOT NULL,
                updated_at   TEXT    NOT NULL,
                consumed_at  TEXT,
                consumed_by  TEXT
            );"#,
        ),
        nxm_db::migrate::Migration::new(
            8,
            "create_audit_log",
            r#"CREATE TABLE IF NOT EXISTS audit_log (
                id          INTEGER PRIMARY KEY AUTOINCREMENT,
                session_id  TEXT    NOT NULL,
                action      TEXT    NOT NULL,
                from_state  TEXT,
                to_state    TEXT    NOT NULL,
                user        TEXT    NOT NULL,
                auth_method TEXT    NOT NULL,
                ai_tool     TEXT    NOT NULL,
                ai_model    TEXT,
                at          TEXT    NOT NULL
            );"#,
        ),
        nxm_db::migrate::Migration::new(
            9,
            "create_task_table",
            r#"CREATE TABLE IF NOT EXISTS task (
                session_id  TEXT    NOT NULL,
                task_id     INTEGER NOT NULL,
                created_at  TEXT    NOT NULL,
                PRIMARY KEY (session_id, task_id)
            );"#,
        ),
        nxm_db::migrate::Migration::new(
            10,
            "create_task_revision_table",
            r#"CREATE TABLE IF NOT EXISTS task_revision (
                id              INTEGER PRIMARY KEY AUTOINCREMENT,
                session_id      TEXT    NOT NULL,
                task_id         INTEGER NOT NULL,
                version         INTEGER NOT NULL,
                status          TEXT    NOT NULL,
                title           TEXT    NOT NULL,
                body            TEXT,
                exec_order      INTEGER NOT NULL,
                result          TEXT,
                spawned_task_id INTEGER,
                blocked_by      INTEGER,
                at              TEXT    NOT NULL,
                FOREIGN KEY (session_id, task_id) REFERENCES task(session_id, task_id),
                UNIQUE (session_id, task_id, version)
            );"#,
        ),
        nxm_db::migrate::Migration::new(
            11,
            "create_task_audit_table",
            r#"CREATE TABLE IF NOT EXISTS task_audit (
                id           INTEGER PRIMARY KEY AUTOINCREMENT,
                session_id   TEXT    NOT NULL,
                task_id      INTEGER NOT NULL,
                action       TEXT    NOT NULL,
                from_status  TEXT,
                to_status    TEXT,
                user         TEXT    NOT NULL,
                auth_method  TEXT    NOT NULL,
                ai_tool      TEXT    NOT NULL,
                ai_model     TEXT,
                at           TEXT    NOT NULL,
                FOREIGN KEY (session_id, task_id) REFERENCES task(session_id, task_id)
            );"#,
        ),
        nxm_db::migrate::Migration::new(
            12,
            "create_session_handoff",
            r#"CREATE TABLE IF NOT EXISTS session_handoff (
                id                 INTEGER PRIMARY KEY AUTOINCREMENT,
                session_id         TEXT    NOT NULL,
                version            INTEGER NOT NULL,
                goal_next          TEXT,
                decisions          TEXT,
                next_steps         TEXT,
                failed_approaches  TEXT,
                relevant_files     TEXT,
                context_usage_pct  INTEGER,
                at                 TEXT    NOT NULL,
                UNIQUE (session_id, version)
            );"#,
        ),
        nxm_db::migrate::Migration::new(
            13,
            "create_slug_counter",
            r#"CREATE TABLE IF NOT EXISTS slug_counter (
                slug    TEXT    PRIMARY KEY,
                next_nn INTEGER NOT NULL
            );"#,
        ),
        nxm_db::migrate::Migration::new(
            14,
            "create_prompt",
            r#"CREATE TABLE IF NOT EXISTS prompt (
                session_id TEXT    NOT NULL,
                task_no    TEXT    NOT NULL,
                slug       TEXT    NOT NULL,
                lang       TEXT    NOT NULL,
                path       TEXT    NOT NULL,
                PRIMARY KEY (session_id, task_no, lang)
            );"#,
        ),
        nxm_db::migrate::Migration::new(
            15,
            "create_retention_log",
            r#"CREATE TABLE IF NOT EXISTS retention_log (
                id           INTEGER PRIMARY KEY AUTOINCREMENT,
                purged_count INTEGER NOT NULL,
                older_than   TEXT    NOT NULL,
                at           TEXT    NOT NULL
            );"#,
        ),
    ];
    MigrationSet::new(migrations).map_err(TrackerError::Database)
}
