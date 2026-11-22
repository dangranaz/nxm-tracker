//! `TaskStatus` — gli stati canonici di un task/todo versionato.
//!
//! Stati: `open`, `in_progress`, `done`, `superseded`, `blocked`, `cancelled`.
//! Nessun I/O, nessun database: solo tipi e funzioni pure.

use std::convert::TryInto;
use std::fmt;

use thiserror::Error;

/// Status of a task (todo) within a session.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum TaskStatus {
    /// Task created, not yet started.
    Open,
    /// Task currently being worked on.
    InProgress,
    /// Task completed successfully.
    Done,
    /// Task replaced by a newer version.
    Superseded,
    /// Task blocked on a dependency.
    Blocked,
    /// Task cancelled without completion.
    Cancelled,
}

impl TaskStatus {
    /// Returns the lowercase string used for DB storage.
    pub fn as_str(self) -> &'static str {
        match self {
            TaskStatus::Open => "open",
            TaskStatus::InProgress => "in_progress",
            TaskStatus::Done => "done",
            TaskStatus::Superseded => "superseded",
            TaskStatus::Blocked => "blocked",
            TaskStatus::Cancelled => "cancelled",
        }
    }

    /// Is this a terminal state (no further transitions expected)?
    pub fn is_terminal(self) -> bool {
        matches!(self, TaskStatus::Done | TaskStatus::Cancelled)
    }
}

impl fmt::Display for TaskStatus {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_str())
    }
}

impl TryFrom<&str> for TaskStatus {
    type Error = ParseTaskStatusError;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        match value {
            "open" => Ok(TaskStatus::Open),
            "in_progress" => Ok(TaskStatus::InProgress),
            "done" => Ok(TaskStatus::Done),
            "superseded" => Ok(TaskStatus::Superseded),
            "blocked" => Ok(TaskStatus::Blocked),
            "cancelled" => Ok(TaskStatus::Cancelled),
            other => Err(ParseTaskStatusError(other.to_string())),
        }
    }
}

/// Error parsing a task status string.
#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
#[error("unknown task status: {0}")]
pub struct ParseTaskStatusError(pub String);

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn task_status_as_str_matches_db_values() {
        assert_eq!(TaskStatus::Open.as_str(), "open");
        assert_eq!(TaskStatus::InProgress.as_str(), "in_progress");
        assert_eq!(TaskStatus::Done.as_str(), "done");
        assert_eq!(TaskStatus::Superseded.as_str(), "superseded");
        assert_eq!(TaskStatus::Blocked.as_str(), "blocked");
        assert_eq!(TaskStatus::Cancelled.as_str(), "cancelled");
    }

    #[test]
    fn task_status_predicates() {
        assert!(!TaskStatus::Open.is_terminal());
        assert!(!TaskStatus::InProgress.is_terminal());
        assert!(TaskStatus::Done.is_terminal());
        assert!(!TaskStatus::Superseded.is_terminal());
        assert!(!TaskStatus::Blocked.is_terminal());
        assert!(TaskStatus::Cancelled.is_terminal());
    }

    #[test]
    fn task_status_try_from_string_roundtrip() {
        for s in [
            TaskStatus::Open,
            TaskStatus::InProgress,
            TaskStatus::Done,
            TaskStatus::Superseded,
            TaskStatus::Blocked,
            TaskStatus::Cancelled,
        ] {
            let parsed: TaskStatus = s.as_str().try_into().unwrap();
            assert_eq!(parsed, s);
        }
    }

    #[test]
    fn task_status_try_from_invalid_errors() {
        let err: ParseTaskStatusError = TryInto::<TaskStatus>::try_into("invalid").unwrap_err();
        assert!(err.to_string().contains("invalid"));
    }
}
