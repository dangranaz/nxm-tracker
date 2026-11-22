//! `TaskView` — vista di lettura di una revisione di task.

use crate::domain::task_status::TaskStatus;

/// Vista di lettura di una task revision (versione corrente).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TaskView {
    pub session_id: String,
    pub task_id: i64,
    pub version: i64,
    pub status: TaskStatus,
    pub title: String,
    pub body: Option<String>,
    pub exec_order: i64,
    pub result: Option<String>,
    pub spawned_task_id: Option<i64>,
    pub blocked_by: Option<i64>,
    pub created_at: String,
    pub at: String,
}
