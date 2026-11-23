# API pubblica — `nxm-tracker` v1.1.0

## Funzioni di livello superiore

### `product_name() -> &'static str`

Restituisce il nome canonico del prodotto binario.

**Esempio:**
```rust
use nxm_tracker::product_name;

assert_eq!(product_name(), "nxm-tracker");
```

---

## `TrackerConfig`

Wrapper attorno a [`nxm_config::GeneralConfig`] per la gestione della configurazione.

### Metodi

| Metodo | Descrizione |
|--------|-------------|
| `load() -> Result<Self, TrackerError>` | Scopre il config root e carica/crea default |
| `load_or_create_from(paths: &ConfigPaths) -> Result<Self, TrackerError>` | Carica da path esplicito, crea default se mancante |
| `load_from(paths: &ConfigPaths) -> Result<Self, TrackerError>` | Carica da path esplicito senza creare default |
| `save(&self) -> Result<(), TrackerError>` | Salva nel config root scoperto |
| `save_to(&self, paths: &ConfigPaths) -> Result<(), TrackerError>` | Salva in path esplicito |
| `server_port(&self) -> u16` | Porta del server MCP/HTTP |
| `database_url(&self) -> &str` | URL di connessione al database |
| `database_driver(&self) -> DatabaseKind` | Tipo di driver del database |

### Esempio
```rust
use nxm_config::ConfigPaths;
use nxm_tracker::TrackerConfig;

let paths = ConfigPaths::with_root(&temp_root);
let config = TrackerConfig::load_or_create_from(&paths)?;
assert_eq!(config.server_port(), 8080);
```

---

## `init_logging() -> Result<(), TrackerError>`

Inizializza il logging strutturato JSON. I log vanno in
`~/.nxmconfig/logs/nxm-tracker.log` (rotazione giornaliera, max 7 file) e stderr.
Il livello si risolve da `NXM_MCP_LOG`, poi `RUST_LOG`, poi default `"info"`.

Safe da chiamare più volte: solo la prima call installa il subscriber.

---

## `Database`

Wrapper attorno a [`nxm_db::Db`] per l'accesso al datasource SQLite.

| Metodo | Descrizione |
|--------|-------------|
| `open() -> Result<Self, TrackerError>` | Apri il datasource SQLite predefinito |
| `open_from(config: &DbConfig) -> Result<Self, TrackerError>` | Apri da config esplicita |
| `handle(&self) -> &nxm_db::Db` | Restituisce l'handle sottostante |
| `migrate(&self, set: &MigrationSet) -> Result<(), TrackerError>` | Esegue una migration set |

---

## `default_migration_set() -> Result<MigrationSet, TrackerError>`

Restituisce il set di migration predefinito:
- v1: `create_sessions`
- v2: `create_feature_sessions` (con FK su `sessions(id)`)
- v3: `create_handoffs` (1:1 con `sessions`, `session_id` UNIQUE)
- v4: `create_activity_plans` (1:N con `sessions`)
- v5: `create_tasks` (1:N con `sessions`)
- v6: `create_task_reports` (1:1 con `tasks`, `task_id` UNIQUE)

---

## `TrackerError`

Enumerazione degli errori del crate:

| Variante | Origine |
|----------|---------|
| `Config(nxm_config::ConfigError)` | Errore configurazione |
| `Logging(nxm_logging::LoggingError)` | Errore inizializzazione logging |
| `Database(nxm_db::DbError)` | Errore datasource |
| `InvalidSessionStatus(String)` | Status sessione non riconosciuto nel DB |
| `InvalidTaskStatus(String)` | Status task non riconosciuto nel DB |
| `TaskNotOpen { id, status }` | Operazione lifecycle su task non `open` |

---

## Modello di dominio (`session`)

### `Session`

