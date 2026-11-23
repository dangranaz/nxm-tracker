# Handoff — `nxm-tracker`

Stato del progetto e prossimi passi. Questo documento è una traccia permanente
del lavoro svolto e di quanto resta da implementare.

---

## Stato attuale

| Campo | Valore |
|-------|--------|
| **Crate** | `nxm_tracker` |
| **Versione** | `1.1.0` |
| **Ultimo commit** | `2c676ee` — feat(tracker): handoff, activity plan, and execution tasks per session (v1.1.0) |
| **Ultimo tag** | `v1.1.0` su entrambi i remote |
| **Ultima release GitHub** | https://github.com/dangranaz/nxm-tracker/releases/tag/v1.1.0 |
| **Posizione locale** | `~/AiProjects/nexum/mcpServers/nxm-tracker` |
| **Copia pubblicazione** | `~/AiProjects/dangranaz/nxm-tracker` |
| **Remote GitHub** | `dangranaz/nxm-tracker` (public) |
| **Remote GitLab** | `nxmAiDevelop/nxmMcpServers/nxm-tracker` (public, utente `granata.phoenix`) |
| **Licenza** | PolyForm Noncommercial 1.0.0 (`LICENSE.md`) |

---

## Dipendenze integrate (path dependency, crate standalone)

| Crate | Versione | Ruolo |
|-------|----------|-------|
| `nxm-config` | `0.2.0` | Configurazione (`ConfigPaths`, `GeneralConfig`, env override `NXM_*`) |
| `nxm-logging` | `0.2.0` | Logging strutturato JSON (`init_product_logging`, file + stderr) |
| `nxm-db` | `0.2.0` | Datasource SQLite con pool + migration |
| `sqlx` | `0.9` | SQL query sul pool di `nxm-db` (sqlite, runtime-tokio) |
| `chrono` | `0.4` | Timestamp RFC3339 per sessioni e feature session |
| `serde` | `1` | Serializzazione (`derive`) per i modelli di dominio |
| `serde_json` | `1` | Argomenti e risultati dei tool MCP |
| `tokio` | `1` | Runtime per bloccare le future sqlx in contesto sincrono |
| `nxm-mcp-server` | `0.1.0` | Shell MCP: trasporto stdio, registro tool, dispatcher |

---

## Pubblic API (v1.1.0)

- `product_name() -> &'static str`
- `TrackerConfig` — `load()`, `load_or_create_from()`, `load_from()`, `save()`, `save_to()`, `server_port()`, `database_url()`, `database_driver()`
- `init_logging() -> Result<(), TrackerError>`
- `Database` — `open()`, `open_from()`, `migrate()`, `handle()`
- `default_migration_set() -> Result<MigrationSet, TrackerError>` (tabelle `sessions` v1 e `feature_sessions` v2)
- `TrackerError` — wrappa `Config`, `Logging`, `Database`, più `InvalidSessionStatus`
- `Session` — `id`, `name`, `started_at`, `ended_at`, `status`, `design_phase_complete`
- `SessionStatus` — `open`, `paused`, `closed` (serializzato come stringa minuscola)
- `FeatureSession` — `id`, `session_id`, `name`, `description`, `status`, `created_at`
- `NewSession`, `NewFeatureSession` — input per la creazione
- `SessionRepository` — trait per persistenza sessioni e feature session
- `SqliteSessionRepository` — implementazione SQLite sul pool di `nxm-db`;
  implementa anche `HandoffRepository` e `TaskRepository`, più
  `create_session_full(new, handoff, plan)` (transazione atomica)
- `Handoff` (1:1), `ActivityPlan` (1:N), `HandoffRepository`
  (`get_handoff`, `upsert_handoff`, `list_activity_plans`)
- `Task`, `TaskStatus` (`open`/`closed`/`invalid`), `TaskReport` (1:1),
  `NewTask`, `TaskRepository` (`create_task`, `get_task`, `list_tasks`,
  `list_tasks_by_status`, `close_task`, `invalidate_task`, `get_task_report`)
