<div align="center">

# nxm-tracker

<!-- Licensed MIT — part of the nxm tooling. -->
[![License](https://img.shields.io/badge/license-MIT-blue.svg)](LICENSE)
[![Rust](https://img.shields.io/badge/Rust-1.75%2B-000000?logo=rust&logoColor=white)](https://www.rust-lang.org)
[![MCP](https://img.shields.io/badge/MCP-JSON--RPC%20stdio-5A4FCF)](#mcp-configuration)
[![Version](https://img.shields.io/badge/version-0.3.0-blue)](CHANGELOG.md)

**Tracker for long, multi-day work sessions — part of the NXM platform**

<br>

<picture>
  <source srcset="docs/assets/banner-tracker-v2.webp" type="image/webp">
  <img src="docs/assets/banner-tracker-v2.png" alt="Nexum-AI banner" width="1000" height="553">
</picture>

<br>

`nxm-tracker` keeps track of work sessions that span several days and are used to
generate and test code. After the design phase, integrated feature sessions are
created so the work can be managed correctly across days.

Configuration is provided by [`nxm-config`](https://github.com/dangranaz/nxm-config)
(develop branch). Structured logging is provided by
[`nxm-logging`](https://github.com/dangranaz/nxm-logging) (develop branch).

- **Repo:** session-tracking crate for the NXM platform
- **Configuration:** [`nxm-config`](https://github.com/dangranaz/nxm-config) —
  `ConfigPaths`, `GeneralConfig`, env overrides (`NXM_*`)
- **Logging:** [`nxm-logging`](https://github.com/dangranaz/nxm-logging) — JSON
  logging to `~/.nxmconfig/logs/nxm-tracker.log` + stderr
- **Current state:** scaffold only — no session-tracking logic, database or MCP tools
  implemented yet
- **Public API:** `product_name()`, `TrackerConfig`, `init_logging()`, `TrackerError`

</div>

## Purpose

`nxm-tracker` is being built to keep track of work sessions that span several days and
are used to generate and test code. Once the design phase is complete, integrated
feature sessions are created so the work can be managed correctly across days.

## Status

**Scaffold only.** No session-tracking logic, database or MCP tools are implemented yet.
Configuration is handled by `nxm-config`, logging by `nxm-logging`. The public API is
`product_name()`, `TrackerConfig`, `init_logging()`, and `TrackerError`.

## Configuration

`nxm-tracker` uses [`nxm-config`](https://github.com/dangranaz/nxm-config) for
configuration discovery, loading, and saving.

- Config root: `NXM_CONFIG_HOME` or `~/.nxmconfig`
- Config file: `config.toml`
- Environment overrides: `NXM_SERVER_HOST`, `NXM_SERVER_PORT`, `NXM_DATABASE_DRIVER`,
  `NXM_DATABASE_URL`, `NXM_CORS_ORIGINS`

Example:

```rust
use nxm_tracker::TrackerConfig;

let config = TrackerConfig::load()?;
println!("server port: {}", config.server_port());
println!("database url: {}", config.database_url());
```

## Logging

`nxm-tracker` uses [`nxm-logging`](https://github.com/dangranaz/nxm-logging) for
structured JSON logging.

- Log file: `~/.nxmconfig/logs/nxm-tracker.log` (daily rotation, 7 files retained)
- Also logs to stderr
- Log level: `NXM_MCP_LOG`, then `RUST_LOG`, then `"info"`

Example:

```rust
use nxm_tracker::init_logging;

init_logging()?;
```

## Development

```sh
cargo build
cargo test --all-targets
cargo clippy --all-targets --all-features -- -D warnings
cargo fmt --all -- --check
```

The mandatory engineering rules live in `.pi/rules/` and are summarised in `AGENTS.md`.

## License

Open source, **MIT** license ([LICENSE](LICENSE)).
