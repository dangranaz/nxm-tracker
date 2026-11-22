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

Il server registra i seguenti 8 tool:

| Tool | Descrizione |
|------|-------------|
| `create_session` | Crea una nuova sessione di lavoro |
| `get_session` | Legge una sessione per ID |
| `list_sessions` | Lista tutte le sessioni (ordinate per data discendente) |
| `update_session` | Aggiorna nome, stato o flag design phase di una sessione |
| `close_session` | Chiude una sessione (imposta `closed` e `ended_at`) |
| `set_design_phase_complete` | Imposta il flag `design_phase_complete` |
| `create_feature_session` | Crea una feature session collegata a una work session |
| `list_feature_sessions` | Lista le feature session di una work session |

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

### Esempio request

```json
{
  "name": "Refactoring Q4",
  "started_at": "2026-11-20T08:00:00+01:00"
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

## Stati delle sessioni

| Stato | Descrizione | Transizioni consentite |
|-------|-------------|----------------------|
| `open` | Sessione attivamente in lavorazione | → `paused`, → `closed` |
| `paused` | Sessione temporaneamente sospesa | → `open` (riprendi), → `closed` |
| `closed` | Sessione terminata e archiviata | Nessuna (non riapribile) |

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

> Nota: negli esempi sopra ogni richiesta avvia una nuova istanza del server. In un client MCP reale, il server rimane attivo e si inviano multiple richieste sulla stessa sessione stdio.
