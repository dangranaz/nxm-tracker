# Guida utente — `nxm-tracker` MCP Tools

Questa guida spiega come utilizzare i 8 tool MCP esposti da `nxm-tracker`
da qualsiasi client MCP (es. pi, Claude Code, OpenCode, Cursor, ecc.).

## Trasporto

Il server `nxm-tracker` espone i tool tramite trasporto **stdio** (JSON-RPC 2.0
su stdin/stdout). Per avviare il server:

```bash
# 1. Costruisci il binario release
./scripts/build.sh

# 2. Avvia il server in background
./scripts/trackerUp

# 3. Il server è ora in ascolto su stdin/stdout pronto per ricevere richieste MCP
# 4. Per fermarlo
./scripts/trackerDown
```

## Tool disponibili

Il server registra i seguenti 17 tool:

| Tool | Descrizione |
|------|-------------|
| `create_session` | Crea una nuova sessione di lavoro (con handoff e piano opzionali) |
| `get_session` | Legge una sessione per ID |
| `list_sessions` | Lista tutte le sessioni (ordinate per data discendente) |
| `update_session` | Aggiorna nome, stato o flag design phase di una sessione |
| `close_session` | Chiude una sessione (imposta `closed` e `ended_at`) |
| `set_design_phase_complete` | Imposta il flag `design_phase_complete` |
| `create_feature_session` | Crea una feature session collegata a una work session |
| `list_feature_sessions` | Lista le feature session di una work session |
| `get_handoff` | Legge l'handoff 1:1 di una sessione |
| `update_handoff` | Crea/sostituisce l'handoff 1:1 di una sessione (upsert) |
| `list_activity_plans` | Lista i passi del piano di attività di una sessione (ordinati) |
| `create_task` | Crea un task di esecuzione per una sessione (nome = branch git) |
| `list_tasks` | Lista i task di una sessione, filtro opzionale per stato |
| `get_task` | Legge un task per ID |
| `close_task` | Chiude un task `open` — report finale obbligatorio (1:1) |
| `invalidate_task` | Marca un task `open` come `invalid` (reason opzionale in `note`) |
| `get_task_report` | Legge il report finale 1:1 di un task |

Tutte le risposte seguono l'envelope MCP standard:

```json
{
  "content": [{"type": "text", "text": "<payload>"}],
  "isError": false
}
```

In caso di errore, `isError` è `true` e `text` contiene il messaggio descrittivo.

---

## 1. `create_session`

Crea una nuova sessione di lavoro.

### Parametri

| Parametro | Tipo | Obbligatorio | Descrizione |
|-----------|------|--------------|-------------|
| `name` | stringa | Sì | Nome della sessione (non vuoto) |
| `started_at` | stringa | Sì | Timestamp di inizio in formato RFC3339 |
| `working_directory` | stringa | No | Percorso a un progetto git. Se fornito, il server verifica che sia un repo git valido e crea un branch `integration_<short>` da `develop`/`master`/`main` prima di creare la sessione |
| `handoff` | stringa | No | Testo dell'handoff, salvato 1:1 con la sessione (aggiornabile poi con `update_handoff`) |
| `plan` | array di stringhe | No | Piano di attività: lista ordinata di passi della sessione |

### Esempio request

```json
{
  "name": "Refactoring Q4",
  "started_at": "2026-11-20T08:00:00+01:00"
}
```

### Esempio request (con handoff e piano)

```json
{
  "name": "Refactoring Q4",
  "started_at": "2026-11-20T08:00:00+01:00",
  "handoff": "Design completato, vedi docs/architecture.md",
  "plan": ["setup schema DB", "scrivi test", "refactor modulo auth"]
}
```

### Esempio request (con integrazione git)

```json
{
  "name": "Refactoring Q4",
  "started_at": "2026-11-20T08:00:00+01:00",
  "working_directory": "/percorso/al/progetto/git"
}
```

