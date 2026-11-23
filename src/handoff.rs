//! Handoff and activity-plan model for `nxm-tracker`.
//!
//! Every work session has exactly one [`Handoff`] (1:1, frequently updated)
//! holding the precise handoff description, and an ordered list of
//! [`ActivityPlan`] items describing the plan of activities. Both are created
//! together with the session by `create_session` and persisted via the
//! [`HandoffRepository`] trait.

use crate::error::TrackerError;
use serde::{Deserialize, Serialize};

/// The handoff document of a work session (1:1 with the session).
///
/// The content is meant to be updated frequently as the session progresses,
/// so it is stored as free-form text with an `updated_at` timestamp.
///
/// # Example
///
/// ```
/// use nxm_tracker::handoff::Handoff;
///
/// let handoff = Handoff {
///     id: None,
///     session_id: 1,
///     content: "Design phase complete, see docs/architecture.md".to_string(),
///     updated_at: "2026-11-22T09:00:00+01:00".to_string(),
/// };
/// assert_eq!(handoff.session_id, 1);
/// ```
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Handoff {
    pub id: Option<i64>,
    pub session_id: i64,
    pub content: String,
    pub updated_at: String,
}

/// A single step of the activity plan of a work session.
///
/// `position` is the 1-based order of the step inside the plan.
///
/// # Example
///
/// ```
/// use nxm_tracker::handoff::ActivityPlan;
///
/// let step = ActivityPlan {
///     id: None,
///     session_id: 1,
///     position: 1,
///     description: "Set up the database schema".to_string(),
///     created_at: "2026-11-22T09:00:00+01:00".to_string(),
/// };
/// assert_eq!(step.position, 1);
/// ```
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ActivityPlan {
    pub id: Option<i64>,
    pub session_id: i64,
    pub position: i64,
    pub description: String,
    pub created_at: String,
}

/// Abstraction over persistence of handoffs and activity plans.
///
/// The bundled implementation is
/// [`crate::handoff_repository::SqliteHandoffRepository`]-style: see
/// [`crate::session_repository::SqliteSessionRepository`], which implements
/// this trait on the `nxm-db` SQLite pool.
pub trait HandoffRepository: Send + Sync {
    /// Returns the handoff of a session, or `None` if the session has none.
    fn get_handoff(&self, session_id: i64) -> Result<Option<Handoff>, TrackerError>;

    /// Creates or replaces the handoff of a session (upsert, 1:1) and returns
    /// the stored row with its `updated_at` timestamp refreshed.
    fn upsert_handoff(&self, session_id: i64, content: &str) -> Result<Handoff, TrackerError>;

    /// Returns the activity-plan items of a session, ordered by `position`.
    fn list_activity_plans(&self, session_id: i64) -> Result<Vec<ActivityPlan>, TrackerError>;
}
