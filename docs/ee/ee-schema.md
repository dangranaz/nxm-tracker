# Schema database — Enterprise Edition

Lo schema EE espande quello pubblico con 13 migration aggiuntive (v3-v15),
portando il totale a 15 migration. Tutte le tabelle usano `STRICT` mode
SQLite e le chiavi esterne sono dichiarative (l'integrità referenziale è
garantita dal codice Rust, poiché il pool `nxm-db` non attiva
`PRAGMA foreign_keys`).

## Tabella: `lifecycle_state` (v3-v4)

Stati canonici del ciclo di vita di una sessione.

| Colonna | Tipo | Note |
|---------|------|------|
| `name` | TEXT PRIMARY KEY | `active`, `onhold`, `abandoned`, `done`, `superseded` |
| `description` | TEXT NOT NULL | Descrizione dello stato |
| `is_visible` | INTEGER NOT NULL | Visibile nel listing di default |
| `is_archivable` | INTEGER NOT NULL | Può essere archiviato |
| `is_reopenable` | INTEGER NOT NULL | Può essere riaperto |
| `sort_order` | INTEGER NOT NULL | Ordine di visualizzazione |

## Tabella: `lifecycle_transition` (v5-v6)

Transizioni permesse tra stati.

| Colonna | Tipo | Note |
|---------|------|------|
| `from_state` | TEXT NOT NULL | Stato di origine (FK dichiarativa) |
| `to_state` | TEXT NOT NULL | Stato di destinazione (FK dichiarativa) |
| `trigger` | TEXT NOT NULL | `set_status` o `reopen` |
| `description` | TEXT NOT NULL | Descrizione umana |

Chiave primaria composita: `(from_state, to_state, trigger)`.

## Tabella: `session` (v7)

Sostituisce/affianca la tabella pubblica `sessions` con id TEXT (slug-based).

| Colonna | Tipo | Note |
|---------|------|------|
| `id` | TEXT PRIMARY KEY | ID slug-based, fornito dal chiamante |
| `status` | TEXT NOT NULL | Riferimento a `lifecycle_state(name)` |
| `created_at` | TEXT NOT NULL | Timestamp creazione (RFC3339) |
| `updated_at` | TEXT NOT NULL | Timestamp ultimo aggiornamento |
| `consumed_at` | TEXT | Timestamp consumo (NULL finché non consumata) |
| `consumed_by` | TEXT | ID sessione che riprende questa |

## Tabella: `audit_log` (v8)

Log di audit per ogni scrittura sul lifecycle.

| Colonna | Tipo | Note |
|---------|------|------|
| `id` | INTEGER PRIMARY KEY AUTOINCREMENT | ID auto-increment |
| `session_id` | TEXT NOT NULL | Riferimento alla sessione |
| `action` | TEXT NOT NULL | Tipo di azione (`create`, `set_status`, `update_session`, `mark_consumed`, `revise`, `close`) |
| `from_state` | TEXT | Stato precedente (NULL per creazione) |
| `to_state` | TEXT NOT NULL | Stato risultante |
| `user` | TEXT NOT NULL | Utente che ha richiesto l'azione |
| `auth_method` | TEXT NOT NULL | Metodo autenticazione (es. `local`) |
| `ai_tool` | TEXT NOT NULL | Strumento AI che ha eseguito l'azione |
| `ai_model` | TEXT | Modello AI usato (opzionale) |
| `at` | TEXT NOT NULL | Timestamp azione (RFC3339) |

## Tabella: `task` (v9)

Identità stabile di un task collegato a una sessione.

| Colonna | Tipo | Note |
|---------|------|------|
| `session_id` | TEXT NOT NULL | Riferimento a `session(id)` |
| `task_id` | INTEGER NOT NULL | Progressivo per sessione (0, 1, 2…) |
| `created_at` | TEXT NOT NULL | Timestamp creazione |

Chiave primaria composita: `(session_id, task_id)`.

## Tabella: `task_revision` (v10)

Revisioni append-only dei task.

