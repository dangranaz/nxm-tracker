# Changelog for nxm-tracker

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
