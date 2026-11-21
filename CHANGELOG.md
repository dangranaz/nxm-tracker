# Changelog for nxm-tracker

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
