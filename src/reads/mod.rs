//! Domain reads (read-only, no writes).
//!
//! Exposes read modules per domain: `session` for session reads
//! (`get_session`, `list_sessions`, `SessionView`, `SessionReadError`),
//! `handoff` for handoff reads (`get_current_handoff`, `HandoffView`,
//! `HandoffReadError`).

pub mod handoff;
pub mod session;
