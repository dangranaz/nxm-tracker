# nxm-tracker Scaffold Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Creare lo scheletro Rust minimale di `nxm-tracker` con AGENTS.md, regole e documentazione, pronto a ricevere la logica di tracciamento delle sessioni.

**Architecture:** Crate Rust standalone (empty `[workspace]`) con library + binary. Le regole engineering vengono copiate dalla fonte canonica `~/AiProjects/nexum/docs/configRust/` in `.pi/rules/` e richiamate da un `AGENTS.md` adattato, come negli altri server MCP del platform.

**Tech Stack:** Rust edition 2021, Cargo, `tracing` 0.1, test with `cargo test`.

**Spec:** `docs/superpowers/specs/2026-11-19-nxm-tracker-design.md`

## Global Constraints

- Nome pacchetto: `nxm_tracker`; nome binario: `nxm-tracker`; versione iniziale: `0.1.0`.
- Posizione: `~/AiProjects/nexum/mcpServers/nxm-tracker`.
- Ogni file sorgente Rust ≤300 righe; test fuori da `src/`, in `tests/`.
- Profili Cargo obbligatori: `dev`, `dev.package."*"`, `release`, `profiling`, `bench` (valori copiati da `rules/rust.md` §5).
- API pubbliche: `///` con almeno un esempio; funzioni pubbliche con logging `tracing` structurato e target `nexum::<module>::<feature>`.
- Niente `unwrap()`/`expect()` nel codice di produzione.
- Conventional Commits; `CHANGELOG.md` e `CONTEXT.md` aggiornati.

## Review Focus

- `cargo test --doc` non fallisce: l'esempio nella doc-comment di `product_name()` deve compilare con la dipendenza `tracing` dichiarata → test: l'esempio è incluso in `src/lib.rs` e verificato da `cargo test --all-targets`.
- Il binario non è automaticamente scoperto se manca `[[bin]]` → test: `cargo run --bin nxm-tracker` stampa il nome prodotto.
- Le regole copiate devono essere quelle canonica complete (non la versione abbreviata di un altro crate) → verifica: diff tra `.pi/rules/rust.md` e la fonte canonica.
- I test in `tests/` devono essere scoperti da Cargo → verifica: `cargo test --all-targets` esegue 2 test.

---

### Task 1: Manifest Cargo e sorgenti minimi

**Files:**
- Create: `Cargo.toml`
- Create: `src/lib.rs`
- Create: `src/main.rs`
- Test: `tests/lib_test.rs`

**Interfaces:**
- Produces: `nxm_tracker::product_name() -> &'static str` che ritorna `"nxm-tracker"`.

- [ ] **Step 1: Write the failing test**

```rust
//! Smoke tests for the `nxm-tracker` scaffold.

#[test]
fn product_name_is_the_canonical_binary_name() {
    assert_eq!(nxm_tracker::product_name(), "nxm-tracker");
}

#[test]
fn product_name_matches_the_cargo_package_name() {
    assert_eq!(
        nxm_tracker::product_name(),
        env!("CARGO_PKG_NAME").replace('_', "-")
    );
}
```

- [ ] **Step 2: Run test to verify it fails**

Run: `cargo test --all-targets`
Expected: FAIL — `unresolved import nxm_tracker` / manifest missing.

- [ ] **Step 3: Implement manifest and sources**

`Cargo.toml`: `[package]` name `nxm_tracker`, version `0.1.0`, edition `2021`, license `MIT`, description sulla tracciatura di sessioni di lavoro lunghe e sessioni di feature integrate post-design; `[dependencies]` con solo `tracing = "0.1"`; `[workspace]` vuoto; `[[bin]]` name `nxm-tracker` path `src/main.rs`; profili `dev`, `dev.package."*"`, `release`, `profiling`, `bench` con i valori esatti di `rules/rust.md` §5.

`src/lib.rs`: documentazione del crate; `pub fn product_name() -> &'static str` con `///` + esempio doctest e `tracing::debug!(target: "nexum::tracker::api", ...)`.

`src/main.rs`: stampa `product_name()` e `env!("CARGO_PKG_VERSION")`.

- [ ] **Step 4: Run tests to verify they pass**

Run: `cargo test --all-targets`
Expected: PASS — 2 test in `tests/lib_test.rs`; poi `cargo test --doc` PASS.

- [ ] **Step 5: Commit**

```bash
git add Cargo.toml src/lib.rs src/main.rs tests/lib_test.rs
git commit -m "feat(tracker): initial project skeleton"
```

### Task 2: AGENTS.md, regole e documentazione

**Files:**
- Create: `AGENTS.md`
- Create: `.pi/rules/{rust,architecture,testing,code-quality,security,releases}.md`
- Create: `README.md`, `CONTEXT.md`, `CHANGELOG.md`, `.gitignore`

- [ ] **Step 1: Copy the canonical rules**

Copiare i 6 file da `~/AiProjects/nexum/docs/configRust/rules/` in `.pi/rules/` senza modifiche. Verificare con `diff` che siano identici.

- [ ] **Step 2: Write AGENTS.md**

Intestazione `# AGENTS.md — nxm-tracker`, puntamento alle regole canonica upstream e alla copia `.pi/rules/`, key reminders (Think Before Coding, Simplicity First, Surgical Changes, Goal-Driven Execution, Testing, Conventional Commits, Versioning + CHANGELOG, Documentation), sezioni comportamentali 1–7 dell'AGENTS.md canonica, comandi `cargo build` / `cargo test --all-targets` / `cargo clippy --all-targets --all-features -- -D warnings` / `cargo fmt --all -- --check`.

- [ ] **Step 3: Write docs**

`README.md`: scopo del progetto (sessioni di lavoro lunghe multi-giorno; sessioni di feature integrate dopo il design), stato scaffold, comandi. `CONTEXT.md`: progetto, locazione, linguaggio, regole, current state, next increment. `CHANGELOG.md`: sezione `## 0.1.0 - 2026-11-19` con `- feat(tracker): initial project skeleton`. `.gitignore`: `/target` e `Cargo.lock`.

- [ ] **Step 4: Verify documentation and rules**

Run: `for f in .pi/rules/*.md; do diff -q "$f" "$HOME/AiProjects/nexum/docs/configRust/rules/$(basename "$f")"; done`
Expected: nessuna differenza. Verificare che tutti i file esistano.

- [ ] **Step 5: Commit**

```bash
git add AGENTS.md .pi/rules README.md CONTEXT.md CHANGELOG.md .gitignore
git commit -m "docs(tracker): add agent rules and project documentation"
```