### Esempio response

```json
{
  "content": [
    {
      "type": "text",
      "text": "{\"id\":1,\"name\":\"Refactoring Q4\",\"started_at\":\"2026-11-20T08:00:00+01:00\",\"ended_at\":null,\"status\":\"open\",\"design_phase_complete\":false}"
    }
  ],
  "isError": false
}
```

---

## 2. `get_session`

Legge una sessione esistente per ID.

### Parametri

| Parametro | Tipo | Obbligatorio | Descrizione |
|-----------|------|--------------|-------------|
| `id` | integer | Sì | ID della sessione |

### Esempio request

```json
{
  "id": 1
}
```

### Esempio response (trovata)

```json
{
  "content": [
    {
      "type": "text",
      "text": "{\"id\":1,\"name\":\"Refactoring Q4\",\"started_at\":\"2026-11-20T08:00:00+01:00\",\"ended_at\":null,\"status\":\"open\",\"design_phase_complete\":false}"
    }
  ],
  "isError": false
}
```

### Esempio response (non trovata)

```json
{
  "content": [
    {
      "type": "text",
      "text": "session 1 not found"
    }
  ],
  "isError": true
}
```

---

## 3. `list_sessions`

Lista tutte le sessioni di lavoro, ordinate per `started_at` discendente.

### Parametri

Nessuno.

### Esempio request

```json
{}
```

### Esempio response

```json
{
  "content": [
    {
      "type": "text",
      "text": "[{\"id\":2,\"name\":\"Seconda\",\"started_at\":\"2026-11-21T09:00:00+01:00\",\"ended_at\":null,\"status\":\"open\",\"design_phase_complete\":false},{\"id\":1,\"name\":\"Prima\",\"started_at\":\"2026-11-20T08:00:00+01:00\",\"ended_at\":null,\"status\":\"open\",\"design_phase_complete\":false}]"
    }
  ],
  "isError": false
}
```

---

## 4. `update_session`

Aggiorna uno o più campi di una sessione esistente. Almeno uno tra `name`, `status`, `design_phase_complete` deve essere fornito.

### Parametri

| Parametro | Tipo | Obbligatorio | Descrizione |
|-----------|------|--------------|-------------|
| `id` | integer | Sì | ID della sessione |
| `name` | stringa | No | Nuovo nome (non vuoto se fornito) |
| `status` | stringa | No | Nuovo stato: `open`, `paused`, `closed` |
| `design_phase_complete` | boolean | No | Flag design phase completa |

### Esempio request (aggiorna nome e stato)

```json
{
  "id": 1,
  "name": "Refactoring Q4 (riveduto)",
  "status": "paused"
}
```

### Esempio response

```json
{
  "content": [
    {
      "type": "text",
      "text": "{\"id\":1,\"name\":\"Refactoring Q4 (riveduto)\",\"started_at\":\"2026-11-20T08:00:00+01:00\",\"ended_at\":null,\"status\":\"paused\",\"design_phase_complete\":false}"
    }
  ],
  "isError": false
}
```

---

## 5. `close_session`

Chiude una sessione di lavoro imposta `status = "closed"` e `ended_at` al timestamp corrente.

### Parametri

| Parametro | Tipo | Obbligatorio | Descrizione |
|-----------|------|--------------|-------------|
| `id` | integer | Sì | ID della sessione |

### Esempio request

```json
{
  "id": 1
}
```

### Esempio response (successo)

```json
{
  "content": [
    {
      "type": "text",
      "text": "session 1 closed"
    }
  ],
  "isError": false
}
```

---

## 6. `set_design_phase_complete`

Imposta il flag `design_phase_complete` su una sessione.

### Parametri

| Parametro | Tipo | Obbligatorio | Descrizione |
|-----------|------|--------------|-------------|
| `id` | integer | Sì | ID della sessione |
| `complete` | boolean | Sì | Valore del flag |

