//! Dominio puro del lifecycle delle sessioni (portato da `nxm-mcp-tracker-ee`).
//!
//! Definisce l'enum `Status` (5 stati canonici) con i suoi predicati, i tipi
//! valore `Transition`/`Trigger`, il contesto di azione `ActionContext` e
//! lo stato dei task `TaskStatus`. Nessun I/O, nessun database, nessun trait
//! di repository — solo tipi e funzioni pure testabili in isolamento.

pub mod action_context;
pub mod status;
pub mod task_status;
pub mod transition;

pub use action_context::{ActionContext, ActionContextError, AuthMethod, ParseAuthMethodError};
pub use status::{ParseStatusError, Status};
pub use task_status::{ParseTaskStatusError, TaskStatus};
pub use transition::{ParseTriggerError, Transition, Trigger};
