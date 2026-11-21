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
- Public API:
  - `product_name() -> &'static str` — canonical product name
  - `TrackerConfig` — wrapper around `nxm_config::GeneralConfig` with `load()`,
    `load_or_create_from()`, `load_from()`, `save()`, `save_to()`, `server_port()`,
    `database_url()`, `database_driver()` accessors
  - `init_logging() -> Result<(), TrackerError>` — initializes structured logging
    (JSON to `~/.nxmconfig/logs/nxm-tracker.log` + stderr)
  - `Database` — wrapper around `nxm_db::Db` with `open()`, `open_from()`,
    `migrate()`, `handle()` accessors
  - `default_migration_set()` — migration set with `create_sessions` (v1) and
    `create_feature_sessions` (v2) tables
  - `TrackerError` — wraps `nxm_config::ConfigError`, `nxm_logging::LoggingError`,
    `nxm_db::DbError`, plus `InvalidSessionStatus`
  - `Session` — work session (`id`, `name`, `started_at`, `ended_at`, `status`,
    `design_phase_complete`)
  - `SessionStatus` — `open`, `paused`, `closed` (serialized as lowercase strings)
  - `FeatureSession` — integrated feature session post-design phase
  - `NewSession`, `NewFeatureSession` — creation input structs
  - `SessionRepository` — trait for session/feature session persistence
  - `SqliteSessionRepository` — SQLite implementation on `nxm-db` pool
  - MCP tools: `CreateSessionTool`, `GetSessionTool`, `ListSessionsTool`,
    `UpdateSessionTool`, `CloseSessionTool`, `SetDesignPhaseCompleteTool`,
    `CreateFeatureSessionTool`, `ListFeatureSessionsTool`
- Binary initializes logging, loads configuration, opens database, applies migrations,
  registers MCP tools, and serves on stdio via `nxm-mcp-server` shell.
- Tests in `tests/config_test.rs` (config integration), `tests/logging_test.rs`
  (logging integration), `tests/database_test.rs` (database integration),
  `tests/lib_test.rs` (product name smoke tests), `tests/session_test.rs`
  (session model integration, 11 tests), and
  `tests/mcp_tools_test.rs` (MCP tools integration, 11 tests).

## Project layout

- `docs/` — project documentation (architecture, API, etc.)
- `scripts/` — build and release scripts
- `builds/` — build artifacts
- `src/` — Rust source code
- `tests/` — integration tests

## Next increment

Espandere repository pattern con ulteriori operazioni query se necessario; verificare
file `LICENSE` senza estensione; aggiungere esempi d'uso in `docs/examples/`.

Vedi `ROADMAP.md` per la roadmap completa con release completate, stato attuale e
pianificazione futura (v0.8.0, v0.9.0, v1.0.0, post-v1.0).