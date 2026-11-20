<div align="center">

# nxm-tracker

<!-- Licensed MIT — part of the nxm tooling. -->
[![License](https://img.shields.io/badge/license-MIT-blue.svg)](LICENSE)
[![Rust](https://img.shields.io/badge/Rust-1.75%2B-000000?logo=rust&logoColor=white)](https://www.rust-lang.org)
[![MCP](https://img.shields.io/badge/MCP-JSON--RPC%20stdio-5A4FCF)](#mcp-configuration)
[![Version](https://img.shields.io/badge/version-0.4.0-blue)](CHANGELOG.md)

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
Datasource access is provided by [`nxm-db`](https://github.com/dangranaz/nxm-db)
(develop branch).

- **Repo:** session-tracking crate for the NXM platform
- **Configuration:** [`nxm-config`](https://github.com/dangranaz/nxm-config) —
  `ConfigPaths`, `GeneralConfig`, env overrides (`NXM_*`)
- **Logging:** [`nxm-logging`](https://github.com/dangranaz/nxm-logging) — JSON
  logging to `~/.nxmconfig/logs/nxm-tracker.log` + stderr
- **Database:** [`nxm-db`](https://github.com/dangranaz/nxm-db) — SQLite datasource
  with connection pooling and migrations
- **Current state:** scaffold — configuration, logging, and database access integrated;
  session-tracking logic not yet implemented
- **Public API:** `product_name()`, `TrackerConfig`, `init_logging()`, `Database`,
  `default_migration_set()`, `TrackerError`

</div>

## Purpose

`nxm-tracker` is being built to keep track of work sessions that span several days and
are used to generate and test code. Once the design phase is complete, integrated
feature sessions are created so the work can be managed correctly across days.

## Status

**Scaffold.** Session-tracking logic not yet implemented. Configuration is handled by
`nxm-config`, logging by `nxm-logging`, and datasource access by `nxm-db`. The public
API is `product_name()`, `TrackerConfig`, `init_logging()`, `Database`,
`default_migration_set()`, and `TrackerError`.

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

## Database

`nxm-tracker` uses [`nxm-db`](https://github.com/dangranaz/nxm-db) for datasource
access.

- Backend: SQLite (default)
- Default path: `~/.nxm/shared-db/data.db`
- Migrations: `create_sessions` table for session tracking

Example:

```rust
use nxm_tracker::{Database, default_migration_set};

let db = Database::open()?;
let set = default_migration_set()?;
db.migrate(&set)?;
```

## Project layout

- `docs/` — project documentation
- `scripts/` — build and release scripts
- `builds/` — build artifacts
- `src/` — Rust source code
- `tests/` — integration tests

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
