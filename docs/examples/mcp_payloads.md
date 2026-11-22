# Esempi di chiamata MCP — `nxm-tracker`

Payload JSON request/response per tutti e 8 gli tool MCP esposti da `nxm-tracker`.

Ogni tool restituisce un envelope MCP standard:

```json
{
  "content": [{"type": "text", "text": "<JSON stringificato o messaggio testuale>"}],
  "isError": false
}
```

In caso di errore, `isError` è `true` e `text` contiene il messaggio descrittivo.

---

## 1. `create_session`

Crea una nuova sessione di lavoro.

**Request:**
```json
{
  "name": "Refactoring Q4",
  "started_at": "2026-11-20T08:00:00+01:00"
}
```

**Response (esempio):**
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

Restituisce una sessione per ID.

**Request:**
```json
{
  "id": 1
}
```

**Response (trovata):**
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

**Response (non trovata):**
```json
{
  "content": [
    {
      "type": "text",
      "text": "session 1 not found"
    }
  ],
  "isError": false
}
```

---

## 3. `list_sessions`

Lista tutte le sessioni (ordinate per `started_at` discendente).

**Request:**
```json
{}
```

**Response (esempio):**
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

Aggiorna uno o più campi di una sessione esistente (almeno uno tra `name`, `status`, `design_phase_complete`).

**Request (aggiorna nome e stato):**
```json
{
  "id": 1,
  "name": "Refactoring Q4 (riveduto)",
  "status": "paused"
}
```

**Request (aggiorna solo il flag design phase):**
```json
{
  "id": 1,
  "design_phase_complete": true
}
```

**Response (esempio):**
```json
{
  "content": [
    {
      "type": "text",
      "text": "{\"id\":1,\"name\":\"Refactoring Q4 (riveduto)\",\"started_at\":\"2026-11-20T08:00:00+01:00\",\"ended_at\":null,\"status\":\"paused\",\"design_phase_complete\":true}"
    }
  ],
  "isError": false
}
```

**Response (sessione non trovata):**
```json
{
  "content": [
    {
      "type": "text",
      "text": "session 9999 not found"
    }
  ],
  "isError": false
}
```

---

## 5. `close_session`

Chiude una sessione imposta `status = "closed"` e `ended_at` al timestamp corrente.

**Request:**
```json
{
  "id": 1
}
```

**Response (successo):**
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

**Response (non trovata):**
```json
{
  "content": [
    {
      "type": "text",
      "text": "session 9999 not found"
    }
  ],
  "isError": false
}
```

---

## 6. `set_design_phase_complete`

Imposta il flag `design_phase_complete` su una sessione.

**Request:**
```json
{
  "id": 1,
  "complete": true
}
```

**Response (successo):**
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

**Response (non trovata):**
```json
{
  "content": [
    {
      "type": "text",
      "text": "session 9999 not found"
    }
  ],
  "isError": false
}
```

---

## 7. `create_feature_session`

Crea una nuova feature session collegata a una work session esistente.

**Request:**
```json
{
  "session_id": 1,
  "name": "Implementa OAuth",
  "description": "Aggiungi flusso di login OpenID Connect"
}
```

**Request (senza descrizione):**
```json
{
  "session_id": 1,
  "name": "Aggiungi test"
}
```

**Response (esempio):**
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

Lista tutte le feature session di una work session (ordinate per `created_at` discendente).

**Request:**
```json
{
  "session_id": 1
}
```

**Response (esempio):**
```json
{
  "content": [
    {
      "type": "text",
      "text": "[{\"id\":1,\"session_id\":1,\"name\":\"Implementa OAuth\",\"description\":\"Aggiungi flusso di login OpenID Connect\",\"status\":\"open\",\"created_at\":\"2026-11-20T08:00:00+01:00\"},{\"id\":2,\"session_id\":1,\"name\":\"Aggiungi test\",\"description\":null,\"status\":\"open\",\"created_at\":\"2026-11-21T09:00:00+01:00\"}]"
    }
  ],
  "isError": false
}
```

**Response (nessuna feature session):**
```json
{
  "content": [
    {
      "type": "text",
      "text": "[]"
    }
  ],
  "isError": false
}
```

---

## Flusso completo esempio (cURL-style)

```bash
# 1. Crea sessione
echo '{"name":"Migrazione DB","started_at":"2026-11-20T08:00:00+01:00"}' | nxm-tracker create_session

# 2. Aggiorna stato e design phase
echo '{"id":1,"status":"paused","design_phase_complete":true}' | nxm-tracker update_session

# 3. Crea feature session
echo '{"session_id":1,"name":"Script SQL","description":"Script di migrazione"}' | nxm-tracker create_feature_session

# 4. Chiudi sessione
echo '{"id":1}' | nxm-tracker close_session
```

> Nota: `nxm-tracker` espone i tool via trasporto stdio (MCP). Gli esempi sopra mostrano i payload JSON da inviare e le risposte attese.
