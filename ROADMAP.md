# Roadmap — `nxm-tracker`

Visione: tracciare sessioni di lavoro lunghe, multi-giorno, e le sessioni di feature
integrate create dopo la fase di design, esposte come strumenti MCP per l'integrazione
con altri prodotti NXM.

---

## Release completate

### v0.1.0 — 2026-11-19 — Scaffolding iniziale
- Crate Rust edition 2021 standalone
- `product_name()`, `lib.rs`, `main.rs`, test smoke
- AGENTS.md + regole locali da `docs/configRust/`

### v0.2.0 — 2026-11-20 — Integrazione `nxm-config`
- `TrackerConfig` wrapper su `nxm_config::GeneralConfig`
- `load()`, `load_or_create_from()`, `load_from()`, `save()`, `save_to()`
- Accessori: `server_port()`, `database_url()`, `database_driver()`
- `TrackerError` con `Config`

### v0.3.0 — 2026-11-20 — Integrazione `nxm-logging`
- `init_logging()` wrapper su `nxm_logging::init_product_logging`
- Log JSON su `~/.nxmconfig/logs/nxm-tracker.log` + stderr
- `TrackerError` con `Logging`

### v0.4.0 — 2026-11-20 — Integrazione `nxm-db`
- `Database` wrapper su `nxm_db::Db`
- `open()`, `open_from()`, `migrate()`, `handle()`
- `default_migration_set()` con tabella `sessions` (v1)
- `TrackerError` con `Database`

### v0.5.0 — 2026-11-20 — Licenza PolyForm Noncommercial 1.0.0
- Cambio licenza da MIT a PolyForm Noncommercial 1.0.0
- `LICENSE.md`, aggiornamento README e Cargo.toml

### v0.6.0 — 2026-11-21 — Session model (core business)
- `Session` con `id`, `name`, `started_at`, `ended_at`, `status`, `design_phase_complete`
- `SessionStatus` enum: `Open`, `Paused`, `Closed`
- `FeatureSession` per integrated feature session post-design
- `SessionRepository` trait + `SqliteSessionRepository` su pool `nxm-db`
- Migration v2: `create_feature_sessions` con FK su `sessions`
- `TrackerError::InvalidSessionStatus`
- 11 test in `tests/session_test.rs`
- Dipendenze: `sqlx`, `chrono`, `serde`, `tokio`

### v0.7.0 — 2026-11-21 — MCP tools
- 8 tool MCP: `create_session`, `get_session`, `list_sessions`, `update_session`,
  `close_session`, `set_design_phase_complete`, `create_feature_session`,
  `list_feature_sessions`
- `main.rs` riscritto su shell `nxm-mcp-server`: logging → config → db → migration → registra tool → serve stdio
- `SessionStatus` serializzato come stringhe minuscole (`open`, `paused`, `closed`)
- `SessionRepository::update_session()` per update parziali
- 11 test in `tests/mcp_tools_test.rs`
- Dipendenze: `nxm-mcp-server`, `serde_json`

### v0.7.2 — 2026-11-21 — Security policy
- `docs/security/SECURITY.md` with supported versions, vulnerability reporting process, disclosure timeline
- Documented security best practices: input validation, secret handling, dependency auditing, no `unsafe` code, data protection
- Documented MCP tools security behavior

### v0.7.1 — 2026-11-21 — Documentazione stati sessione
- `docs/api/api.md` — riferimento API pubblica con esempi
- `docs/architecture/architecture.md` — struttura moduli, flusso avvio, schema DB, tabella stati sessione
- `README.md` — versione 0.7.1, stato attuale, public API, sezione MCP Tools, sezione Session States
- `AGENTS.md` — scopo aggiornato (non più scaffold)
- Diagramma Archify rigenerato includendo `nxm-db` (validate 9/9, deliver 9/9)
- `HANDOFF-2026-11-21.md` — handoff per prossima sessione

---

## Stato attuale (v0.7.1)

