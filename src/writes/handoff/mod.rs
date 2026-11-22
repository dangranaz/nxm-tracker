//! Writes of the versioned handoff (`update_session`).
//!
//! Defines `update_session` (inserts a new revision + audit row,
//! atomic transaction) and `HandoffWriteError`.
//! The module is split to respect the 300-line/file limit.

pub mod error;
pub mod input;
pub mod update_session;

pub use self::error::HandoffWriteError;
pub use self::input::HandoffInput;
pub use self::update_session::update_session;
