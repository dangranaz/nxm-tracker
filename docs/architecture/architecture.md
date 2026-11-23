# Architettura — `nxm-tracker`

## Panoramica

`nxm-tracker` è un crate Rust (edition 2021) standalone che traccia sessioni di lavoro
lunghe, multi-giorno, le sessioni di feature integrate create dopo la fase di design,
oltre a handoff (1:1), piano di attività (1:N) ed execution task (1:N) per sessione.

Il progetto segue i principi di **deep modules**: interfacce semplici, implementazione
nascosta, ogni file di produzione sotto le 300 righe, test separati in `tests/`.

## Struttura dei moduli

```
nxm-tracker crate
├── main.rs               # Entry point: logging → config → db → migration → tool → serve
├── lib.rs                # Esporta API pubblica
├── config.rs             # TrackerConfig (wrapper su nxm-config)
├── logging.rs            # init_logging() (wrapper su nxm-logging)
├── error.rs              # TrackerError (unione degli errori dei sottosistemi)
├── database.rs           # Database (wrapper su nxm-db) + default_migration_set()
├── session.rs            # Session, SessionStatus, FeatureSession, SessionRepository trait
├── session_repository.rs # SqliteSessionRepository (impl su nxm-db pool) + create_session_full
├── handoff.rs            # Handoff, ActivityPlan, HandoffRepository trait
├── handoff_repository.rs # impl HandoffRepository su SqliteSessionRepository
├── task.rs               # Task, TaskStatus, TaskReport, NewTask, TaskRepository trait
├── task_repository.rs    # impl TaskRepository (close_task/invalidate_task in transazione)
├── git_workspace.rs      # helper git (integration branch)
├── mcp_tools.rs          # tool MCP sessione + feature session
└── tracking_tools.rs     # tool MCP handoff / piano / task
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

### Tabella `handoffs` (migration v3) — 1:1 con la sessione

| Colonna | Tipo | Note |
|---------|------|------|
| `id` | INTEGER PK AUTOINCREMENT | Identificativo univoco |
| `session_id` | INTEGER NOT NULL UNIQUE | FK su `sessions(id)` ON DELETE CASCADE |
| `content` | TEXT NOT NULL | Testo libero dell'handoff, aggiornato frequentemente |
| `updated_at` | TEXT NOT NULL | Ultimo aggiornamento (RFC3339) |

### Tabella `activity_plans` (migration v4) — 1:N con la sessione

| Colonna | Tipo | Note |
|---------|------|------|
| `id` | INTEGER PK AUTOINCREMENT | Identificativo univoco |
| `session_id` | INTEGER NOT NULL | FK su `sessions(id)` ON DELETE CASCADE |
| `position` | INTEGER NOT NULL | Ordine 1-based dello step nel piano |
| `description` | TEXT NOT NULL | Descrizione dello step |
| `created_at` | TEXT NOT NULL | Timestamp creazione (RFC3339) |

### Tabella `tasks` (migration v5) — 1:N con la sessione

| Colonna | Tipo | Note |
|---------|------|------|
| `id` | INTEGER PK AUTOINCREMENT | Identificativo univoco |
| `session_id` | INTEGER NOT NULL | FK su `sessions(id)` ON DELETE CASCADE |
| `name` | TEXT NOT NULL | Nome del task = nome del branch git del task |
| `status` | TEXT NOT NULL DEFAULT 'open' | `open`, `closed`, `invalid` |
| `note` | TEXT | Motivazione (es. reason di invalidazione) |
| `created_at` | TEXT NOT NULL | Timestamp creazione (RFC3339) |
| `closed_at` | TEXT | Timestamp chiusura (solo se `closed`) |

### Tabella `task_reports` (migration v6) — 1:1 con il task

| Colonna | Tipo | Note |
|---------|------|------|
| `id` | INTEGER PK AUTOINCREMENT | Identificativo univoco |
| `task_id` | INTEGER NOT NULL UNIQUE | FK su `tasks(id)` ON DELETE CASCADE |
| `content` | TEXT NOT NULL | Report finale obbligatorio alla chiusura |
| `created_at` | TEXT NOT NULL | Timestamp creazione (RFC3339) |

## Stati di una work session (`SessionStatus`)

Ogni work session può trovarsi in uno dei tre stati seguenti. Le transizioni
consentite sono indicate nella colonna "Transizioni possibili".

| Stato | Valore DB | Descrizione | Transizioni possibili | Operatore MCP |
|-------|-----------|-------------|----------------------|---------------|
| `Open` | `open` | Sessione attivamente in lavorazione. Stato iniziale di ogni nuova sessione. | `Paused` (sospendi), `Closed` (chiudi) | `create_session`, `update_session`, `close_session` |
| `Paused` | `paused` | Sessione temporaneamente sospesa. Può essere ripresa in un secondo momento. | `Open` (riprendi), `Closed` (chiudi) | `update_session`, `close_session` |
| `Closed` | `closed` | Sessione terminata e archiviato. Stato terminale: non può essere riaperta. | Nessuna (terminale) | `close_session` |

### Diagramma delle transizioni

```
Open ──→ Paused ──→ Open  (riprendi)
  │                │
  └──────→ Closed  (terminale)