Tutti i quality gate verdi: `build` ✅, `test --all-targets` ✅ (31/31),
`clippy -D warnings` ✅, `fmt --check` ✅.

---

## Roadmap futura

### v0.8.0 — Esempi d'uso e usabilità (priorità alta)

| Feature | Descrizione | Criterio di successo |
|---------|-------------|---------------------|
| Esempi d'uso esecutibili | Blocchi di codice compilabili in `docs/api/api.md` e `docs/architecture/architecture.md` oppure cartella `docs/examples/` | Ogni esempio compila e illustra un flusso completo: crea → leggi → aggiorna → chiudi sessione; crea feature session |
| Esempi di chiamata MCP | Esempi di payload JSON per ogni tool MCP | Tutti e 8 gli tool hanno almeno un esempio request/response |

### v0.9.0 — Espansione repository e robustezza (priorità media)

| Feature | Descrizione | Criterio di successo |
|---------|-------------|---------------------|
| Query aggiuntive su repository | Search per nome, filtro per stato (`open`/`paused`/`closed`), paginazione su `list_sessions` | Test di integrazione per ogni nuova query |
| File `LICENSE` senza estensione | Creare `LICENSE` (copia di `LICENSE.md`) per compatibilità con tool terzi | File presente, badge README aggiornato se necessario |
| Altri backend (opzionale) | Valutare supporto oltre SQLite se richiesto da prodotti NXM | Decisione documentata in `docs/architecture/` |

### v1.0.0 — Production ready (lungo termine)

| Feature | Descrizione | Criterio di successo |
|---------|-------------|---------------------|
| Validazione input MCP | Validazione argomenti degli tool (nomi non vuoti, date RFC3339 valide, ID esistenti) | Errori chiari con `isError=true` per input invalidi |
| Test end-to-end | Test su trasporto stdio reale (non solo chiamate dirette a `execute`) | Un test MCP end-to-end che passa |
| Documentazione utente | Guida all'uso dei tool MCP da client esterni | `docs/user-guide.md` o sezione in `README.md` |
| Gestione errori migliorata | Distinzione errori di business vs errori di sistema | Tutti i tool restituiscono `isError=true` con messaggio descrittivo |

### Post-v1.0 — Integrazione piattaforma NXM

| Feature | Descrizione |
|---------|-------------|
| Integrazione con `nxm-conductor` / `nxm-cortexdb` | Condivisione stato sessione tra prodotti NXM |
| Notifiche / webhook | Notifica cambio stato sessione a listener esterni |
| Export/import sessioni | Esportazione sessioni in formato JSON/CSV per backup o migrazione |
| Metriche / dashboard | Conteggio sessioni aperte, chiuse, in pausa per periodo |

---

## Suggerimenti su priorità

1. **Inizia da v0.8.0 (esempi d'uso)** — è priorità alta, completano la documentazione già avviata e rendono il prodotto immediatamente utilizzabile da sviluppatori esterni.
2. **Non aggiungere funzionalità speculative** — regola del progetto: solo ciò che è richiesto. Se nessun prodotto NXM richiede ancora altri backend o notifiche, rimandali.
3. **Mantieni i file sotto 300 righe** — regola ingegneristica del progetto; se `mcp_tools.rs` o `session_repository.rs` crescono troppo, valuta di suddividerli.
4. **Ogni nuova funzione ha test in `tests/`** — regola obbligatoria; per v0.8.0 gli esempi dovrebbero essere accompagnati da test di integrazione se contengono logica.

---

## Criteri di successo per ogni milestone

- `cargo build` ✅
- `cargo test --all-targets` ✅
- `cargo clippy --all-targets --all-features -- -D warnings` ✅
- `cargo fmt --all -- --check` ✅
- Documentazione aggiornata (`CONTEXT.md`, `CHANGELOG.md`, `HANDOFF.md`, `README.md`) nello stesso commit del feature
- Tag e release su entrambi i remote (GitHub + GitLab)
