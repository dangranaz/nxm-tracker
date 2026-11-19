# Context — nxm-tracker

Project: tracker for long, multi-day work sessions and integrated feature sessions created after the design phase.
Location: `~/AiProjects/nexum/mcpServers/nxm-tracker`
Language: Rust (edition 2021)
Rules: canonical in `~/AiProjects/nexum/docs/configRust/`, mirrored in `.pi/rules/`

## Current state

- Minimal standalone crate: `Cargo.toml` with the standard build profiles, a library
  (`src/lib.rs`) and a binary (`src/main.rs`).
- The only public API is `product_name() -> &'static str`, documented with an example and
  structured `tracing` logging. The binary prints the product name and version.
- Smoke tests live in `tests/lib_test.rs` and verify the canonical binary name.

## Next increment

Implement the session model: long-running work sessions spanning multiple days, their
state, and the integrated feature sessions generated after the design phase. Track
tests separately in `tests/` and keep every production file under 300 lines.
