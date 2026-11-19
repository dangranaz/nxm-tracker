<div align="center">

# nxm-tracker

<!-- Licensed MIT — part of the nxm tooling. -->
[![License](https://img.shields.io/badge/license-MIT-blue.svg)](LICENSE)
[![Rust](https://img.shields.io/badge/Rust-1.75%2B-000000?logo=rust&logoColor=white)](https://www.rust-lang.org)
[![MCP](https://img.shields.io/badge/MCP-JSON--RPC%20stdio-5A4FCF)](#mcp-configuration)
[![Version](https://img.shields.io/badge/version-0.1.0-blue)](CHANGELOG.md)

**Tracker for long, multi-day work sessions — part of the NXM platform**

<br>

<picture>
  <source srcset="docs/assets/nexum-banner.webp?v=2" type="image/webp">
  <img src="docs/assets/nexum-banner.png?v=3" alt="Nexum-AI banner" width="1600" height="400">
</picture>

<br>

`nxm-tracker` keeps track of work sessions that span several days and are used to
generate and test code. After the design phase, integrated feature sessions are
created so the work can be managed correctly across days.

- **Repo:** session-tracking crate for the NXM platform
- **Current state:** scaffold only — no session-tracking logic, database or MCP tools
  implemented yet
- **Public API:** `product_name()` returns the canonical product name

</div>

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

## License

Open source, **MIT** license ([LICENSE](LICENSE)).