- `TrackerError::InvalidTaskStatus`, `TrackerError::TaskNotOpen`
- MCP tool (17): `create_session` (con `handoff`/`plan` opzionali), `get_session`,
  `list_sessions`, `update_session`, `close_session`, `set_design_phase_complete`,
  `create_feature_session`, `list_feature_sessions`, `get_handoff`,
  `update_handoff`, `list_activity_plans`, `create_task`, `list_tasks`,
  `get_task`, `close_task` (report obbligatorio), `invalidate_task`,
  `get_task_report`

---

## Test (86/86 passati, di cui 24 nuovi in v1.1.0)

| File | Test |
|------|------|
| `tests/lib_test.rs` | `product_name_is_the_canonical_binary_name`, `product_name_matches_the_cargo_package_name` |
| `tests/config_test.rs` | `tracker_config_loads_with_defaults_from_temp_root`, `tracker_config_save_and_load_roundtrip`, `tracker_config_custom_values_persist` |
| `tests/logging_test.rs` | `init_logging_first_wins` |
| `tests/database_test.rs` | `database_opens_in_temp_directory`, `database_migration_set_applies`, `default_migration_set_has_two_migrations` |
| `tests/session_test.rs` | `create_and_get_session`, `get_nonexistent_session_returns_none`, `list_sessions_returns_all_ordered`, `set_session_status_changes_status`, `set_session_status_nonexistent_returns_false`, `close_session_sets_ended_at_and_status`, `close_nonexistent_session_returns_false`, `set_design_phase_complete_toggles_flag`, `create_and_list_feature_sessions`, `list_feature_sessions_empty_for_session_without_features`, `feature_session_belongs_to_correct_session` |
| `tests/mcp_tools_test.rs` | 15 test sui tool sessione/feature session |
| `tests/handoff_test.rs` | 6 test: `create_session_full` con/senza handoff+plan, upsert handoff, get senza handoff, isolamento piano per sessione |
| `tests/task_test.rs` | 10 test: CRUD task, filtro per stato, close con report, close doppio → `TaskNotOpen`, invalidate con reason, stati terminali, `TaskStatus` roundtrip |
| `tests/tracking_tools_test.rs` | 8 test sui nuovi MCP tool handoff/piano/task incl. validazioni (report obbligatorio, task non open, session/task inesistenti) |

---

## Qualità gate

- `cargo build` ✅
- `cargo test --all-targets` ✅ (86/86)
- `cargo clippy --all-targets --all-features -- -D warnings` ✅ (eseguito con `CARGO_TARGET_DIR=/tmp` per evitare esaurimento metadati btrfs)
- `cargo fmt --all -- --check` ✅

---

## Architettura documentata

- Diagramma Archify generato (include `nxm-db`):
  `.archify/architecture-nxm-tracker-20251120-1/nxm-tracker-architecture.html`
  (validate ✅, deliver ✅ 9/9 artifact checks)
- Documentazione API: `docs/api/api.md`
- Documentazione architettura: `docs/architecture/architecture.md`

---

## Struttura progetto

