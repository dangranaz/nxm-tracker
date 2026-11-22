# nxm-tracker Enterprise Edition (EE)

`nxm-tracker` Enterprise Edition è la variante privata della piattaforma di
tracciamento sessioni, destinata all'uso interno NXM (destinazione:
`granata.phoenix` / GitLab `nxmAiDevelop/nxmMcpServers/nxm-tracker`).

A differenza dell'edizione pubblica (`nxm-tracker` su GitHub `dangranaz`),
questa edizione include il **ciclo di vita completo delle sessioni** con audit
trail, task versionati, handoff registrato e retention dei log — portata dal
crate `nxm-mcp-tracker-ee` (`nxmHistory` monorepo).

## Differenze rispetto all'edizione pubblica

| Funzionalità | Public (master/develop) | Enterprise (ee) |
|--------------|------------------------|-----------------|
| Sessioni | `sessions` (id INTEGER) | `session` (id TEXT, lifecycle state) |
| Stati sessione | open / paused / closed | active / onhold / abandoned / done / superseded |
| Audit log | ❌ | ✅ (`audit_log` con attribuzione completa) |
| Task versionati | ❌ | ✅ (`task`, `task_revision`, `task_audit`) |
| Handoff versionato | ❌ | ✅ (`session_handoff`) |
| Slug counter | ❌ | ✅ (`slug_counter`) |
| Prompt registry | ❌ | ✅ (`prompt`) |
| Consume / mark consumed | ❌ | ✅ (`consumed_at`, `consumed_by`) |
| Retention log | ❌ | ✅ (`retention_log`) |
| Tool MCP totali | 8 | 19 |
| Migration totali | 2 | 15 |

## Stato del branch

Il branch `ee` contiene l'implementazione completa del porting da
`nxm-mcp-tracker-ee` (v0.4.0) adattata alle dipendenze locali di
`nxm-tracker` (`nxm-config`, `nxm-logging`, `nxm-db`, `nxm-mcp-server`).

## Accesso

Questa edizione è **privata** e non viene pubblicata su GitHub pubblica. La
sincronizzazione avviene verso il remote GitLab privato.
