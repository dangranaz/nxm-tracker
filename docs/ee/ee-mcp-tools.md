# Tool MCP — Enterprise Edition

Oltre agli 8 tool dell'edizione pubblica, l'edizione EE espone **11 tool
aggiuntivi** per il ciclo di vita completo, l'audit e la gestione task.

## Tool di creazione e transizione sessione

### `create_session`
Crea una nuova sessione del lifecycle con stato `active` e relativo audit log.
Se `working_directory` è fornito, verifica che sia un git repository e crea
un branch `integration_<short>` da `develop`/`master`/`main`.

**Parametri:** `session_id` (string, slug-based), `name` (string), `started_at` (RFC3339), `working_directory` (string, opzionale), `context` (object con `user`, `ai_tool`, `ai_model` opzionale, `auth_method`).

### `transition_session`
Cambia lo stato di una sessione esistente seguendo le transizioni valide del
dominio. Inserisce una riga in `audit_log`.

**Parametri:** `session_id`, `to_state` (`active`/`onhold`/`abandoned`/`done`/`superseded`), `trigger` (`set_status`/`reopen`), `context`.

## Tool di handoff e consumo

### `update_session`
Inserisce una nuova revisione di handoff (versioned) e relativa riga di audit.

**Parametri:** `session_id`, `goal_next` (opzionale), `decisions` (array string), `next_steps` (array), `failed_approaches` (array), `relevant_files` (array), `context_usage_pct` (integer, opzionale), `context`.

### `mark_consumed`
Marca una sessione come consumata (`status = superseded`, imposta `consumed_at` e `consumed_by`). Se `require_human_signoff` è attivo nella configurazione, richiede una firma umana valida.

**Parametri:** `session_id`, `consumed_by` (string, opzionale), `context`.

## Tool di prompt registry

### `register_prompt`
Registra o aggiorna (upsert) il riferimento a un brief di delega per
`(session_id, task_no, lang)`. Non scrive audit log.

**Parametri:** `session_id`, `task_no`, `slug`, `lang` (`ita`/`eng`), `path`.

## Tool di task versionati

### `create_task`
Crea un nuovo task con prima revisione (versione 0).

**Parametri:** `session_id`, `task_id` (integer), `title`, `body` (opzionale), `exec_order` (integer), `context`.

### `revise_task`
Crea una nuova revisione di un task esistente (incrementa `version`).

**Parametri:** `session_id`, `task_id`, `title`, `body` (opzionale), `exec_order`, `context`.

### `set_task_status`
Cambia lo stato di un task (nuova revisione).

**Parametri:** `session_id`, `task_id`, `status` (`open`/`in_progress`/`done`/`superseded`/`blocked`/`cancelled`), `context`.

### `close_task`
Chiude un task impostando stato `cancelled`.

**Parametri:** `session_id`, `task_id`, `context`.

### `get_task`
Legge l'ultima revisione di un task.

**Parametri:** `session_id`, `task_id`.

### `list_session_tasks`
Lista tutte le revisioni correnti dei task di una sessione.

**Parametri:** `session_id`.

### `list_open_tasks`
Lista tutti i task non chiusi (`status != done` e `status != cancelled`).

**Parametri:** nessuno.

## Tool di catalogo lifecycle

### `list_lifecycle_states`
Lista tutti gli stati del ciclo di vita con relative proprietà.

**Parametri:** nessuno.

### `list_lifecycle_transitions`
Lista tutte le transizioni valide tra stati.

**Parametri:** nessuno.

## Envelope di risposta

Tutti i tool restituiscono l'envelope MCP standard:

```json
{
  "content": [{"type": "text", "text": "<payload>"}],
  "isError": false
}
```

In caso di errore: `isError: true` con messaggio descrittivo in `text`.

## Attribuzione obbligatoria

Tutti gli strumenti di scrittura richiedono un oggetto `context` con i campi
obbligatori `user` e `ai_tool`. Senza di essi il tool restituisce errore.
