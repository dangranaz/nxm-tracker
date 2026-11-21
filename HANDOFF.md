# Handoff — `nxm-tracker`

Stato del progetto e prossimi passi. Questo documento è una traccia permanente
del lavoro svolto e di quanto resta da implementare.

---

## Stato attuale

| Campo | Valore |
|-------|--------|
| **Crate** | `nxm_tracker` |
| **Versione** | `0.6.0` |
| **Posizione locale** | `~/AiProjects/nexum/mcpServers/nxm-tracker` |
| **Copia pubblicazione** | `~/AiProjects/dangranaz/nxm-tracker` |
| **Remote GitHub** | `dangranaz/nxm-tracker` (public) |
| **Remote GitLab** | `nxmAiDevelop/nxmMcpServers/nxm-tracker` (public, utente `granata.phoenix`) |
| **Licenza** | PolyForm Noncommercial 1.0.0 (`LICENSE.md`) |
| **Ultimo tag** | `v0.5.0` su entrambi i remote |
| **Ultima release GitHub** | https://github.com/dangranaz/nxm-tracker/releases/tag/v0.5.0 |

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
| `tokio` | `1` | Runtime per bloccare le future sqlx in contesto sincrono |

---

## Pubblic API (v0.6.0)

- `product_name() -> &'static str`
- `TrackerConfig` — `load()`, `load_or_create_from()`, `load_from()`, `save()`, `save_to()`, `server_port()`, `database_url()`, `database_driver()`
- `init_logging() -> Result<(), TrackerError>`
- `Database` — `open()`, `open_from()`, `migrate()`, `handle()`
- `default_migration_set() -> Result<MigrationSet, TrackerError>` (tabelle `sessions` v1 e `feature_sessions` v2)
- `TrackerError` — wrappa `Config`, `Logging`, `Database`, più `InvalidSessionStatus`
- `Session` — `id`, `name`, `started_at`, `ended_at`, `status`, `design_phase_complete`
- `SessionStatus` — `open`, `paused`, `closed`
- `FeatureSession` — `id`, `session_id`, `name`, `description`, `status`, `created_at`
- `NewSession`, `NewFeatureSession` — input per la creazione
- `SessionRepository` — trait per persistenza sessioni e feature session
- `SqliteSessionRepository` — implementazione SQLite sul pool di `nxm-db`

---

## Test (20/20 passati)

| File | Test |
|------|------|
| `tests/lib_test.rs` | `product_name_is_the_canonical_binary_name`, `product_name_matches_the_cargo_package_name` |
| `tests/config_test.rs` | `tracker_config_loads_with_defaults_from_temp_root`, `tracker_config_save_and_load_roundtrip`, `tracker_config_custom_values_persist` |
| `tests/logging_test.rs` | `init_logging_first_wins` |
| `tests/database_test.rs` | `database_opens_in_temp_directory`, `database_migration_set_applies`, `default_migration_set_has_two_migrations` |
| `tests/session_test.rs` | `create_and_get_session`, `get_nonexistent_session_returns_none`, `list_sessions_returns_all_ordered`, `set_session_status_changes_status`, `set_session_status_nonexistent_returns_false`, `close_session_sets_ended_at_and_status`, `close_nonexistent_session_returns_false`, `set_design_phase_complete_toggles_flag`, `create_and_list_feature_sessions`, `list_feature_sessions_empty_for_session_without_features`, `feature_session_belongs_to_correct_session` |

---

## Qualità gate

- `cargo build` ✅
- `cargo test --all-targets` ✅ (20/20)
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
│   ├── architecture/        # (vuoto, da popolare)
│   ├── api/                 # (vuoto, da popolare)
│   └── superpowers/         # piani e specifiche esistenti
├── scripts/
│   └── build.sh             # Build release + copia in builds/
├── src/
│   ├── config.rs            # TrackerConfig (nxm-config wrapper)
│   ├── database.rs          # Database (nxm-db wrapper + default_migration_set)
│   ├── error.rs             # TrackerError (Config + Logging + Database + InvalidSessionStatus)
│   ├── lib.rs               # Esporta API pubblica
│   ├── logging.rs           # init_logging() (nxm-logging wrapper)
│   ├── main.rs              # Entry point: logging → config → db → migration
│   ├── session.rs           # Session model: Session, SessionStatus, FeatureSession, SessionRepository trait
│   └── session_repository.rs# SqliteSessionRepository (nxm-db pool backend)
├── tests/
│   ├── config_test.rs
│   ├── database_test.rs
│   ├── lib_test.rs
│   ├── logging_test.rs
│   └── session_test.rs      # Session model integration tests (11 tests)
├── LICENSE.md               # PolyForm Noncommercial 1.0.0
├── README.md                # Aggiornato con banner + licenza
├── CHANGELOG.md
├── CONTEXT.md
├── HANDOFF.md               # Questo documento
└── Cargo.toml
```

---

## Script di pubblicazione

- `~/AiProjects/nexum/scripts/publicNxmTracker` — rsync `--inplace --delete --exclude='.git' --exclude='target'` da locale a `~/AiProjects/dangranaz/nxm-tracker`
- `~/AiProjects/nexum/scripts/publicNxmConfig` — uguale per `nxm-config` (in `dangranaz/nxm-config`)

---

## Problemi noti

1. **btrfs metadata space** — su questo host i metadati btrfs di `/home` si esauriscono facilmente durante compilazioni pesanti (specialmente `clippy` con `nxm-db` che trascina `sqlx`, `tokio`, `mongodb`). Soluzione temporanea: usare `CARGO_TARGET_DIR=/tmp/...` o `cargo clean` prima di operazioni git.
2. **LICENSE** — non esiste un file `LICENSE` (senza estensione), solo `LICENSE.md`. Il badge nel README punta a `LICENSE.md`.

---

## Cosa resta da fare (next steps)

### Priorità alta

1. **Popolare `docs/`** — documentazione API e architettura appena aggiunte (`docs/api/api.md`, `docs/architecture/architecture.md`); eventualmente aggiungere esempi d'uso.

### Priorità media

2. **Implementare MCP tools** — il crate è un MCP server, ma al momento non espone ancora tool MCP.
3. **Espandere repository pattern** — `SqliteSessionRepository` è già implementato; eventualmente aggiungere ulteriori operazioni query o altri backend.
4. **Verificare il file `LICENSE`** (senza estensione) se richiesto da qualche tool di terze parti.

---

## Flusso operativo da seguire

1. Modifiche in `~/AiProjects/nexum/mcpServers/nxm-tracker`
2. `cargo test --all-targets`, `cargo clippy --all-targets -- -D warnings`, `cargo fmt --all -- --check`
3. `git add -A`, `git commit`, `git tag vX.Y.Z`
4. `git push origin master` + `git push origin vX.Y.Z`
5. `git push gitlab master` + `git push gitlab vX.Y.Z`
6. `gh release create vX.Y.Z ...` (su GitHub)
7. `/home/daniele/AiProjects/nexum/scripts/publicNxmTracker` (sincronizza `dangranaz`)
