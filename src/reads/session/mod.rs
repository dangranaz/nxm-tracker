//! Domain reads of sessions (`get_session`, `list_sessions`).
//!
//! Defines `SessionView` and `SessionReadError`. Read-only on the `session`
//! table: no writes, no mutations, no write transactions. Functions are
//! free on `pool`, not methods of `LifecycleRepo` (which reads only the
//! lifecycle catalog).
//!
//! The module is purely read-only: no audit rows, no INSERT/UPDATE on the
//! database.

pub mod error;
pub mod find_resumable;
pub mod get_session;
pub mod list_sessions;
pub mod view;

pub use self::error::SessionReadError;
pub use self::find_resumable::{find_resumable_session, ResumableSession};
pub use self::get_session::get_session;
pub use self::list_sessions::list_sessions;
pub use self::view::SessionView;
