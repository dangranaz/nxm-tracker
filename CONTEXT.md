# Context — nxm-tracker

Project: tracker for long, multi-day work sessions and integrated feature sessions created after the design phase.
Location: `~/AiProjects/nexum/mcpServers/nxm-tracker`
Language: Rust (edition 2021)
Rules: canonical in `~/AiProjects/nexum/docs/configRust/`, mirrored in `.pi/rules/`

## Current state

- Standalone crate with `nxm-config` (`../../shared/nxm-config`, v0.2.0) as a path dependency
  for configuration management.
- Public API:
  - `product_name() -> &'static str` — canonical product name
  - `TrackerConfig` — wrapper around `nxm_config::GeneralConfig` with `load()`,
    `load_or_create_from()`, `load_from()`, `save()`, `save_to()`, `server_port()`,
    `database_url()`, `database_driver()` accessors
  - `TrackerError` — wraps `nxm_config::ConfigError`
- Binary prints product name, version, server port, database URL, and driver.
- Tests in `tests/config_test.rs` (config integration) and `tests/lib_test.rs`
  (product name smoke tests).

## Next increment

Implement the session model: long-running work sessions spanning multiple days, their
state, and the integrated feature sessions generated after the design phase. Track
tests separately in `tests/` and keep every production file under 300 lines.