```
nxm-tracker/
├── .archify/                # Diagrammi Archify (candidato + HTML renderizzato)
├── builds/                  # Artefatti di build (.gitkeep)
├── docs/
│   ├── architecture/        # architecture.md (struttura moduli, flusso avvio, DB schema, stati sessione)
│   ├── api/                 # api.md (riferimento API pubblica con esempi)
│   └── superpowers/         # piani e specifiche esistenti
├── scripts/
│   ├── build.sh             # Build release + copia in builds/
│   ├── trackerUp            # Avvia nxm-tracker MCP server in background
│   └── trackerDown          # Ferma nxm-tracker MCP server
├── src/
│   ├── config.rs             # TrackerConfig (nxm-config wrapper)
│   ├── database.rs           # Database (nxm-db wrapper + default_migration_set, migration v1–v6)
│   ├── error.rs              # TrackerError (Config + Logging + Database + InvalidSessionStatus + InvalidTaskStatus + TaskNotOpen)
│   ├── git_workspace.rs      # Git workspace helpers (is_git_repo, base_branch, create_integration_branch, sanitize_branch_name)
│   ├── handoff.rs            # Handoff, ActivityPlan, HandoffRepository trait
│   ├── handoff_repository.rs # impl HandoffRepository su SqliteSessionRepository (upsert via ON CONFLICT)
│   ├── lib.rs                # Esporta API pubblica
│   ├── logging.rs            # init_logging() (nxm-logging wrapper)
│   ├── main.rs               # Entry point: logging → config → db → migration → registra 17 tool MCP → serve
│   ├── mcp_tools.rs          # Tool MCP sessione + feature session (create_session con handoff/plan)
│   ├── session.rs            # Session model: Session, SessionStatus, FeatureSession, SessionRepository trait
│   ├── session_repository.rs # SqliteSessionRepository (nxm-db pool) + create_session_full (transazione)
│   ├── task.rs               # Task, TaskStatus, TaskReport, NewTask, TaskRepository trait
│   ├── task_repository.rs    # impl TaskRepository (close_task/invalidate_task in transazione)
│   └── tracking_tools.rs     # 9 MCP tool handoff/piano/task
├── tests/
│   ├── config_test.rs
│   ├── database_test.rs
│   ├── examples_test.rs          # Verifica esempi in docs/examples/ (4 test)
│   ├── git_workspace_test.rs     # Git workspace helpers (10 test)
│   ├── lib_test.rs
│   ├── logging_test.rs
│   ├── mcp_tools_test.rs         # MCP tools integration tests (15 test)
│   ├── scripts_test.rs           # trackerUp/trackerDown (3 test)
│   ├── session_test.rs           # Session model integration tests (14 test)
│   ├── stdio_test.rs             # Trasporto stdio end-to-end (7 test, 17 tool in tools/list)
│   ├── handoff_test.rs           # Handoff + piano di attività (6 test)
│   ├── task_test.rs              # Task + report (10 test)
│   └── tracking_tools_test.rs    # Tool MCP handoff/piano/task (8 test)
├── LICENSE               # PolyForm Noncommercial 1.0.0 (senza estensione)
├── LICENSE.md            # PolyForm Noncommercial 1.0.0
├── README.md             # Aggiornato con banner + licenza
├── CHANGELOG.md
├── CONTEXT.md
├── HANDOFF.md            # Questo documento
├── Cargo.toml
├── docs/
│   ├── architecture/     # architecture.md
│   ├── api/              # api.md
│   ├── examples/         # Esempi d'uso (6 file Rust + mcp_payloads.md)
│   ├── superpowers/      # piani e specifiche esistenti
│   └── user-guide.md     # Guida utente per i tool MCP (v1.0.0)
└── scripts/
    ├── build.sh          # Build release + copia in builds/
    ├── trackerUp         # Avvia nxm-tracker MCP server in background
    └── trackerDown       # Ferma nxm-tracker MCP server
```

---

## Script di pubblicazione

- `~/AiProjects/nexum/scripts/publicNxmTracker` — rsync `--inplace --delete --exclude='.git' --exclude='target'` da locale a `~/AiProjects/dangranaz/nxm-tracker`
- `~/AiProjects/nexum/scripts/publicNxmConfig` — uguale per `nxm-config` (in `dangranaz/nxm-config`)

---

## Problemi noti

