# nxm-tracker

Tracker for long, multi-day work sessions and the integrated feature sessions created
after the design phase.

Part of the NXM platform.

## Purpose

`nxm-tracker` is being built to keep track of work sessions that span several days and
are used to generate and test code. Once the design phase is complete, integrated
feature sessions are created so the work can be managed correctly across days.

## Status

**Scaffold only.** No session-tracking logic, database or MCP tools are implemented yet.
The current public API is `product_name()`, which returns the canonical product name.

## Development

```sh
cargo build
cargo test --all-targets
cargo clippy --all-targets --all-features -- -D warnings
cargo fmt --all -- --check
```

The mandatory engineering rules live in `.pi/rules/` and are summarised in `AGENTS.md`.
