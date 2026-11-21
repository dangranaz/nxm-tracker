//! Session model for `nxm-tracker`.
//!
//! Defines the domain types for long, multi-day work sessions and the
//! integrated feature sessions created after the design phase, plus the
//! [`SessionRepository`] trait that abstracts persistence.

use crate::error::TrackerError;
use serde::{Deserialize, Serialize};

/// Lifecycle status of a work session.
///
/// Transitions: `Open` → `Paused` → `Open` (resume) or `Open` → `Closed`.
/// A `Closed` session cannot be reopened.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum SessionStatus {
    /// Session is actively being worked on.
    Open,
    /// Session is temporarily suspended.
    Paused,
    /// Session is finished and archived.
    Closed,
}

impl std::fmt::Display for SessionStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            SessionStatus::Open => write!(f, "open"),
            SessionStatus::Paused => write!(f, "paused"),
            SessionStatus::Closed => write!(f, "closed"),
        }
    }
}

impl From<SessionStatus> for String {
    fn from(s: SessionStatus) -> Self {
        s.to_string()
    }
}

impl TryFrom<&str> for SessionStatus {
    type Error = TrackerError;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        match value {
            "open" => Ok(SessionStatus::Open),
            "paused" => Ok(SessionStatus::Paused),
            "closed" => Ok(SessionStatus::Closed),
            other => Err(TrackerError::InvalidSessionStatus(other.to_string())),
        }
    }
}

/// A long, multi-day work session.
///
/// # Example
///
/// ```
/// use nxm_tracker::session::{Session, SessionStatus};
///
/// let session = Session {
///     id: None,
///     name: "Q4 refactoring".to_string(),
///     started_at: "2026-11-20T08:00:00+01:00".to_string(),
///     ended_at: None,
///     status: SessionStatus::Open,
///     design_phase_complete: false,
/// };
/// assert_eq!(session.status, SessionStatus::Open);
/// ```
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Session {
    pub id: Option<i64>,
    pub name: String,
    pub started_at: String,
    pub ended_at: Option<String>,
    pub status: SessionStatus,
    pub design_phase_complete: bool,
}

/// An integrated feature session created after the design phase of a work
/// session.
///
/// # Example
///
/// ```
/// use nxm_tracker::session::FeatureSession;
///
/// let feature = FeatureSession {
///     id: None,
///     session_id: 1,
///     name: "Implement OAuth".to_string(),
///     description: Some("Add OIDC login flow".to_string()),
///     status: "open".to_string(),
///     created_at: "2026-11-20T08:00:00+01:00".to_string(),
/// };
/// assert_eq!(feature.session_id, 1);
/// ```
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FeatureSession {
    pub id: Option<i64>,
    pub session_id: i64,
    pub name: String,
    pub description: Option<String>,
    pub status: String,
    pub created_at: String,
}

/// Input data for creating a new work session.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NewSession {
    pub name: String,
    pub started_at: String,
}

/// Input data for creating a new feature session.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NewFeatureSession {
    pub session_id: i64,
    pub name: String,
    pub description: Option<String>,
}

/// Abstraction over persistence of sessions and feature sessions.
///
/// Products implement this trait for their chosen backend. The bundled
/// implementation is [`crate::session_repository::SqliteSessionRepository`].
pub trait SessionRepository: Send + Sync {
    /// Inserts a new session and returns it with its assigned ID.
    fn create_session(&self, new: &NewSession) -> Result<Session, TrackerError>;

    /// Returns the session with the given ID, or `None` if not found.
    fn get_session(&self, id: i64) -> Result<Option<Session>, TrackerError>;

    /// Returns all sessions, ordered by `started_at` descending.
    fn list_sessions(&self) -> Result<Vec<Session>, TrackerError>;

    /// Updates the status of a session. Returns `false` if not found.
    fn set_session_status(&self, id: i64, status: SessionStatus) -> Result<bool, TrackerError>;

    /// Marks a session as closed, setting `ended_at` to now.
    /// Returns `false` if not found.
    fn close_session(&self, id: i64) -> Result<bool, TrackerError>;

    /// Marks the design phase as complete (or incomplete) for a session.
    /// Returns `false` if not found.
    fn set_design_phase_complete(&self, id: i64, complete: bool) -> Result<bool, TrackerError>;

    /// Inserts a new feature session linked to a work session.
    fn create_feature_session(
        &self,
        new: &NewFeatureSession,
    ) -> Result<FeatureSession, TrackerError>;

    /// Returns all feature sessions for a work session, ordered by
    /// `created_at` descending.
    fn list_feature_sessions(&self, session_id: i64) -> Result<Vec<FeatureSession>, TrackerError>;
}
