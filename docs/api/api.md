# API pubblica — `nxm-tracker` v0.6.0

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

---

## `TrackerError`

Enumerazione degli errori del crate:

| Variante | Origine |
|----------|---------|
| `Config(nxm_config::ConfigError)` | Errore configurazione |
| `Logging(nxm_logging::LoggingError)` | Errore inizializzazione logging |
| `Database(nxm_db::DbError)` | Errore datasource |
| `InvalidSessionStatus(String)` | Status string non riconosciuta nel DB |

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

Implementazione di [`SessionRepository`] su SQLite.

| Metodo | Descrizione |
|--------|-------------|
| `from_database(db: Database) -> Self` | Costruisce da [`crate::Database`] |
| `db(&self) -> &nxm_db::Db` | Restituisce l'handle sottostante |

Le query sqlx (async) vengono eseguite su un runtime tokio per-istanza, così il trait rimane sincrono.

---

## Test

I test di integrazione sono in `tests/session_test.rs` (11 test) e coprono:
- CRUD completo su sessioni e feature session
- Transizioni di stato e flag
- Comportamento su ID inesistenti
- Isolamento delle feature session per work session

Tutti i test usano directory temporanee per non toccare `~/.nxm/shared-db`.
