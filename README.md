<div align="center">

# nxm-tracker

<!-- Licensed PolyForm Noncommercial 1.0.0 — part of the nxm tooling. -->
[![License](https://img.shields.io/badge/license-PolyForm%20Noncommercial-blue.svg)](LICENSE.md)
[![Rust](https://img.shields.io/badge/Rust-1.75%2B-000000?logo=rust&logoColor=white)](https://www.rust-lang.org)
[![MCP](https://img.shields.io/badge/MCP-JSON--RPC%20stdio-5A4FCF)](#mcp-configuration)
[![Version](https://img.shields.io/badge/version-1.1.0-blue)](CHANGELOG.md)

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
  logic integrated. Seventeen MCP tools exposed via stdio:
  `create_session`, `get_session`, `list_sessions`, `update_session`,
  `close_session`, `set_design_phase_complete`, `create_feature_session`,
  `list_feature_sessions`, `get_handoff`, `update_handoff`, `list_activity_plans`,
  `create_task`, `list_tasks`, `get_task`, `close_task`, `invalidate_task`,
  `get_task_report`.
- **Public API:** `product_name()`, `TrackerConfig`, `init_logging()`, `Database`,
  `default_migration_set()`, `TrackerError`, `Session`, `SessionStatus`,
  `FeatureSession`, `NewSession`, `NewFeatureSession`, `SessionRepository`,
  `SqliteSessionRepository`, `Handoff`, `ActivityPlan`, `HandoffRepository`,
  `Task`, `TaskStatus`, `TaskReport`, `NewTask`, `TaskRepository`, plus seventeen
  MCP tool structs

</div>

## Purpose

`nxm-tracker` is being built to keep track of work sessions that span several days and
are used to generate and test code. Once the design phase is complete, integrated
feature sessions are created so the work can be managed correctly across days.

## Status

**Implemented (v1.1.0).** Session-tracking logic, handoffs/activity plans, execution
tasks, and seventeen MCP tools are implemented. Production ready.
Configuration is handled by `nxm-config`, logging by `nxm-logging`, datasource access
by `nxm-db`, and MCP server shell by `nxm-mcp-server`.

Public API includes `product_name()`, `TrackerConfig`, `init_logging()`, `Database`,
`default_migration_set()`, `TrackerError`, `Session`, `SessionStatus`,
`FeatureSession`, `NewSession`, `NewFeatureSession`, `SessionRepository`,
`SqliteSessionRepository`, `Handoff`, `ActivityPlan`, `HandoffRepository`,
`Task`, `TaskStatus`, `TaskReport`, `NewTask`, `TaskRepository`, and MCP tool structs:
`CreateSessionTool`, `GetSessionTool`, `ListSessionsTool`, `UpdateSessionTool`,
`CloseSessionTool`, `SetDesignPhaseCompleteTool`, `CreateFeatureSessionTool`,
`ListFeatureSessionsTool`, `GetHandoffTool`, `UpdateHandoffTool`,
`ListActivityPlansTool`, `CreateTaskTool`, `ListTasksTool`, `GetTaskTool`,
`CloseTaskTool`, `InvalidateTaskTool`, `GetTaskReportTool`.

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
| `get_handoff` | Get the 1:1 handoff of a work session |
| `update_handoff` | Create/replace the handoff of a work session (upsert) |
| `list_activity_plans` | List the activity-plan steps of a work session (ordered) |
| `create_task` | Create an execution task for a work session (name = git branch) |
| `list_tasks` | List tasks of a work session, optionally filtered by status |
| `get_task` | Get a task by ID |
| `close_task` | Close an `open` task — a non-empty final `report` is required (stored 1:1) |
| `invalidate_task` | Mark an `open` task as `invalid` (optional `reason` stored in `note`) |
| `get_task_report` | Get the final report attached to a task |

Each tool accepts JSON arguments and returns JSON results with `isError` flag.

`create_session` also accepts two optional arguments:

- `handoff` (string) — the handoff description, stored 1:1 with the session and
  updatable later via `update_handoff`
- `plan` (array of strings) — the ordered activity plan of the session

## Handoffs, activity plans, and tasks

Every work session can carry:

- **Handoff (1:1)** — free-form text describing the current handoff state. It is
  updated frequently via `update_handoff` (upsert on `session_id`).
- **Activity plan (1:N)** — ordered steps (`position`, `description`), created
  with the session via the `plan` argument of `create_session`.
- **Tasks (1:N)** — execution tasks created after the session with `create_task`.
  A task's `name` is also its git branch name (e.g. `task/setup-db`).

### Task states

| State | DB value | Description |
|-------|----------|-------------|
| `Open` | `open` | Task pending execution. Initial state of every new task. |
| `Closed` | `closed` | Task finished. Requires a mandatory final report (1:1). Terminal. |
| `Invalid` | `invalid` | Task no longer has value (e.g. superseded by another task's outcome). Terminal. |

Transitions: `Open → Closed` (via `close_task` with `report`), `Open → Invalid`
(via `invalidate_task`, optional `reason`). `Closed` and `Invalid` are terminal.

## Session States

A work session can be in one of three states:

| Stato | Valore DB | Descrizione |
|-------|-----------|-------------|
| `Open` | `open` | Session actively being worked on. Initial state of every new session. |
| `Paused` | `paused` | Session temporarily suspended. Can be resumed later. |
| `Closed` | `closed` | Session finished and archived. Terminal state: cannot be reopened. |

Transitions: `Open → Paused → Open` (resume), `Open/Paused → Closed` (terminal).

See [`docs/architecture/architecture.md`](docs/architecture/architecture.md) for the
full state transition table and allowed operations.

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
