# Sicurezza — Enterprise Edition

L'edizione Enterprise implementa controlli di sicurezza avanzati rispetto
all'edizione pubblica, incentrati su **audit trail**, **attribuzione
obbligatoria** e **retention controllata**.

## Audit trail completo

Ogni operazione di scrittura sul lifecycle genera una riga immutabile in
`audit_log` con i seguenti campi di attribuzione:

- `user`: identità utente che ha richiesto l'azione
- `auth_method`: metodo di autenticazione (attualmente `local`)
- `ai_tool`: strumento AI che ha eseguito l'azione
- `ai_model`: modello AI usato (opzionale)
- `at`: timestamp RFC3339 dell'azione

Le tabelle `task`, `task_revision`, `task_audit`, `session_handoff` seguono
lo stesso principio di append-only: nessuna cancellazione o modifica di
righe esistenti.

## Attribuzione obbligatoria (`ActionContext`)

Tutti gli strumenti di scrittura (`create_session`, `transition_session`,
`update_session`, `mark_consumed`, `create_task`, `revise_task`,
`set_task_status`, `close_task`) richiedono un oggetto `context` con:

| Campo | Tipo | Obbligatorio | Descrizione |
|-------|------|--------------|-------------|
| `user` | string | Sì | Utente richiedente |
| `ai_tool` | string | Sì | Strumento AI esecutore |
| `ai_model` | string | No | Modello AI usato |
| `auth_method` | string | No (default `local`) | Metodo autenticazione |

Senza `user` o `ai_tool` il tool restituisce errore e nessuna scrittura
viene eseguita.

## Retention dei log di audit

La tabella `retention_log` traccia permanentemente ogni operazione di
purga dell'audit log. Non è mai soggetta a retention stessa.

Configurazione tramite `nxm-config`:
- `retention_days`: numero di giorni oltre cui le righe `audit_log`
  vengono purgate (disabilitato se assente o <= 0)

Ogni purga scrive una riga in `retention_log` con `purged_count`,
`older_than` e timestamp `at`.

## Chiavi di sessione slug-based

A differenza dell'edizione pubblica (id INTEGER AUTOINCREMENT), l'edizione
EE usa ID di sessione basati su slug (es. `refactoring-q4-001`). Il
contatore `slug_counter` garantisce che le chiavi mintate non vengano mai
riutilizzate, anche dopo archiviazione o rimozione.

## Conformità

- Nessun dato sensibile viene memorizzato in chiaro oltre a quanto
  esplicitamente fornito dall'utente nei campi di testo libero.
- Le chiavi API e i segreti NON devono essere inseriti nei campi `title`,
  `body`, `goal_next`, `decisions`, ecc.
- L'accesso ai dati è gestito a livello di trasporto (stdio MCP); il
  crate non implementa autenticazione propria oltre al campo `auth_method`.