| Campo | Tipo | Descrizione |
|-------|------|-------------|
| `id` | `Option<i64>` | ID assegnato dal DB (None prima dell'insert) |
| `name` | `String` | Nome della sessione |
| `started_at` | `String` | Timestamp inizio (RFC3339) |
| `ended_at` | `Option<String>` | Timestamp fine |
| `status` | `SessionStatus` | Stato della sessione |
| `design_phase_complete` | `bool` | Flag completamento design phase |

### `SessionStatus`

Enum: `Open`, `Paused`, `Closed`. Si serializza in DB come stringhe `"open"`, `"paused"`, `"closed"`.

### `FeatureSession`

| Campo | Tipo | Descrizione |
|-------|------|-------------|
| `id` | `Option<i64>` | ID assegnato dal DB |
| `session_id` | `i64` | FK alla work session padre |
| `name` | `String` | Nome della feature session |
| `description` | `Option<String>` | Descrizione opzionale |
| `status` | `String` | Stato (default `"open"`) |
| `created_at` | `String` | Timestamp creazione (RFC3339) |

### Struct di creazione

| Struct | Campi |
|--------|-------|
| `NewSession` | `name: String`, `started_at: String` |
| `NewFeatureSession` | `session_id: i64`, `name: String`, `description: Option<String>` |

---

## `SessionRepository` (trait)

| Metodo | Descrizione |
|--------|-------------|
| `create_session(&self, new: &NewSession) -> Result<Session, TrackerError>` | Inserisce una nuova sessione |
| `get_session(&self, id: i64) -> Result<Option<Session>, TrackerError>` | Restituisce sessione per ID |
| `list_sessions(&self) -> Result<Vec<Session>, TrackerError>` | Tutte le sessioni (started_at DESC) |
| `set_session_status(&self, id: i64, status: SessionStatus) -> Result<bool, TrackerError>` | Aggiorna status |
| `close_session(&self, id: i64) -> Result<bool, TrackerError>` | Chiude sessione, imposta `ended_at` |
| `set_design_phase_complete(&self, id: i64, complete: bool) -> Result<bool, TrackerError>` | Toggle flag design phase |
| `create_feature_session(&self, new: &NewFeatureSession) -> Result<FeatureSession, TrackerError>` | Inserisce feature session |
| `list_feature_sessions(&self, session_id: i64) -> Result<Vec<FeatureSession>, TrackerError>` | Feature session di una work session |

Tutte le operazioni che accettano un `id` restituiscono `false` se l'entità non esiste.

---

## `SqliteSessionRepository`

Implementazione di [`SessionRepository`], [`HandoffRepository`] e [`TaskRepository`]
su SQLite.

| Metodo | Descrizione |
|--------|-------------|
| `from_database(db: Database) -> Self` | Costruisce da [`crate::Database`] |
| `db(&self) -> &nxm_db::Db` | Restituisce l'handle sottostante |
| `create_session_full(new, handoff, plan)` | Crea sessione + handoff (1:1, opzionale) + piano (1:N) in una transazione |

Le query sqlx (async) vengono eseguite su un runtime tokio per-istanza, così i trait rimangono sincroni.

---

## Handoff e piano di attività (`handoff`)

### `Handoff` (1:1 con la sessione)

| Campo | Tipo | Descrizione |
|-------|------|-------------|
| `id` | `Option<i64>` | ID assegnato dal DB |
| `session_id` | `i64` | FK alla work session (UNIQUE) |
| `content` | `String` | Testo libero dell'handoff, aggiornato frequentemente |
| `updated_at` | `String` | Ultimo aggiornamento (RFC3339) |

### `ActivityPlan` (1:N con la sessione)

| Campo | Tipo | Descrizione |
|-------|------|-------------|
| `id` | `Option<i64>` | ID assegnato dal DB |
| `session_id` | `i64` | FK alla work session |
| `position` | `i64` | Ordine 1-based dello step |
| `description` | `String` | Descrizione dello step |
| `created_at` | `String` | Timestamp creazione (RFC3339) |

### `HandoffRepository` (trait)

| Metodo | Descrizione |
|--------|-------------|
| `get_handoff(&self, session_id: i64) -> Result<Option<Handoff>, TrackerError>` | Handoff della sessione, `None` se assente |
| `upsert_handoff(&self, session_id: i64, content: &str) -> Result<Handoff, TrackerError>` | Crea o sostituisce l'handoff (1:1), aggiorna `updated_at` |
| `list_activity_plans(&self, session_id: i64) -> Result<Vec<ActivityPlan>, TrackerError>` | Step del piano ordinati per `position` |

---

## Task di esecuzione (`task`)

### `TaskStatus`

Enum: `Open`, `Closed`, `Invalid`. Serializza come stringhe minuscole
(`"open"`, `"closed"`, `"invalid"`). Transizioni: `Open → Closed` (con report
obbligatorio), `Open → Invalid`; `Closed`/`Invalid` terminali.

### `Task`

| Campo | Tipo | Descrizione |
|-------|------|-------------|
| `id` | `Option<i64>` | ID assegnato dal DB |
| `session_id` | `i64` | FK alla work session |
| `name` | `String` | Nome del task = nome del branch git del task |
| `status` | `TaskStatus` | Stato del task |
| `note` | `Option<String>` | Nota (es. reason di invalidazione) |
| `created_at` | `String` | Timestamp creazione (RFC3339) |
| `closed_at` | `Option<String>` | Timestamp chiusura (solo se `Closed`) |

### `TaskReport` (1:1 con il task)

| Campo | Tipo | Descrizione |
|-------|------|-------------|
| `id` | `Option<i64>` | ID assegnato dal DB |
| `task_id` | `i64` | FK al task (UNIQUE) |
| `content` | `String` | Report finale, obbligatorio alla chiusura |
| `created_at` | `String` | Timestamp creazione (RFC3339) |

### `NewTask`

| Campo | Tipo |
|-------|------|
| `session_id` | `i64` |
| `name` | `String` (anche nome del branch git) |

### `TaskRepository` (trait)

| Metodo | Descrizione |
|--------|-------------|
| `create_task(&self, new: &NewTask) -> Result<Task, TrackerError>` | Nuovo task in stato `open` |
| `get_task(&self, id: i64) -> Result<Option<Task>, TrackerError>` | Task per ID |
| `list_tasks(&self, session_id: i64) -> Result<Vec<Task>, TrackerError>` | Tutti i task della sessione (created_at DESC) |
| `list_tasks_by_status(&self, session_id: i64, status: TaskStatus) -> Result<Vec<Task>, TrackerError>` | Task filtrati per stato |
| `close_task(&self, id: i64, report: &str) -> Result<Option<Task>, TrackerError>` | Chiude un task `open` con report obbligatorio (transazione atomica); `None` se non trovato, `TaskNotOpen` se non `open` |
| `invalidate_task(&self, id: i64, reason: Option<&str>) -> Result<Option<Task>, TrackerError>` | Task `open` → `invalid`, con reason opzionale in `note` |
| `get_task_report(&self, task_id: i64) -> Result<Option<TaskReport>, TrackerError>` | Report finale del task |

---

## Test

I test di integrazione sono in `tests/session_test.rs` (11 test) e coprono:
- CRUD completo su sessioni e feature session
- Transizioni di stato e flag
- Comportamento su ID inesistenti
- Isolamento delle feature session per work session

Tutti i test usano directory temporanee per non toccare `~/.nxm/shared-db`.
