# AGENTS.md — nxm-tracker

Behavioral guidelines for this crate. Merge with project-specific instructions as needed.

## Purpose

`nxm-tracker` keeps track of long, multi-day work sessions used to generate and test
code. After the design phase, integrated feature sessions are created so the work can
be managed correctly. The session model (Session, SessionStatus, FeatureSession,
SessionRepository, SqliteSessionRepository) and eight MCP tools
(create_session, get_session, list_sessions, update_session, close_session,
set_design_phase_complete, create_feature_session, list_feature_sessions) are
implemented and exposed via stdio.

## Mandatory engineering rules

These rules are mandatory and take precedence over implementation convenience. They are
mirrored from the canonical source `~/AiProjects/nexum/docs/configRust/` and kept locally
in `.pi/rules/`, as in the other MCP server crates:

- `.pi/rules/rust.md`
- `.pi/rules/architecture.md`
- `.pi/rules/testing.md`
- `.pi/rules/code-quality.md`
- `.pi/rules/security.md`
- `.pi/rules/releases.md`

Upstream counterparts (source of truth):

- `~/AiProjects/nexum/docs/configRust/AGENTS.md`
- `~/AiProjects/nexum/docs/configRust/rules/*.md`

If a task conflicts with these rules, follow the rules and explain the conflict before
proceeding. To add more rules later, create another file under `.pi/rules/` and list it
here — this list is explicit and must stay complete.

## Key reminders

- **Think Before Coding**: don't assume; if something is unclear, stop and ask.
- **Simplicity First**: minimum code that solves the problem. No speculative features.
- **Surgical Changes**: touch only what you must; match existing style.
- **Goal-Driven Execution**: define verifiable success criteria before implementing.
- **Testing**: every new function with business logic MUST have unit tests in `tests/`;
  run `cargo test --all-targets` after implementation.
- **Conventional Commits**: `<type>(<scope>): <description>`; scope names the affected
  crate when applicable.
- **Versioning + CHANGELOG**: bump the crate version in `Cargo.toml` and update
  `CHANGELOG.md` in the same commit for `feat`/`fix`/breaking changes.
- **Documentation**: public APIs need `///` doc comments with at least one example;
  update `CONTEXT.md` and `CHANGELOG.md` at end of session.

## 1. Think Before Coding

**Don't assume. Don't hide confusion. Surface tradeoffs.**

Before implementing:

- State your assumptions explicitly. If uncertain, ask.
- If multiple interpretations exist, present them — don't pick silently.
- If a simpler approach exists, say so. Push back when warranted.
- If something is unclear, stop. Name what's confusing. Ask.

## 2. Simplicity First

**Minimum code that solves the problem. Nothing speculative.**

- No features beyond what was asked.
- No abstractions for single-use code.
- No "flexibility" or "configurability" that wasn't requested.
- No error handling for impossible scenarios.
- If you write 200 lines and it could be 50, rewrite it.

Ask yourself: "Would a senior engineer say this is overcomplicated?" If yes, simplify.

## 3. Surgical Changes

**Touch only what you must. Clean up only your own mess.**

When editing existing code:

- Don't "improve" adjacent code, comments, or formatting.
- Don't refactor things that aren't broken.
- Match existing style, even if you'd do it differently.
- If you notice unrelated dead code, mention it — don't delete it.

When your changes create orphans:

- Remove imports/variables/functions that YOUR changes made unused.
- Don't remove pre-existing dead code unless asked.

The test: Every changed line should trace directly to the user's request.

## 4. Goal-Driven Execution

**Define success criteria. Loop until verified.**

Transform tasks into verifiable goals:

- "Add validation" → "Write tests for invalid inputs, then make them pass"
- "Fix the bug" → "Write a test that reproduces it, then make it pass"
- "Refactor X" → "Ensure tests pass before and after"

For multi-step tasks, state a brief plan:

```
1. [Step] → verify: [check]
2. [Step] → verify: [check]
3. [Step] → verify: [check]
```

Strong success criteria let you loop independently. Weak criteria ("make it work")
require constant clarification.

## 5. Implementation Requirements

- Always inspect the existing architecture before modifying code.
- Always plan significant changes before implementation.
- Always write and execute relevant tests.
- Never declare a task complete without verifying its implementation.
- Never silently ignore an engineering rule.

## Commands

- `cargo build` — compile
- `cargo test --all-targets` — run tests
- `cargo clippy --all-targets --all-features -- -D warnings` — lint
- `cargo fmt --all -- --check` — formatting check

## 6. Completion Report

At the end of each implementation task, report:

- Files created and modified.
- Architectural decisions.
- Tests created and executed.
- Test results.
- Any rule violations or unresolved issues.
