//! Domain reads of handoff (`get_current_handoff`).
//!
//! Defines `HandoffView` and `HandoffReadError`. Read-only on the
//! `session_handoff` table: no writes, no mutations, no write
//! transactions.

pub mod error;
pub mod get_current_handoff;
pub mod list_handoff_revisions;
pub mod view;

pub use self::error::HandoffReadError;
pub use self::get_current_handoff::get_current_handoff;
pub use self::list_handoff_revisions::list_handoff_revisions;
pub use self::view::HandoffView;