### Esempio request

```json
{
  "id": 1,
  "complete": true
}
```

### Esempio response

```json
{
  "content": [
    {
      "type": "text",
      "text": "session 1 design_phase_complete set to true"
    }
  ],
  "isError": false
}
```

---

## 7. `create_feature_session`

Crea una nuova feature session collegata a una work session esistente.

### Parametri

| Parametro | Tipo | Obbligatorio | Descrizione |
|-----------|------|--------------|-------------|
| `session_id` | integer | Sì | ID della work session padre (deve esistere) |
| `name` | stringa | Sì | Nome della feature session (non vuoto) |
| `description` | stringa | No | Descrizione della feature session |

### Esempio request

```json
{
  "session_id": 1,
  "name": "Implementa OAuth",
  "description": "Aggiungi flusso di login OpenID Connect"
}
```

### Esempio response

```json
{
  "content": [
    {
      "type": "text",
      "text": "{\"id\":1,\"session_id\":1,\"name\":\"Implementa OAuth\",\"description\":\"Aggiungi flusso di login OpenID Connect\",\"status\":\"open\",\"created_at\":\"2026-11-20T08:00:00+01:00\"}"
    }
  ],
  "isError": false
}
```

---

## 8. `list_feature_sessions`

Lista tutte le feature session di una work session, ordinate per `created_at` discendente.

### Parametri

| Parametro | Tipo | Obbligatorio | Descrizione |
|-----------|------|--------------|-------------|
| `session_id` | integer | Sì | ID della work session padre |

### Esempio request

```json
{
  "session_id": 1
}
```

### Esempio response

```json
{
  "content": [
    {
      "type": "text",
      "text": "[{\"id\":1,\"session_id\":1,\"name\":\"Implementa OAuth\",\"description\":\"Aggiungi flusso di login OpenID Connect\",\"status\":\"open\",\"created_at\":\"2026-11-20T08:00:00+01:00\"}]"
    }
  ],
  "isError": false
}
```

---

## 9. `get_handoff`

Legge l'handoff 1:1 di una sessione.

### Parametri

| Parametro | Tipo | Obbligatorio | Descrizione |
|-----------|------|--------------|-------------|
| `session_id` | intero | Sì | ID della work session |

### Esempio request

```json
{ "session_id": 1 }
```

### Esempio response

```json
{
  "content": [
    {
      "type": "text",
      "text": "{\"id\":1,\"session_id\":1,\"content\":\"Design completato\",\"updated_at\":\"2026-11-22T09:00:00+01:00\"}"
    }
  ],
  "isError": false
}
```

Errore se la sessione non ha handoff: `session <id> has no handoff`.

---

## 10. `update_handoff`

Crea o sostituisce l'handoff 1:1 di una sessione (upsert su `session_id`).

### Parametri

| Parametro | Tipo | Obbligatorio | Descrizione |
|-----------|------|--------------|-------------|
| `session_id` | intero | Sì | ID della work session |
| `content` | stringa | Sì | Nuovo contenuto dell'handoff (non vuoto; sostituisce il precedente) |

### Esempio request

```json
{ "session_id": 1, "content": "Handoff aggiornato: task 1 e 2 chiusi, task 3 in corso" }
```

---

## 11. `list_activity_plans`

Lista i passi del piano di attività di una sessione, ordinati per `position`.

### Parametri

| Parametro | Tipo | Obbligatorio | Descrizione |
|-----------|------|--------------|-------------|
| `session_id` | intero | Sì | ID della work session |

### Esempio request

```json
{ "session_id": 1 }
```

---

## 12. `create_task`

Crea un task di esecuzione per una sessione (stato iniziale `open`). Il `name`
del task è anche il nome del branch git del task.

### Parametri

| Parametro | Tipo | Obbligatorio | Descrizione |
|-----------|------|--------------|-------------|
| `session_id` | intero | Sì | ID della work session |
| `name` | stringa | Sì | Nome del task = nome del branch (es. `task/setup-db`) |

