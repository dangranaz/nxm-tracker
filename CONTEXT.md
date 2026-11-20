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
- Public API:
  - `product_name() -> &'static str` — canonical product name
  - `TrackerConfig` — wrapper around `nxm_config::GeneralConfig` with `load()`,
    `load_or_create_from()`, `load_from()`, `save()`, `save_to()`, `server_port()`,
    `database_url()`, `database_driver()` accessors
  - `init_logging() -> Result<(), TrackerError>` — initializes structured logging
    (JSON to `~/.nxmconfig/logs/nxm-tracker.log` + stderr)
  - `Database` — wrapper around `nxm_db::Db` with `open()`, `open_from()`,
    `migrate()`, `handle()` accessors
  - `default_migration_set()` — migration set with `create_sessions` table
  - `TrackerError` — wraps `nxm_config::ConfigError`, `nxm_logging::LoggingError`,
    and `nxm_db::DbError`
- Binary initializes logging, loads configuration, opens database, and applies migrations.
- Tests in `tests/config_test.rs` (config integration), `tests/logging_test.rs`
  (logging integration), `tests/database_test.rs` (database integration), and
  `tests/lib_test.rs` (product name smoke tests).

## Project layout

- `docs/` — project documentation (architecture, API, etc.)
- `scripts/` — build and release scripts
- `builds/` — build artifacts
- `src/` — Rust source code
- `tests/` — integration tests

## Next increment

Implement the session model: long-running work sessions spanning multiple days, their
state, and the integrated feature sessions generated after the design phase. Track
tests separately in `tests/` and keep every production file under 300 lines.