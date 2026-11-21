# Architettura — `nxm-tracker`

## Panoramica

`nxm-tracker` è un crate Rust (edition 2021) standalone che traccia sessioni di lavoro
lunghe, multi-giorno, e le sessioni di feature integrate create dopo la fase di design.
Il crate è strutturato come un MCP server shell ma al momento non espone ancora tool
MCP (priorità media).

Il progetto segue i principi di **deep modules**: interfacce semplici, implementazione
nascosta, ogni file di produzione sotto le 300 righe, test separati in `tests/`.

## Struttura dei moduli

```
nxm-tracker crate
├── main.rs              # Entry point: logging → config → db → migration
├── lib.rs               # Esporta API pubblica
├── config.rs            # TrackerConfig (wrapper su nxm-config)
├── logging.rs           # init_logging() (wrapper su nxm-logging)
├── error.rs             # TrackerError (unione degli errori dei sottosistemi)
├── database.rs          # Database (wrapper su nxm-db) + default_migration_set()
├── session.rs           # Session, SessionStatus, FeatureSession, SessionRepository trait
└── session_repository.rs# SqliteSessionRepository (implementazione su nxm-db pool)
```

### Dipendenze

| Crate | Ruolo |
|-------|-------|
| `nxm-config` | Gestione configurazione (`GeneralConfig`, `ConfigPaths`) |
| `nxm-logging` | Logging strutturato JSON (file + stderr) |
| `nxm-db` | Datasource SQLite con pool + migration |
| `sqlx` | Query SQL sul pool di `nxm-db` |
| `chrono` | Timestamp RFC3339 |
| `serde` | Serializzazione modelli di dominio |
| `tokio` | Runtime per bloccare future sqlx in contesto sincrono |
| `thiserror` | Definizione errori |

## Flusso di avvio (`main.rs`)

1. `init_logging()` — installa il subscriber JSON globale ("first wins")
2. `TrackerConfig::load()` — scopre e carica la configurazione
3. `Database::open()` — apre il datasource SQLite predefinito
4. `default_migration_set()` + `db.migrate()` — applica le migration v1 e v2

## Schema del database

### Tabella `sessions` (migration v1)

| Colonna | Tipo | Note |
|---------|------|------|
| `id` | INTEGER PK AUTOINCREMENT | Identificativo univoco |
| `name` | TEXT NOT NULL | Nome della sessione |
| `started_at` | TEXT NOT NULL | Timestamp inizio (RFC3339) |
| `ended_at` | TEXT | Timestamp fine (nullable) |
| `status` | TEXT NOT NULL DEFAULT 'open' | `open`, `paused`, `closed` |
| `design_phase_complete` | INTEGER NOT NULL DEFAULT 0 | Flag booleano (0/1) |

### Tabella `feature_sessions` (migration v2)

| Colonna | Tipo | Note |
|---------|------|------|
| `id` | INTEGER PK AUTOINCREMENT | Identificativo univoco |
| `session_id` | INTEGER NOT NULL | FK su `sessions(id)` ON DELETE CASCADE |
| `name` | TEXT NOT NULL | Nome della feature session |
| `description` | TEXT | Descrizione opzionale |
| `status` | TEXT NOT NULL DEFAULT 'open' | Stato della feature session |
| `created_at` | TEXT NOT NULL | Timestamp creazione (RFC3339) |

## Stati di una work session

```
Open ──→ Paused ──→ Open  (resume)
  │                │
  └──────→ Closed  (terminal)
```

- `Closed` è uno stato terminale: una sessione chiusa non può essere riaperta.
- `close_session()` imposta `status = 'closed'` e `ended_at = now`.

## SessionRepository

Il trait `SessionRepository` astrae la persistenza. L'implementazione fornita è
`SqliteSessionRepository`, che opera sul pool restituito da `nxm_db::Db::sqlite_pool()`
tramite un runtime tokio per-istanza (le query sqlx sono async, il trait è sync).

Tutte le operazioni restituiscono `Result<_, TrackerError>`.

## File di produzione per dimensione

Tutti i file in `src/` sono sotto le 300 righe, come richiesto dalle regole del progetto.
