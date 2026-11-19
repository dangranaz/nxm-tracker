# nxm-tracker — Design (approved 2026-11-19)

> Versione breve: si tratta di uno scaffold minimale, non di un documento di progetto completo.

## Intent and why

`nxm-tracker` è un nuovo crate Rust che terrà traccia di sessioni di lavoro lunghe,
della durata di più giorni, usate per generare e testare codice. Dopo la fase di
design vengono create sessioni di feature integrate che permettono di gestire il
lavoro correttamente. In questa fase viene creato solo lo scheletro: lo scopo è
registrato in `description`, README e CONTEXT, senza logica di dominio.

## Goals

- Scheletro Rust edition 2021 coerente con `nxm-conductor` e `nxm-cortexdb`.
- `AGENTS.md` + copia locale delle regole da `~/AiProjects/nexum/docs/configRust/`.
- Compila, passa i test e rispetta le regole ingegneristiche (profiles Cargo, test
  separati da `src/`, file ≤300 righe, Conventional Commits).

## Non-goals / anti-goals

- Nessun database, nessun tool MCP, nessuna integrazione con altri crate.
- Nessuna dipendenza applicativa: solo `tracing` per il logging strutturale obbligatorio.
- Nessuna logica di tracciamento delle sessioni in questo incremento.

## Constraints

- Regole canonica: `~/AiProjects/nexum/docs/configRust/AGENTS.md` e
  `~/AiProjects/nexum/docs/configRust/rules/{rust,architecture,testing,code-quality,security,releases}.md`.
- Posizione: `~/AiProjects/nexum/mcpServers/nxm-tracker` (accanto agli altri server MCP).
- Nome pacchetto `nxm_tracker`, nome binario `nxm-tracker`, versione `0.1.0`.

## Approaches

1. **Crate Rust in `mcpServers/nxm-tracker`** (scelto) — le regole sono Rust-specifiche
   e gli altri due server MCP sono in quella cartella: massima coerenza.
2. Solo cartella con AGENTS.md e regole, senza `Cargo.toml` — scartato: le regole
   presuppongono Rust e non produrrebbe uno scheletro utilizzabile.
3. Progetto Node/TypeScript — scartato: le regole non si applicano e il prefisso
   `nxm_*` è una convenzione Rust.

## Files

`Cargo.toml`, `src/lib.rs`, `src/main.rs`, `tests/lib_test.rs`, `README.md`,
`CONTEXT.md`, `CHANGELOG.md`, `.gitignore`, `AGENTS.md`, `.pi/rules/*.md` (6 file).