```

### Regole di transizione

- `Open` → `Paused`: la sessione viene sospesa, `ended_at` rimane `NULL`.
- `Paused` → `Open`: la sessione viene ripresa, `ended_at` rimane `NULL`.
- `Open` → `Closed`: la sessione viene chiusa, `ended_at` viene impostato a `now`.
- `Paused` → `Closed`: la sessione viene chiusa da stato sospeso, `ended_at` viene impostato a `now`.
- `Closed` → *nessuno*: stato terminale, nessuna transizione in uscita è consentita.

### Elenco attività per gestione stati

| Attività | Tool MCP | Descrizione |
|----------|----------|-------------|
| Creare una nuova sessione | `create_session` | Inserisce una sessione con stato `open` e `started_at` impostato |
| Recuperare una sessione | `get_session` | Restituisce la sessione con il suo stato attuale |
| Listare tutte le sessioni | `list_sessions` | Restituisce tutte le sessioni ordinate per `started_at` desc |
| Aggiornare nome/stato/flag | `update_session` | Modifica `name`, `status` (`open`/`paused`/`closed`) e `design_phase_complete` |
| Chiudere una sessione | `close_session` | Imposta `status = 'closed'` e `ended_at = now` |
| Segnare design phase completa | `set_design_phase_complete` | Imposta `design_phase_complete = true/false` |
| Creare una feature session | `create_feature_session` | Crea una feature session collegata a una work session |
| Listare feature session | `list_feature_sessions` | Restituisce le feature session di una work session |

- `Closed` è uno stato terminale: una sessione chiusa non può essere riaperta.
- `close_session()` imposta `status = 'closed'` e `ended_at = now`.

## Stati di un task (`TaskStatus`)

I task vengono creati dopo la sessione (`create_task`) per eseguire il piano di
attività e/o la sessione stessa. Il `name` del task è anche il nome del branch git.

| Stato | Valore DB | Descrizione | Transizioni possibili | Operatore MCP |
|-------|-----------|-------------|----------------------|---------------|
| `Open` | `open` | Task in attesa di esecuzione. Stato iniziale. | `Closed` (con report obbligatorio), `Invalid` | `create_task`, `close_task`, `invalidate_task` |
| `Closed` | `closed` | Task completato; ha un report finale 1:1 in `task_reports`. | Nessuna (terminale) | `close_task` |
| `Invalid` | `invalid` | Task senza più valore (es. superato dall'esito di un altro task). | Nessuna (terminale) | `invalidate_task` |

### Regole di transizione

- `Open → Closed`: solo via `close_task(id, report)` con report non vuoto; il report
  viene inserito in `task_reports` (1:1) e `closed_at = now`, in una singola
  transazione atomica.
- `Open → Invalid`: via `invalidate_task(id, reason?)`; il `reason` opzionale è
  salvato in `tasks.note`.
- `Closed`/`Invalid` → nessuna transizione: stati terminali.

## SessionRepository

Il trait `SessionRepository` astrae la persistenza. L'implementazione fornita è
`SqliteSessionRepository`, che opera sul pool restituito da `nxm_db::Db::sqlite_pool()`
tramite un runtime tokio per-istanza (le query sqlx sono async, il trait è sync).

Tutte le operazioni restituiscono `Result<_, TrackerError>`.

## File di produzione per dimensione

Tutti i file in `src/` sono sotto le 300 righe, come richiesto dalle regole del progetto.
