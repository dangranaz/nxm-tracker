# Security Policy

## Versioni supportate

Le seguenti versioni ricevono aggiornamenti di sicurezza:

| Versione | Supportata |
|----------|------------|
| 0.7.1+   | ✅ Sì      |
| < 0.7.1  | ❌ No      |

## Segnalazione di una vulnerabilità

Se scopri una vulnerabilità di sicurezza in `nxm-tracker`, segnalala prima di renderla pubblica.

### Come segnalarla

Apri una issue privata su GitHub oppure invia un'e-mail al maintainer con i seguenti dettagli:

- Descrizione della vulnerabilità
- Passi per riprodurla (input, argomenti MCP tool, configurazione)
- Versione del crate interessata (`Cargo.toml` `version`)
- Impatto potenziale (es. esecuzione codice, esposizione dati, denial of service)
- Eventuale fix o workaround suggerito

### Cosa non fare

- Non aprire issue pubbliche per vulnerabilità di sicurezza prima di aver ricevuto una risposta dal maintainer.
- Non divulgare la vulnerabilità su social media, forum o altri canali pubblici prima del patch.

## Policy di disclosure

| Fase | Tempistica prevista |
|------|---------------------|
| Conferma ricezione report | entro 7 giorni |
| Valutazione e classificazione | entro 14 giorni |
| Rilascio patch o workaround | entro 30 giorni (o accordo diverso con il reporter) |
| Pubblicazione advisory | dopo il rilascio del patch, con credito al reporter se desiderato |

## Best practice di sicurezza adottate

Questa sezione riassume le scelte di sicurezza già presenti nel crate, coerenti con le regole in `.pi/rules/security.md`.

### Input validation

Tutti gli argumenti dei tool MCP vengono validati prima dell'uso:

- `id`, `session_id` devono essere interi validi (`i64`)
- `name` non può essere vuoto
- `status` deve essere uno tra `open`, `paused`, `closed` (tramite `SessionStatus::try_from`)
- `design_phase_complete` deve essere booleano

### Gestione dei segreti

- Non sono memorizzati segreti (password, token, chiavi) nel codice sorgente né nel database.
- La configurazione viene letta da `~/.nxmconfig` (file di proprietà dell'utente).
- I log sono scritti in JSON su `~/.nxmconfig/logs/nxm-tracker.log` e stderr; non includono dati sensibili per default.

### Audit delle dipendenze

Il progetto esegue come parte del `just check`:

- `cargo audit` — verifica vulnerabilità note nelle dipendenze
- `cargo deny check` — verifica licenze e policy sulle dipendenze

### Codice unsafe

Il crate non utilizza blocchi `unsafe`. Tutte le operazioni sono in safe Rust.

### Protezione dei dati

- Il database è SQLite locale (`~/.nxm/shared-db`), non esposto su rete direttamente.
- Il trasporto MCP predefinito è stdio (JSON-RPC su input/output standard).
- Se usato in modalità HTTP/HTTPS, il trasporto è gestito da `nxm-mcp-server` con le sue proprie garanzie di trasporto.

## MCP Tools e sicurezza

I otto tool MCP esposti (`create_session`, `get_session`, `list_sessions`, `update_session`, `close_session`, `set_design_phase_complete`, `create_feature_session`, `list_feature_sessions`) restituiscono sempre un envelope con `isError` in caso di fallimento, mai un errore di protocollo JSON-RPC. Gli errori contengono messaggi descrittivi ma non espongono stack trace interni né dettagli del filesystem.

## Riferimenti

- Regole di sicurezza del progetto: `.pi/rules/security.md` (source of truth: `~/AiProjects/nexum/docs/configRust/rules/security.md`)
- Licenza: PolyForm Noncommercial 1.0.0 (`LICENSE.md`)