1. **btrfs metadata space** — su questo host i metadati btrfs di `/home` si esauriscono facilmente durante compilazioni pesanti (specialmente `clippy` con `nxm-db` che trascina `sqlx`, `tokio`, `mongodb`). Soluzione temporanea: usare `CARGO_TARGET_DIR=/tmp/...` o `cargo clean` prima di operazioni git.
2. **AGENTS.md e .pi/** — non più tracciati da git (aggiunti a `.gitignore`). Sono documenti locali dell'agente, sincronizzati da `~/AiProjects/nexum/docs/configRust/`.

---

## Cosa resta da fare (next steps)

### Priorità alta

1. ✅ **Aggiungere esempi d'uso esecutibili** — completato in v0.8.0
   - `docs/examples/create_and_get_session.rs`
   - `docs/examples/update_session.rs`
   - `docs/examples/close_session.rs`
   - `docs/examples/feature_sessions.rs`
   - `docs/examples/full_workflow.rs`
   - `docs/examples/mcp_payloads.md`
   - `tests/examples_test.rs` (verifica che i file esistano e siano ben formati)

### Priorità media

2. ✅ **Espandere repository pattern** — completato in v0.9.0
   - `search_sessions_by_name` — ricerca per nome (case-insensitive)
   - `list_sessions_by_status` — filtro per stato (open/paused/closed)
   - `list_sessions_paginated` — paginazione offset/limit
   - Test di integrazione in `tests/session_test.rs`
3. ✅ **File `LICENSE` senza estensione** — completato in v0.9.0
   - Creato `LICENSE` come copia di `LICENSE.md`
4. ✅ **Integrazione git workspace per creazione sessione** — completato in v0.10.0
   - `src/git_workspace.rs` con `is_git_repo`, `base_branch`, `create_integration_branch`, `sanitize_branch_name`
   - Parametro opzionale `working_directory` su `create_session`
   - Creazione branch `integration_<short>` da `develop`/`master`/`main`
   - 10 test in `tests/git_workspace_test.rs`

### ✅ v1.0.0 completato (Production ready)

- Validazione input su tutti e 8 gli tool MCP
- Gestione errori con `isError=true` e messaggi descrittivi
- Test end-to-end su trasporto stdio reale
- Documentazione utente: `docs/user-guide.md`
- Bump versione a `1.0.0`

### ✅ v1.1.0 completato — handoff, piano ed execution task

- Migration v3–v6: `handoffs` (1:1), `activity_plans` (1:N), `tasks`, `task_reports` (1:1)
- `create_session` con `handoff` e `plan` opzionali, creazione atomica (`create_session_full`)
- 9 nuovi MCP tool (17 totali): handoff get/update, piano, task CRUD, `close_task`
  con report obbligatorio 1:1 (transazione), `invalidate_task` con reason
- Task: `name` = branch git; stati `open`/`closed`/`invalid` (closed/invalid terminali)
- 24 nuovi test; quality gate verdi (build, clippy `-D warnings`, fmt, test 86/86)
- Bump versione a `1.1.0` + CHANGELOG + docs (README, CONTEXT, HANDOFF, ROADMAP, api, architecture, user-guide)

### Prossimi passi (post-v1.0, opzionali)

Vedi `ROADMAP.md` per la roadmap completa (integrazione con `nxm-conductor`/`nxm-cortexdb`, notifiche/webhook, metriche/dashboard).

### Da fare quando lo schema diventa complesso (Enterprise Edition, solo branch `ee`)

**Export/import sessioni** — Esportazione sessioni in formato JSON/CSV per backup o migrazione.
Questa funzionalità è riservata all'edizione Enterprise e deve essere implementata solo quando lo schema DB diventerà così complesso da richiedere una conversione del vecchio schema nel nuovo (es. cambi di struttura delle tabelle, aggiunta di nuovi campi obbligatori, migrazione di dati esistenti). Non è prevista per l'edizione pubblica a meno che non si renda necessaria per compatibilità tra versioni dello schema.

---

## Flusso operativo da seguire

### Sviluppo (branch `develop`)
1. Modifiche in `~/AiProjects/nexum/mcpServers/nxm-tracker` su branch `develop`
2. `cargo test --all-targets`, `cargo clippy --all-targets -- -D warnings`, `cargo fmt --all -- --check`
3. `git add -A`, `git commit`, `git push origin develop` + `git push gitlab develop`

### Release (branch `master`)
1. `git checkout master`
2. `git merge develop --no-ff`
3. `git tag vX.Y.Z`
4. `git push origin master` + `git push origin vX.Y.Z`
5. `git push gitlab master` + `git push gitlab vX.Y.Z`
6. `gh release create vX.Y.Z ...` (su GitHub)
7. `/home/daniele/AiProjects/nexum/scripts/publicNxmTracker` (sincronizza `dangranaz`)
