<div align="center">

# nxm-tracker

<!-- Licensed PolyForm Noncommercial 1.0.0 — part of the nxm tooling. -->
[![License](https://img.shields.io/badge/license-PolyForm%20Noncommercial-blue.svg)](LICENSE.md)
[![Rust](https://img.shields.io/badge/Rust-1.75%2B-000000?logo=rust&logoColor=white)](https://www.rust-lang.org)
[![MCP](https://img.shields.io/badge/MCP-JSON--RPC%20stdio-5A4FCF)](#mcp-configuration)
[![Version](https://img.shields.io/badge/version-0.7.0-blue)](CHANGELOG.md)

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
- **Current state:** configuration, logging, database access, and session-tracking
  logic integrated. Eight MCP tools exposed via stdio:
  `create_session`, `get_session`, `list_sessions`, `update_session`,
  `close_session`, `set_design_phase_complete`, `create_feature_session`,
  `list_feature_sessions`.
- **Public API:** `product_name()`, `TrackerConfig`, `init_logging()`, `Database`,
  `default_migration_set()`, `TrackerError`, `Session`, `SessionStatus`,
  `FeatureSession`, `NewSession`, `NewFeatureSession`, `SessionRepository`,
  `SqliteSessionRepository`, plus eight MCP tool structs

</div>

## Purpose

`nxm-tracker` is being built to keep track of work sessions that span several days and
are used to generate and test code. Once the design phase is complete, integrated
feature sessions are created so the work can be managed correctly across days.

## Status

**Implemented (v0.7.0).** Session-tracking logic and eight MCP tools are implemented.
Configuration is handled by `nxm-config`, logging by `nxm-logging`, datasource access
by `nxm-db`, and MCP server shell by `nxm-mcp-server`.

Public API includes `product_name()`, `TrackerConfig`, `init_logging()`, `Database`,
`default_migration_set()`, `TrackerError`, `Session`, `SessionStatus`,
`FeatureSession`, `NewSession`, `NewFeatureSession`, `SessionRepository`,
`SqliteSessionRepository`, and MCP tool structs:
`CreateSessionTool`, `GetSessionTool`, `ListSessionsTool`, `UpdateSessionTool`,
`CloseSessionTool`, `SetDesignPhaseCompleteTool`, `CreateFeatureSessionTool`,
`ListFeatureSessionsTool`.

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
- Migrations: `create_sessions` (v1) and `create_feature_sessions` (v2) tables

Example:

```rust
use nxm_tracker::{Database, default_migration_set};

let db = Database::open()?;
let set = default_migration_set()?;
db.migrate(&set)?;
```

## MCP Tools

`nxm-tracker` exposes eight MCP tools via stdio (JSON-RPC), using the
[`nxm-mcp-server`](https://github.com/dangranaz/nxm-mcp-server) shell:

| Tool | Description |
|------|-------------|
| `create_session` | Create a new work session |
| `get_session` | Get a work session by ID |
| `list_sessions` | List all work sessions (newest first) |
| `update_session` | Update name, status, or design_phase_complete |
| `close_session` | Close a session (sets `ended_at`) |
| `set_design_phase_complete` | Toggle design phase complete flag |
| `create_feature_session` | Create a feature session linked to a work session |
| `list_feature_sessions` | List feature sessions for a work session |

Each tool accepts JSON arguments and returns JSON results with `isError` flag.

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

**PolyForm Noncommercial License 1.0.0** ([LICENSE.md](LICENSE.md)).

Free for noncommercial use. Commercial use requires a separate commercial license
from the copyright holder. See the full license text at
https://polyformproject.com/licenses/noncommercial/1.0.0.
