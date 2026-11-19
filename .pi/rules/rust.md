# Rust Development Rules

These rules are mandatory for all Rust development tasks.

## 1. File Size and Modularity

- Every Rust source file (`.rs`) MUST NOT exceed 300 lines.
- The 300-line limit includes imports, comments, blank lines, attributes and declarations.
- If a file approaches 300 lines, split it into smaller cohesive modules.
- Each module must have a single, well-defined responsibility.
- Avoid oversized `mod.rs` files.
- Never bypass this rule by moving code into unrelated modules.

## 2. Separation of Production Code and Tests

- Production code and tests MUST be stored separately.
- Do not write unit tests inside production `.rs` files.
- Do not use inline `#[cfg(test)] mod tests` blocks.
- Unit tests must reside in dedicated test files.
- Integration tests must reside in the `tests/` directory.
- Organize tests to mirror the production module structure where appropriate.

Example:

```
src/
├── lib.rs
├── services/
│   ├── mod.rs
│   └── user_service.rs
└── utils/
    └── validation.rs

tests/
├── unit/
│   ├── user_service_test.rs
│   └── validation_test.rs
└── integration/
    └── user_workflow_test.rs
```

**Cargo discovery caveat:** Cargo only auto-discovers test files placed directly in `tests/`. Files in subdirectories such as `tests/unit/` or `tests/integration/` are NOT compiled unless each one is registered as an explicit `[[test]]` target (with a `path`) in that crate's `Cargo.toml`. Never add a test file under a subdirectory without also registering it, otherwise it is silently skipped and never runs.

**Visibility caveat:** external test crates can only see `pub` items, not private or `pub(crate)` ones. When tests live outside the production file, plan module visibility deliberately so the behavior under test is reachable through the public API.

**Current state:** the existing flat test files (e.g. `tests/nxm_mcp_server_test.rs`) predate this rule and remain valid until they are moved. New tests must follow the layout above.

## 3. Mandatory Unit Testing

- Every new function containing business logic MUST have unit tests.
- Every modification to existing business logic MUST include corresponding test updates.
- Tests MUST be executed after implementation.
- Use the project's existing testing framework.
- Prefer standard Rust testing facilities unless the project specifies otherwise.
- Test normal execution, edge cases, invalid inputs and error handling.
- Never remove or weaken tests to make an implementation pass.

## 4. Rust Quality Standards

- Follow idiomatic Rust conventions.
- Prefer ownership and borrowing over unnecessary cloning.
- Avoid unnecessary allocations.
- Use explicit error handling with `Result` and `Option`.
- Avoid `unwrap()` and `expect()` in production code unless explicitly justified.
- Prefer descriptive names over abbreviations.
- Minimize public API surface.
- Prefer small functions with clear responsibilities.
- Avoid unnecessary dependencies.

## 5. Build Profiles (Cargo.toml)

Cargo reads profiles **only** from the workspace `Cargo.toml`, not from global config. These profiles are the standard and must be replicated in every product's root `Cargo.toml`. Dual goal: blazing-fast compile/test cycle in development, runtime-optimized binary when a product is stable.

```toml
# --- DEV: fastest possible compile/test cycle ---
[profile.dev]
opt-level = 0                 # no optimization of our code → fast compile
debug = "line-tables-only"    # minimal debuginfo for stacktraces/profiler, faster link
incremental = true

# DEPENDENCIES optimized even in dev: compiled once (then in the sccache cache)
# but their code runs fast during our tests.
[profile.dev.package."*"]
opt-level = 2

# --- RELEASE: runtime-optimized binary for stable products ---
[profile.release]
opt-level = 3
lto = "thin"                  # cross-crate inlining, moderate build cost
codegen-units = 1             # more optimization (slower build, faster runtime)
strip = "symbols"

# --- PROFILING: release WITH symbols, for samply/flamegraph ---
[profile.profiling]
inherits = "release"
debug = true
strip = false

# --- BENCH: symbols for benchmark profiling ---
[profile.bench]
debug = 1
```

Usage rules:
- **dev** (default): all daily work (`check`/`test`/`watch`/`bacon`).
- **release**: `cargo build --release` only when the product is **stable** / shipping.
- **profiling**: `cargo build --profile profiling` + `samply record` when a realistic flamegraph is needed (optimized but with symbols).
- `panic = "abort"` in the release profile is allowed **only on binaries** and must be evaluated case by case (it changes panic semantics, disables some test harnesses): do not default it on libraries.

The compilation cache (sccache) and environment flags (e.g. `split-debuginfo` on macOS) live in the user-level `~/.cargo/config.toml` — NOT in the profiles.

## 6. Logging (tracing)

- Every public function must have structured logging (`tracing::info!`, `tracing::debug!`).
- Use per-functionality targets: `tracing::info!(target: "nexum::<module>::<feature>", ...)`.
- Complex operations: `ENTER` / `EXIT` logs with timing.
- Hot path: **no `format!` in logs** inside hot loops (per-message allocation). Use structured fields: `tracing::debug!(id = m.id(), "emitting")`, not `format!("... {}", ...)`.
- Logging utilities come from `nxm-shared`. Do not reinvent them.

## 7. Error Handling

- Never `unwrap()` or `expect()` in production code (tests only).
- App-level: `anyhow::Result` with `.context("what was happening when this failed")`.
- Library-level: `thiserror` with typed variants.
- Always provide context.

## 8. Mandatory Validation

After every implementation, run the following from each crate's root directory (this repository has no Cargo workspace at the top level, so the commands run per crate — for example `cd nxm_mcp_server`):

```sh
cargo fmt --all -- --check
cargo clippy --all-targets --all-features -- -D warnings
cargo test --all-targets
```

Also verify that no Rust source file exceeds 300 lines.

## 9. Completion Criteria / Definition of Done

An implementation is complete only when:

- All Rust source files comply with the 300-line limit.
- Production code and tests are separated.
- Required unit tests exist.
- All relevant tests pass.
- Formatting checks pass.
- Clippy checks pass.
- Standard build profiles present in the project's `Cargo.toml` (§5).
- Structured logging on public functions, no `format!` in hot paths.
- Integration tests if the change is cross-module/cross-crate.
- criterion/divan benchmark if the code is on the hot path.
- No `unwrap()`/`expect()` in production.
- `///` docs with an example on public APIs.
- Docs updated in both languages (ITA + ENG) where applicable.
- `CONTEXT.md` + `CHANGELOG.md` updated.
- Commit in Conventional Commits, one logical change.

If any validation cannot be executed, report it explicitly.

Never claim successful validation without actually running the commands.
