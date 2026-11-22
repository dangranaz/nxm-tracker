//! `nxm_tracker` — tracks long, multi-day work sessions and the integrated
//! feature sessions created after the design phase.
//!
//! Uses [`nxm-config`](nxm_config) for configuration management,
//! [`nxm-logging`](nxm_logging) for structured logging, and
//! [`nxm-db`](nxm_db) for datasource access.

mod config;
mod database;
mod error;
pub mod git_workspace;
mod logging;
mod mcp_tools;
mod session;
mod session_repository;

pub use config::TrackerConfig;
pub use database::{default_migration_set, Database};
pub use error::TrackerError;
pub use logging::init_logging;
pub use mcp_tools::{
    CloseSessionTool, CreateFeatureSessionTool, CreateSessionTool, GetSessionTool,
    ListFeatureSessionsTool, ListSessionsTool, SetDesignPhaseCompleteTool, UpdateSessionTool,
};
pub use session::{
    FeatureSession, NewFeatureSession, NewSession, Session, SessionRepository, SessionStatus,
};
pub use session_repository::SqliteSessionRepository;

/// Returns the canonical product name of this crate.
///
/// # Examples
///
/// ```
/// use nxm_tracker::product_name;
///
/// assert_eq!(product_name(), "nxm-tracker");
/// ```
pub fn product_name() -> &'static str {
    tracing::debug!(target: "nexum::tracker::api", "product name requested");
    "nxm-tracker"
}
