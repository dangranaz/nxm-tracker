//! Letture dei task/todo versionati.
//!
//! Definisce `get_task`, `list_session_tasks`, `list_open_tasks`,
//! `TaskView` e `TaskReadError`.

pub mod error;
pub mod get_task;
pub mod list_open_tasks;
pub mod list_session_tasks;
pub mod view;

pub use self::error::TaskReadError;
pub use self::get_task::get_task;
pub use self::list_open_tasks::list_open_tasks;
pub use self::list_session_tasks::list_session_tasks;
pub use self::view::TaskView;