### Esempio request

```json
{ "session_id": 1, "name": "task/setup-db" }
```

### Esempio response

```json
{
  "content": [
    {
      "type": "text",
      "text": "{\"id\":1,\"session_id\":1,\"name\":\"task/setup-db\",\"status\":\"open\",\"note\":null,\"created_at\":\"2026-11-22T09:05:00+01:00\",\"closed_at\":null}"
    }
  ],
  "isError": false
}
```

---

## 13. `list_tasks`

Lista i task di una sessione, con filtro opzionale per stato.

### Parametri

| Parametro | Tipo | Obbligatorio | Descrizione |
|-----------|------|--------------|-------------|
| `session_id` | intero | Sì | ID della work session |
| `status` | stringa | No | Filtro: `open`, `closed`, `invalid` |

### Esempio request

```json
{ "session_id": 1, "status": "open" }
```

---

## 14. `get_task`

Legge un task per ID. Errore se non esiste: `task <id> not found`.

### Parametri

| Parametro | Tipo | Obbligatorio | Descrizione |
|-----------|------|--------------|-------------|
| `id` | intero | Sì | ID del task |

### Esempio request

```json
{ "id": 1 }
```

---

## 15. `close_task`

Chiude un task `open`: il parametro `report` è **obbligatorio** e viene
salvato 1:1 in `task_reports`. Imposta `status='closed'` e `closed_at=now` in
transazione atomica.

### Parametri

| Parametro | Tipo | Obbligatorio | Descrizione |
|-----------|------|--------------|-------------|
| `id` | intero | Sì | ID del task |
| `report` | stringa | Sì | Report finale del task (non vuoto) |

### Esempio request

```json
{ "id": 1, "report": "Schema migrato, 42 test verdi" }
```

Errori: `task <id> not found`, `task <id> is not open (status: <status>)`,
`argument 'report' must not be empty`.

---

## 16. `invalidate_task`

Marca un task `open` come `invalid` (stato terminale): utile quando al termine di
un task un altro task creato prima non ha più valore.

### Parametri

| Parametro | Tipo | Obbligatorio | Descrizione |
|-----------|------|--------------|-------------|
| `id` | intero | Sì | ID del task |
| `reason` | stringa | No | Motivazione, salvata in `note` |

### Esempio request

```json
{ "id": 2, "reason": "superato dall'esito del task 1" }
```

---

## 17. `get_task_report`

Legge il report finale 1:1 di un task. Errore se il task non esiste
(`task <id> not found`) o non ha ancora report (`task <id> has no report`).

### Parametri

| Parametro | Tipo | Obbligatorio | Descrizione |
|-----------|------|--------------|-------------|
| `task_id` | intero | Sì | ID del task |

### Esempio request

```json
{ "task_id": 1 }
```

---

## Stati delle sessioni

| Stato | Descrizione | Transizioni consentite |
|-------|-------------|----------------------|
| `open` | Sessione attivamente in lavorazione | → `paused`, → `closed` |
| `paused` | Sessione temporaneamente sospesa | → `open` (riprendi), → `closed` |
| `closed` | Sessione terminata e archiviata | Nessuna (non riapribile) |

## Stati dei task

| Stato | Descrizione | Transizioni consentite |
|-------|-------------|----------------------|
| `open` | Task in attesa di esecuzione | → `closed` (con report obbligatorio), → `invalid` |
| `closed` | Task completato, report 1:1 presente | Nessuna (terminale) |
| `invalid` | Task senza più valore (es. superato da un altro task) | Nessuna (terminale) |

---

## Validazioni di input (v1.0.0)

Tutti i tool validano gli input di ingresso. In caso di input non valido, il tool restituisce `isError=true` con un messaggio descrittivo:

