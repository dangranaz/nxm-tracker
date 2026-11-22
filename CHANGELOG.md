# Changelog for nxm-tracker

## 0.10.0 - 2026-11-22
- feat(tracker): git workspace integration for session creation
  - add `src/git_workspace.rs` with `is_git_repo`, `base_branch`, `create_integration_branch`, `sanitize_branch_name`
  - add optional `working_directory` argument to `create_session` MCP tool
  - when `working_directory` is set, verify it is a git repo and create `integration_<short>` branch from `develop`/`master`/`main` before creating the session
  - add `tests/git_workspace_test.rs` (10 tests)

## 0.9.0 - 2026-11-22
- feat(tracker): expand repository query operations
  - add `SessionRepository::search_sessions_by_name` — case-insensitive name search
  - add `SessionRepository::list_sessions_by_status` — filter by status (open/paused/closed)
  - add `SessionRepository::list_sessions_paginated` — offset/limit pagination
  - add integration tests for all three new queries in `tests/session_test.rs`
- fix(tracker): add LICENSE file without extension
  - add `LICENSE` (copy of `LICENSE.md`) for third-party tool compatibility

## 0.8.1 - 2026-11-22
- feat(tracker): add trackerUp/trackerDown lifecycle scripts
  - add `scripts/trackerUp` — start nxm-tracker MCP server in background (named pipe + keepalive)
  - add `scripts/trackerDown` — stop nxm-tracker MCP server (SIGTERM → SIGKILL, PID file cleanup)
- feat(tracker): add stdio end-to-end tests
  - add `tests/stdio_test.rs` — initialize, tools/list, tools/call create_session, parse error
- feat(tracker): add trackerUp/trackerDown lifecycle tests
  - add `tests/scripts_test.rs` — start, stop, idempotency
- fix(nxm-mcp-server): flush stdio writer after each response
  - `transport.rs`: call `writer.flush()` after every `writeln!`
- fix(nxm-mcp-server): use `?` operator for notification check
  - `dispatch.rs`: replace `if return None` with `req.id.as_ref()?`

## 0.8.0 - 2026-11-22
- feat(tracker): add executable usage examples
  - add `docs/examples/create_and_get_session.rs` — create and read a session
  - add `docs/examples/update_session.rs` — update name/status/design_phase_complete
  - add `docs/examples/close_session.rs` — close a session and set ended_at
  - add `docs/examples/feature_sessions.rs` — create and list feature sessions
  - add `docs/examples/full_workflow.rs` — complete end-to-end flow
  - add `docs/examples/mcp_payloads.md` — JSON request/response payloads for all 8 MCP tools
  - add `tests/examples_test.rs` — verifies example files exist and are well-formed

## 0.7.2 - 2026-11-21

- docs(tracker): add SECURITY.md
  - add security policy with supported versions, vulnerability reporting process, disclosure timeline
  - document security best practices: input validation, secret handling, dependency auditing, no unsafe code, data protection
  - document MCP tools security behavior (isError envelopes, no stack trace leakage)
  - reference `.pi/rules/security.md` as canonical source

## 0.7.1 - 2026-11-21

- docs(tracker): document session states
  - add session state transition table to `docs/architecture/architecture.md`
    (`Open`, `Paused`, `Closed` with DB values, transitions, MCP tool mapping)
  - add allowed activities list for session lifecycle management
  - add Session States section to `README.md` with state table and transitions

## 0.7.0 - 2026-11-21

- feat(tracker): implement MCP tools for session management
  - add `create_session`, `get_session`, `list_sessions`, `update_session`, `close_session`
  - add `set_design_phase_complete`, `create_feature_session`, `list_feature_sessions`
  - add `SessionStatus` serde serialization as lowercase strings (`open`, `paused`, `closed`)
  - add `SessionRepository::update_session` method for partial session updates
  - add `nxm-mcp-server` dependency for MCP server shell integration
  - rewrite `main.rs` to use `nxm-mcp-server` shell: logging → config → db → migration → register tools → serve
  - add 11 MCP tool integration tests in `tests/mcp_tools_test.rs`
  - add `serde_json` dependency

## 0.6.0 - 2026-11-21

- feat(tracker): implement session model
  - add `Session` with `id`, `name`, `started_at`, `ended_at`, `status`, `design_phase_complete`
  - add `SessionStatus` enum (`open`, `paused`, `closed`) with DB string mapping
  - add `FeatureSession` for integrated feature sessions created after the design phase
  - add `SessionRepository` trait + `SqliteSessionRepository` implementation on `nxm-db`
  - add migration v2: `create_feature_sessions` table with FK to `sessions`
  - add `TrackerError::InvalidSessionStatus` for unknown status strings
  - add 11 integration tests in `tests/session_test.rs`
  - add `sqlx`, `chrono`, `serde`, `tokio` dependencies

## 0.5.0 - 2026-11-20

- fix(tracker): change license from MIT to PolyForm Noncommercial 1.0.0
  - add `LICENSE.md` with PolyForm Noncommercial License text
  - update README badge and license section
  - update Cargo.toml license field

## 0.4.0 - 2026-11-20

- feat(tracker): integrate nxm-db for datasource access
  - add `Database` wrapper around `nxm_db::Db`
  - add `Database::open()`, `open_from()`, `migrate()`, `handle()` accessors
  - add `default_migration_set()` with `create_sessions` migration
  - add `TrackerError::Database` wrapping `nxm_db::DbError`
  - binary opens database and applies migrations on startup
  - add database integration tests in `tests/database_test.rs`

## 0.3.0 - 2026-11-20

- feat(tracker): integrate nxm-logging for structured logging
  - add `init_logging()` wrapper around `nxm_logging::init_product_logging`
  - logs written in JSON to `~/.nxmconfig/logs/nxm-tracker.log` + stderr
  - add `TrackerError::Logging` wrapping `nxm_logging::LoggingError`
  - binary initializes logging before loading configuration
  - add logging integration tests in `tests/logging_test.rs`

## 0.2.0 - 2026-11-20

- feat(tracker): integrate nxm-config for configuration management
  - add `TrackerConfig` wrapper around `nxm_config::GeneralConfig`
  - add `TrackerConfig::load()`, `load_or_create_from()`, `load_from()`, `save()`, `save_to()`
  - expose `server_port()`, `database_url()`, `database_driver()` accessors
  - add `TrackerError` wrapping `nxm_config::ConfigError`
  - add config integration tests in `tests/config_test.rs`

## 0.1.0 - 2026-11-19

- feat(tracker): initial project skeleton
