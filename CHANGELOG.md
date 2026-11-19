# Changelog for nxm-tracker

## 0.2.0 - 2026-11-20

- feat(tracker): integrate nxm-config for configuration management
  - add `TrackerConfig` wrapper around `nxm_config::GeneralConfig`
  - add `TrackerConfig::load()`, `load_or_create_from()`, `load_from()`, `save()`, `save_to()`
  - expose `server_port()`, `database_url()`, `database_driver()` accessors
  - add `TrackerError` wrapping `nxm_config::ConfigError`
  - add config integration tests in `tests/config_test.rs`

## 0.1.0 - 2026-11-19

- feat(tracker): initial project skeleton
