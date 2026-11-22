# Context — nxm-tracker

Project: tracker for long, multi-day work sessions and integrated feature sessions created after the design phase.
Location: `~/AiProjects/nexum/mcpServers/nxm-tracker`
Language: Rust (edition 2021)
Rules: canonical in `~/AiProjects/nexum/docs/configRust/`, mirrored in `.pi/rules/`

## Current state

- Standalone crate with path dependencies:
  - `nxm-config` (`../../shared/nxm-config`, v0.2.0) for configuration management
  - `nxm-logging` (`../../shared/nxm-logging`, v0.2.0) for structured logging
  - `nxm-db` (`../../shared/nxm-db`, v0.2.0) for datasource access (SQLite)
  - `sqlx` (0.9, sqlite + runtime-tokio) for SQL queries on the `nxm-db` pool
  - `chrono` (0.4) for RFC3339 timestamps
  - `serde` (1, derive) for serialization
  - `serde_json` (1) for MCP tool arguments/results
  - `tokio` (1, rt) for blocking async sqlx calls in sync context
  - `nxm-mcp-server` (0.1.0, path dep) for MCP server shell (transport, tool registry, dispatch)
- Version: 1.1.0-EE (Enterprise Edition, branch `ee`)
- Documentazione dedicata: `docs/ee/` (presente solo su questo branch)
- Schema DB: 15 migrations (v1-v15)
  - v1: `create_sessions`, v2: `create_feature_sessions`
  - v3: `create_lifecycle_state`, v4: `seed_lifecycle_state`
  - v5: `create_lifecycle_transition`, v6: `seed_lifecycle_transition`
  - v7: `create_session` (TEXT id, status FK), v8: `create_audit_log`
  - v9: `create_task_table`, v10: `create_task_revision_table`, v11: `create_task_audit_table`
  - v12: `create_session_handoff`, v13: `create_slug_counter`
  - v14: `create_prompt`, v15: `create_retention_log`
- Public API:
  - `product_name() -> &'static str` — canonical product name
  - `TrackerConfig` — wrapper around `nxm_config::GeneralConfig` with `load()`,
    `load_or_create_from()`, `load_from()`, `save()`, `save_to()`, `server_port()`,
    `database_url()`, `database_driver()` accessors
  - `init_logging() -> Result<(), TrackerError>` — initializes structured logging
    (JSON to `~/.nxmconfig/logs/nxm-tracker.log` + stderr)
  - `Database` — wrapper around `nxm_db::Db` with `open()`, `open_from()`,
    `migrate()`, `handle()` accessors
  - `default_migration_set()` — migration set with 15 migrations
  - `TrackerError` — wraps `nxm_config::ConfigError`, `nxm_logging::LoggingError`,
    `nxm_db::DbError`, plus `InvalidSessionStatus`
  - `Session` — work session (`id`, `name`, `started_at`, `ended_at`, `status`,
    `design_phase_complete`)
  - `SessionStatus` — `open`, `paused`, `closed` (serialized as lowercase strings)
  - `FeatureSession` — integrated feature session post-design phase
  - `NewSession`, `NewFeatureSession` — creation input structs
  - `SessionRepository` — trait for session/feature session persistence
  - `SqliteSessionRepository` — SQLite implementation on `nxm-db` pool
  - `Session` (lifecycle) — TEXT id, status FK to `lifecycle_state`, consumed_at/consumed_by
  - `Status` — active/onhold/abandoned/done/superseded with predicates
  - `Transition`/`Trigger` — 7 valid transitions
  - `ActionContext` — user/ai_tool mandatory, ai_model optional, auth_method
  - `TaskStatus` — open/in_progress/done/superseded/blocked/cancelled
  - `LifecycleRepo` — trait (read-only: states, allowed_transitions)
  - `SqliteRepo`, `MemoryRepo` — adapters
  - MCP tools: `CreateSessionTool`, `GetSessionTool`, `ListSessionsTool`,
    `UpdateSessionTool`, `CloseSessionTool`, `SetDesignPhaseCompleteTool`,
    `CreateFeatureSessionTool`, `ListFeatureSessionsTool`
  - Write ops: `create_session`, `transition_session`, `update_session` (handoff),
    `register_prompt`, `mark_consumed`
  - Read ops: `get_session`, `list_sessions`, `find_resumable_session`,
    `get_current_handoff`, `list_handoff_revisions`
  - Task ops: `create_task`, `revise_task`, `set_task_status`, `close_task`,
    `get_task`, `list_session_tasks`, `list_open_tasks`
- Binary initializes logging, loads configuration, opens database, applies migrations,
  registers MCP tools, and serves on stdio via `nxm-mcp-server` shell.
- Tests in `tests/config_test.rs` (config integration), `tests/logging_test.rs`
  (logging integration), `tests/database_test.rs` (database integration),
  `tests/lib_test.rs` (product name smoke tests), `tests/session_test.rs`
  (session model integration, 14 tests), `tests/mcp_tools_test.rs`
  (MCP tools integration, 15 tests), `tests/stdio_test.rs` (stdio end-to-end,
  7 tests), `tests/scripts_test.rs` (trackerUp/trackerDown lifecycle, 3 tests),
  `tests/examples_test.rs` (example files verification, 4 tests),
  and `tests/git_workspace_test.rs` (git workspace helpers, 10 tests).
- `SECURITY.md` — security policy with supported versions, vulnerability reporting process, disclosure timeline, security best practices

## Project layout

- `docs/` — project documentation (architecture, API, user guide, etc.)
- `scripts/` — build and release scripts
  - `build.sh` — build release + copia in builds/
  - `trackerUp` — avvia nxm-tracker MCP server in background
  - `trackerDown` — ferma nxm-tracker MCP server
- `builds/` — build artifacts
- `src/` — Rust source code
  - `config.rs`, `database.rs`, `domain/` (status, transition, action_context, task_status),
    `error.rs`, `git_workspace.rs`, `lib.rs`, `logging.rs`, `main.rs`,
    `mcp_tools.rs`, `reads/` (session, handoff), `repo/` (sqlite, memory),
    `session.rs`, `session_repository.rs`, `writes/` (create, transition, error,
    prompt, consume, handoff)
- `tests/` — integration tests

## Next increment

Esposizione dei nuovi tool MCP (`mark_consumed`, `register_prompt`, `create_task`,
`revise_task`, `set_task_status`, `close_task`, `get_task`, `list_session_tasks`,
`list_open_tasks`, `list_lifecycle_states`, `list_lifecycle_transitions`) e aggiornamento
di `main.rs` per usare il nuovo repository e schema DB. Prossimo incremento: v1.2.0.

Vedi `ROADMAP.md` per la roadmap completa con release completate, stato attuale e
pianificazione futura (v0.8.0, v0.9.0, v1.0.0, v1.1.0, post-v1.0).
