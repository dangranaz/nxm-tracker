//! Task model for `nxm-tracker`.
//!
//! After a work session is created, one or more [`Task`]s are created to
//! execute the activity plan and/or the session itself. A task's `name` is
//! also the name of its git branch. A task can be closed only with a final
//! [`TaskReport`] attached (1:1), or marked as invalid when it no longer has
//! value (e.g. superseded by the outcome of another task).

use crate::error::TrackerError;
use serde::de;
use serde::{Deserialize, Deserializer, Serialize, Serializer};

/// Lifecycle status of a task.
///
/// Transitions: `Open` → `Closed` (requires a final report) or
/// `Open` → `Invalid`. `Closed` and `Invalid` are terminal states.
///
/// Serializes as lowercase string (`"open"`, `"closed"`, `"invalid"`) to match
/// the database storage format.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TaskStatus {
    /// Task is pending execution.
    Open,
    /// Task finished successfully; a final report is attached (1:1).
    Closed,
    /// Task no longer has value (e.g. superseded by another task's outcome).
    Invalid,
}

impl Serialize for TaskStatus {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        match self {
            TaskStatus::Open => serializer.serialize_str("open"),
            TaskStatus::Closed => serializer.serialize_str("closed"),
            TaskStatus::Invalid => serializer.serialize_str("invalid"),
        }
    }
}

impl<'de> Deserialize<'de> for TaskStatus {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        let s = String::deserialize(deserializer)?;
        TaskStatus::try_from(s.as_str()).map_err(de::Error::custom)
    }
}

impl std::fmt::Display for TaskStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            TaskStatus::Open => write!(f, "open"),
            TaskStatus::Closed => write!(f, "closed"),
            TaskStatus::Invalid => write!(f, "invalid"),
        }
    }
}

impl From<TaskStatus> for String {
    fn from(s: TaskStatus) -> Self {
        s.to_string()
    }
}

impl TryFrom<&str> for TaskStatus {
    type Error = TrackerError;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        match value {
            "open" => Ok(TaskStatus::Open),
            "closed" => Ok(TaskStatus::Closed),
            "invalid" => Ok(TaskStatus::Invalid),
            other => Err(TrackerError::InvalidTaskStatus(other.to_string())),
        }
    }
}

/// An execution task of a work session (plan step and/or session work).
///
/// `name` is also the git branch name of the task.
///
/// # Example
///
/// ```
/// use nxm_tracker::task::{Task, TaskStatus};
///
/// let task = Task {
///     id: None,
///     session_id: 1,
///     name: "task/setup-db".to_string(),
///     status: TaskStatus::Open,
///     note: None,
///     created_at: "2026-11-22T09:00:00+01:00".to_string(),
///     closed_at: None,
/// };
/// assert_eq!(task.status, TaskStatus::Open);
/// ```
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Task {
    pub id: Option<i64>,
    pub session_id: i64,
    pub name: String,
    pub status: TaskStatus,
    pub note: Option<String>,
    pub created_at: String,
    pub closed_at: Option<String>,
}

/// The final report of a task, attached 1:1 when the task is closed.
///
/// # Example
///
/// ```
/// use nxm_tracker::task::TaskReport;
///
/// let report = TaskReport {
///     id: None,
///     task_id: 7,
///     content: "All migrations applied, tests green.".to_string(),
///     created_at: "2026-11-22T18:00:00+01:00".to_string(),
/// };
/// assert_eq!(report.task_id, 7);
/// ```
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TaskReport {
    pub id: Option<i64>,
    pub task_id: i64,
    pub content: String,
    pub created_at: String,
}

/// Input data for creating a new task.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NewTask {
    pub session_id: i64,
    /// Task name; also used as the git branch name of the task.
    pub name: String,
}

/// Abstraction over persistence of tasks and their final reports.
///
/// The bundled implementation is
/// [`crate::session_repository::SqliteSessionRepository`], which implements
/// this trait on the `nxm-db` SQLite pool.
pub trait TaskRepository: Send + Sync {
    /// Inserts a new task in status `open` and returns it with its assigned ID.
    fn create_task(&self, new: &NewTask) -> Result<Task, TrackerError>;

    /// Returns the task with the given ID, or `None` if not found.
    fn get_task(&self, id: i64) -> Result<Option<Task>, TrackerError>;

    /// Returns all tasks of a session, ordered by `created_at` descending.
    fn list_tasks(&self, session_id: i64) -> Result<Vec<Task>, TrackerError>;

    /// Returns the tasks of a session with the given status.
    fn list_tasks_by_status(
        &self,
        session_id: i64,
        status: TaskStatus,
    ) -> Result<Vec<Task>, TrackerError>;

    /// Closes an `open` task attaching its final `report` (1:1) and setting
    /// `closed_at` to now. Returns the updated task.
    ///
    /// Returns `Ok(None)` if the task does not exist and
    /// [`TrackerError::TaskNotOpen`] if the task is not in status `open`.
    fn close_task(&self, id: i64, report: &str) -> Result<Option<Task>, TrackerError>;

    /// Marks an `open` task as `invalid`, optionally recording a `reason` in
    /// the task note. Returns the updated task, `Ok(None)` if not found, or
    /// [`TrackerError::TaskNotOpen`] if the task is not `open`.
    fn invalidate_task(&self, id: i64, reason: Option<&str>) -> Result<Option<Task>, TrackerError>;

    /// Returns the final report of a task, or `None` if the task has none.
    fn get_task_report(&self, task_id: i64) -> Result<Option<TaskReport>, TrackerError>;
}
