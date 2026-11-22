//! Scritture del lifecycle (`create`, `transition`, `handoff`, `prompt`, `consume`).
//!
//! Tutte le operazioni di scrittura sono atomiche (una transazione SQLite)
//! e scrivono contestualmente una riga in `audit_log` con attribuzione
//! obbligatoria (`ActionContext`).
//!
//! Nessuna scrittura modifica direttamente il database senza passare per
//! queste funzioni.

pub mod consume;
pub mod create;
pub mod error;
pub mod prompt;
pub mod transition;

pub mod handoff;

pub use consume::mark_consumed;
pub use create::create_session;
pub use error::WriteError;
pub use handoff::{update_session, HandoffInput, HandoffWriteError};
pub use prompt::register_prompt;
pub use transition::transition_session;
