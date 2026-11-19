# Testing Rules

Scope: test strategy, coverage obligations, fixtures, test data and how tests are organized and executed.

## 1. Unit Tests — isolated behavior

- Test one module in isolation, including private interfaces.
- **Project rule**: NOT mixed inline with production code. To test private functions without inline tests, expose them as `pub(crate)` and test from a crate-internal `tests/` file, or keep logic in `lib.rs` and test via the public API.
- File: `tests/<module>.rs`, one per module under test.
- Every new function containing business logic MUST have unit tests.
- Every modification to existing business logic MUST include corresponding test updates.
- Tests MUST be executed after implementation.
- Use the project's existing testing framework.
- Test normal execution, edge cases, invalid inputs and error handling.
- Never remove or weaken tests to make an implementation pass.

**Cargo discovery caveat:** Cargo only auto-discovers test files placed directly in `tests/`. Files in subdirectories such as `tests/unit/` or `tests/integration/` are NOT compiled unless each one is registered as an explicit `[[test]]` target (with a `path`) in that crate's `Cargo.toml`. Never add a test file under a subdirectory without also registering it, otherwise it is silently skipped and never runs.

**Visibility caveat:** external test crates can only see `pub` items, not private or `pub(crate)` ones. When tests live outside the production file, plan module visibility deliberately so the behavior under test is reachable through the public API.

## 2. Integration Tests — modules together

- External to the crate, use only the public API, exercise multiple modules.
- File: `tests/integration/<feature>.rs`. Shared helpers in `tests/common/mod.rs`.
- Required for cross-module or cross-crate interactions.

## 3. Benchmarks — performance

- Tool: **criterion** (statistical rigor, confidence intervals) or **divan**.
- File: `benches/<area>.rs`. Enable debug symbols for profiling:
  ```toml
  [profile.bench]
  debug = 1
  ```
- Cloud-CI benchmarks are noisy → reference numbers are taken locally on fixed hardware. In CI use deterministic benchmarks (e.g. `iai`) only for regression detection.
- Tests requiring hardware (GPU/ANE) or a model are `#[ignore]`-gated with a comment explaining how to run them.
