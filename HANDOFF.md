# Handoff — `nxm-tracker`

Stato del progetto e prossimi passi. Questo documento è una traccia permanente
del lavoro svolto e di quanto resta da implementare.

---

## Stato attuale

| Campo | Valore |
|-------|--------|
| **Crate** | `nxm_tracker` |
| **Versione** | `0.5.0` |
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

---

## Pubblic API (v0.5.0)

- `product_name() -> &'static str`
- `TrackerConfig` — `load()`, `load_or_create_from()`, `load_from()`, `save()`, `save_to()`, `server_port()`, `database_url()`, `database_driver()`
- `init_logging() -> Result<(), TrackerError>`
- `Database` — `open()`, `open_from()`, `migrate()`, `handle()`
- `default_migration_set() -> Result<MigrationSet, TrackerError>` (tabella `sessions`)
- `TrackerError` — wrappa `Config`, `Logging`, `Database`

---

## Test (9/9 passati)

| File | Test |
|------|------|
| `tests/lib_test.rs` | `product_name_is_the_canonical_binary_name`, `product_name_matches_the_cargo_package_name` |
| `tests/config_test.rs` | `tracker_config_loads_with_defaults_from_temp_root`, `tracker_config_save_and_load_roundtrip`, `tracker_config_custom_values_persist` |
| `tests/logging_test.rs` | `init_logging_first_wins` |
| `tests/database_test.rs` | `database_opens_in_temp_directory`, `database_migration_set_applies`, `default_migration_set_has_one_migration` |

---

## Qualità gate

- `cargo build` ✅
- `cargo test --all-targets` ✅ (9/9)
- `cargo clippy --all-targets -- -D warnings` ✅ (eseguito con `CARGO_TARGET_DIR=/tmp` per evitare esaurimento metadati btrfs)
- `cargo fmt --all -- --check` ✅

---

## Architettura documentata

- Diagramma Archify generato:
  `.archify/architecture-nxm-tracker-20251120-1/nxm-tracker-architecture.html`
  (gate tutti passati: validate, deliver, check, browser-check)
- **Nota:** il diagramma non include ancora `nxm-db`

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
│   ├── error.rs             # TrackerError (Config + Logging + Database)
│   ├── lib.rs               # Esporta API pubblica
│   ├── logging.rs           # init_logging() (nxm-logging wrapper)
│   └── main.rs              # Entry point: logging → config → db → migration
├── tests/
│   ├── config_test.rs
│   ├── database_test.rs
│   ├── lib_test.rs
│   └── logging_test.rs
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
2. **Diagramma Archify** — va rigenerato per includere `nxm-db` (attualmente mostra solo `nxm-config` e `nxm-logging`).
3. **LICENSE** — non esiste un file `LICENSE` (senza estensione), solo `LICENSE.md`. Il badge nel README punta a `LICENSE.md`.

---

## Cosa resta da fare (next steps)

### Priorità alta

1. **Implementare il session model** — long-running work session, stati (`open`, `closed`, `paused`), integrated feature sessions create dopo la design phase. È il core business del crate (ancora solo scaffold).
2. **Rigenerare diagramma Archify** con `nxm-db` incluso (e possibilmente aggiungere diagrammi `sequence` per il flusso di avvio e `lifecycle` per gli stati delle sessioni).
3. **Popolare `docs/`** — documentazione API, architettura, esempi d'uso.

### Priorità media

4. **Implementare MCP tools** — il crate è un MCP server, ma al momento non espone ancora tool MCP.
5. **Aggiungere repository pattern** sopra `nxm-db` per le operazioni CRUD sulle sessioni.
6. **Verificare il file `LICENSE`** (senza estensione) se richiesto da qualche tool di terze parti.

---

## Flusso operativo da seguire

1. Modifiche in `~/AiProjects/nexum/mcpServers/nxm-tracker`
2. `cargo test --all-targets`, `cargo clippy --all-targets -- -D warnings`, `cargo fmt --all -- --check`
3. `git add -A`, `git commit`, `git tag vX.Y.Z`
4. `git push origin master` + `git push origin vX.Y.Z`
5. `git push gitlab master` + `git push gitlab vX.Y.Z`
6. `gh release create vX.Y.Z ...` (su GitHub)
7. `/home/daniele/AiProjects/nexum/scripts/publicNxmTracker` (sincronizza `dangranaz`)