| Colonna | Tipo | Note |
|---------|------|------|
| `id` | INTEGER PRIMARY KEY AUTOINCREMENT | ID auto-increment |
| `session_id` | TEXT NOT NULL | Riferimento a `task(session_id, task_id)` |
| `task_id` | INTEGER NOT NULL | ID del task |
| `version` | INTEGER NOT NULL | Versione incrementale (0, 1, 2…) |
| `status` | TEXT NOT NULL | `open`, `in_progress`, `done`, `superseded`, `blocked`, `cancelled` |
| `title` | TEXT NOT NULL | Titolo del task |
| `body` | TEXT | Corpo del task |
| `exec_order` | INTEGER NOT NULL | Ordine di esecuzione |
| `result` | TEXT | Esito alla chiusura |
| `spawned_task_id` | INTEGER | Task generato come risultato (stessa sessione) |
| `blocked_by` | INTEGER | Task da cui è bloccato (se `status=blocked`) |
| `at` | TEXT NOT NULL | Timestamp revisione |

Chiave unica: `(session_id, task_id, version)`.

## Tabella: `task_audit` (v11)

Audit dedicato ai task.

| Colonna | Tipo | Note |
|---------|------|------|
| `id` | INTEGER PRIMARY KEY AUTOINCREMENT | ID auto-increment |
| `session_id` | TEXT NOT NULL | Riferimento a `task(session_id, task_id)` |
| `task_id` | INTEGER NOT NULL | ID del task |
| `action` | TEXT NOT NULL | `create`, `revise`, `set_status`, `close` |
| `from_status` | TEXT | Stato precedente |
| `to_status` | TEXT | Stato risultante |
| `user` | TEXT NOT NULL | Utente |
| `auth_method` | TEXT NOTNULL | Metodo autenticazione |
| `ai_tool` | TEXT NOT NULL | Strumento AI |
| `ai_model` | TEXT | Modello AI (opzionale) |
| `at` | TEXT NOT NULL | Timestamp |

## Tabella: `session_handoff` (v12)

Handoff versionato di una sessione (append-only).

| Colonna | Tipo | Note |
|---------|------|------|
| `id` | INTEGER PRIMARY KEY AUTOINCREMENT | ID auto-increment |
| `session_id` | TEXT NOT NULL | Riferimento a `session(id)` |
| `version` | INTEGER NOT NULL | Versione incrementale per sessione |
| `goal_next` | TEXT | Obiettivo prossimo (opzionale) |
| `decisions` | TEXT | JSON array di stringhe |
| `next_steps` | TEXT | JSON array di stringhe |
| `failed_approaches` | TEXT | JSON array di stringhe |
| `relevant_files` | TEXT | JSON array di stringhe |
| `context_usage_pct` | INTEGER | Percentuale uso contesto (nullable) |
| `at` | TEXT NOT NULL | Timestamp |

Chiave unica: `(session_id, version)`.

## Tabella: `slug_counter` (v13)

Contatore monotonic per minting di chiavi session.

| Colonna | Tipo | Note |
|---------|------|------|
| `slug` | TEXT PRIMARY KEY | Slug della sessione |
| `next_nn` | INTEGER NOT NULL | Prossimo numero da assegnare |

## Tabella: `prompt` (v14)

Registry dei brief di delega (metadato puro).

| Colonna | Tipo | Note |
|---------|------|------|
| `session_id` | TEXT NOT NULL | Riferimento a `session(id)` |
| `task_no` | TEXT NOT NULL | Numero task (es. `"01"`) |
| `slug` | TEXT NOT NULL | Slug del brief |
| `lang` | TEXT NOT NULL | `ita` o `eng` |
| `path` | TEXT NOT NULL | Percorso file del brief |

Chiave primaria: `(session_id, task_no, lang)`. Upsert idempotente.

## Tabella: `retention_log` (v15)

Traccia permanente delle purghe dell'audit (NON soggetta a retention).

| Colonna | Tipo | Note |
|---------|------|------|
| `id` | INTEGER PRIMARY KEY AUTOINCREMENT | ID auto-increment |
| `purged_count` | INTEGER NOT NULL | Righe `audit_log` cancellate |
| `older_than` | TEXT NOT NULL | Cutoff: righe con `at < older_than` |
| `at` | TEXT NOT NULL | Timestamp purga (RFC3339) |

## Diagramma relazionale

```
sessions (legacy, id INTEGER)      session (EE, id TEXT PK)
  |                                   |
  |                                   | 1:N
  v                                   v
feature_sessions                  lifecycle_state (1:5 stati)
                                       |
                                       | 1:N
                                       v
                                    lifecycle_transition
                                       |
                                       | 1:N
                                       v
                                    audit_log (1:N per sessione)
                                       |
        task (1:N per sessione)       |
        |                             |
        v                             v
    task_revision (1:N per task)    session_handoff (1:N per sessione)
        |
        v
    task_audit (1:N per task_revision)
```