| Validazione | Messaggio errore |
|-------------|------------------|
| Nome sessione vuoto | `argument 'name' must not be empty` |
| Timestamp non RFC3339 | `argument 'started_at' must be a valid RFC3339 timestamp` |
| Sessione ID inesistente | `session <id> not found` |
| Nome feature session vuoto | `argument 'name' must not be empty` |
| `session_id` padre inesistente | `session <id> not found` |
| Handoff vuoto in `create_session`/`update_handoff` | `argument 'handoff'/'content' must not be empty` |
| Passo di `plan` vuoto | `argument 'plan[i]' must not be empty` |
| `plan` non array di stringhe | `argument 'plan' must be an array of strings` |
| Nome task vuoto | `argument 'name' must not be empty` |
| Task inesistente | `task <id> not found` |
| `close_task` senza report | `missing required argument: report` / `argument 'report' must not be empty` |
| `close_task`/`invalidate_task` su task non `open` | `task <id> is not open (status: <status>)` |
| Stato task non valido in `list_tasks` | `invalid task status: <valore>` |
| Sessione senza handoff in `get_handoff` | `session <id> has no handoff` |
| Task senza report in `get_task_report` | `task <id> has no report` |

---

## Esempio di flusso completo

```bash
# 1. Crea una sessione
echo '{"jsonrpc":"2.0","id":1,"method":"tools/call","params":{"name":"create_session","arguments":{"name":"Migrazione DB","started_at":"2026-11-20T08:00:00+01:00"}}}' | ./builds/nxm-tracker

# 2. Aggiorna stato e design phase
echo '{"jsonrpc":"2.0","id":2,"method":"tools/call","params":{"name":"update_session","arguments":{"id":1,"status":"paused","design_phase_complete":true}}}' | ./builds/nxm-tracker

# 3. Crea una feature session
echo '{"jsonrpc":"2.0","id":3,"method":"tools/call","params":{"name":"create_feature_session","arguments":{"session_id":1,"name":"Script SQL","description":"Script di migrazione"}}}' | ./builds/nxm-tracker

# 4. Chiudi la sessione
echo '{"jsonrpc":"2.0","id":4,"method":"tools/call","params":{"name":"close_session","arguments":{"id":1}}}' | ./builds/nxm-tracker
```

### Flusso con handoff, piano e task (v1.1.0)

```bash
# 1. Crea sessione con handoff e piano
echo '{"jsonrpc":"2.0","id":1,"method":"tools/call","params":{"name":"create_session","arguments":{"name":"Migrazione DB","started_at":"2026-11-22T08:00:00+01:00","handoff":"Design approvato","plan":["setup schema","test","deploy"]}}}' | ./builds/nxm-tracker

# 2. Aggiorna l'handoff durante il lavoro
echo '{"jsonrpc":"2.0","id":2,"method":"tools/call","params":{"name":"update_handoff","arguments":{"session_id":1,"content":"Task setup schema chiuso"}}}' | ./builds/nxm-tracker

# 3. Crea i task di esecuzione (nome = branch)
echo '{"jsonrpc":"2.0","id":3,"method":"tools/call","params":{"name":"create_task","arguments":{"session_id":1,"name":"task/setup-schema"}}}' | ./builds/nxm-tracker

# 4. Chiudi il task con report finale obbligatorio
echo '{"jsonrpc":"2.0","id":4,"method":"tools/call","params":{"name":"close_task","arguments":{"id":1,"report":"Schema migrato, test verdi"}}}' | ./builds/nxm-tracker

# 5. Invalida un task non più utile
echo '{"jsonrpc":"2.0","id":5,"method":"tools/call","params":{"name":"invalidate_task","arguments":{"id":2,"reason":"superato dal task 1"}}}' | ./builds/nxm-tracker
```

> Nota: negli esempi sopra ogni richiesta avvia una nuova istanza del server. In un client MCP reale, il server rimane attivo e si inviano multiple richieste sulla stessa sessione stdio.
