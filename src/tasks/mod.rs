//! Dominio, scritture e letture dei task/todo versionati (slice trackeree-tasks-01).
//! Contiene il dominio puro degli stati (`task_status`), le scritture
//! atomiche (`writes`) e le letture della revisione corrente (`reads`).
//! I tool MCP sono il brief successivo.

pub mod reads;
pub mod writes;

pub use self::reads::{get_task, list_open_tasks, list_session_tasks, TaskReadError, TaskView};
pub use self::writes::{close_task, create_task, revise_task, set_task_status, TaskWriteError};
