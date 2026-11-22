//! Scritture dei task/todo versionati.
//!
//! Definisce `create_task`, `revise_task`, `set_task_status`, `close_task`.
//! Tutte le operazioni sono atomiche (una transazione SQLite) e scrivono
//! contestualmente una riga in `task_audit`.

pub mod close_task;
pub mod create_task;
pub mod revise_task;
pub mod set_task_status;

pub use self::close_task::close_task;
pub use self::create_task::create_task;
pub use self::revise_task::revise_task;
pub use self::set_task_status::set_task_status;

use crate::writes::error::WriteError;

pub type TaskWriteError = WriteError;
