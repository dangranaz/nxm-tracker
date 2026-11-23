//! MCP tools for handoffs, activity plans, and execution tasks.
//!
//! These tools extend the session model: after `create_session` (which can
//! already store the 1:1 handoff and the activity plan), clients manage the
//! handoff text and create one or more execution tasks. A task is named after
//! its git branch and can be closed only with a final report, or marked as
//! invalid when it no longer has value.

use std::sync::Arc;

use crate::handoff::HandoffRepository;
use crate::mcp_tools::{
    get_id, get_opt_str, get_str, impl_mcp_tool, schema_object, text_result, validate_non_empty,
    validate_session_exists,
};
use crate::session_repository::SqliteSessionRepository;
use crate::task::{NewTask, TaskRepository, TaskStatus};
use nxm_mcp_server::mcp::tool::McpTool;
use nxm_mcp_server::server::ServerContext;
use serde_json::{json, Value};

// ---------------------------------------------------------------------------
// GetHandoff
// ---------------------------------------------------------------------------

/// Returns the 1:1 handoff of a work session.
pub struct GetHandoffTool {
    repo: Arc<SqliteSessionRepository>,
}

impl GetHandoffTool {
    pub fn new(repo: Arc<SqliteSessionRepository>) -> Self {
        Self { repo }
    }
}

impl_mcp_tool!(
    GetHandoffTool,
    "get_handoff",
    schema_object(
        json!({
            "session_id": { "type": "integer", "description": "Work session ID" }
        }),
        vec!["session_id"]
    )
);

impl GetHandoffTool {
    pub fn execute(&self, args: &Value) -> Result<Value, String> {
        let session_id = get_id(args, "session_id")?;
        validate_session_exists(&self.repo, session_id)?;

        match self
            .repo
            .get_handoff(session_id)
            .map_err(|e| format!("{e}"))?
        {
            Some(handoff) => Ok(text_result(serde_json::to_string(&handoff).unwrap())),
            None => Err(format!("session {session_id} has no handoff")),
        }
    }
}

// ---------------------------------------------------------------------------
// UpdateHandoff
// ---------------------------------------------------------------------------

/// Creates or replaces the 1:1 handoff of a work session (upsert).
pub struct UpdateHandoffTool {
    repo: Arc<SqliteSessionRepository>,
}

impl UpdateHandoffTool {
    pub fn new(repo: Arc<SqliteSessionRepository>) -> Self {
        Self { repo }
    }
}

impl_mcp_tool!(
    UpdateHandoffTool,
    "update_handoff",
    schema_object(
        json!({
            "session_id": { "type": "integer", "description": "Work session ID" },
            "content": { "type": "string", "description": "New handoff content (replaces the previous one)" }
        }),
        vec!["session_id", "content"]
    )
);

impl UpdateHandoffTool {
    pub fn execute(&self, args: &Value) -> Result<Value, String> {
        let session_id = get_id(args, "session_id")?;
        let content = get_str(args, "content")?;

        validate_session_exists(&self.repo, session_id)?;
        validate_non_empty(&content, "content")?;

        let handoff = self
            .repo
            .upsert_handoff(session_id, &content)
            .map_err(|e| format!("failed to update handoff: {e}"))?;

        Ok(text_result(serde_json::to_string(&handoff).unwrap()))
    }
}

// ---------------------------------------------------------------------------
// ListActivityPlans
// ---------------------------------------------------------------------------

/// Lists the activity-plan items of a work session, ordered by position.
pub struct ListActivityPlansTool {
    repo: Arc<SqliteSessionRepository>,
}

impl ListActivityPlansTool {
    pub fn new(repo: Arc<SqliteSessionRepository>) -> Self {
        Self { repo }
    }
}

impl_mcp_tool!(
    ListActivityPlansTool,
    "list_activity_plans",
    schema_object(
        json!({
            "session_id": { "type": "integer", "description": "Work session ID" }
        }),
        vec!["session_id"]
    )
);

impl ListActivityPlansTool {
    pub fn execute(&self, args: &Value) -> Result<Value, String> {
        let session_id = get_id(args, "session_id")?;
        validate_session_exists(&self.repo, session_id)?;

        let plans = self
            .repo
            .list_activity_plans(session_id)
            .map_err(|e| format!("{e}"))?;

        Ok(text_result(serde_json::to_string(&plans).unwrap()))
    }
}

// ---------------------------------------------------------------------------
// CreateTask
// ---------------------------------------------------------------------------

/// Creates an execution task for a work session (status starts as `open`).
/// The task `name` is also the git branch name of the task.
pub struct CreateTaskTool {
    repo: Arc<SqliteSessionRepository>,
}

impl CreateTaskTool {
    pub fn new(repo: Arc<SqliteSessionRepository>) -> Self {
        Self { repo }
    }
}

impl_mcp_tool!(
    CreateTaskTool,
    "create_task",
    schema_object(
        json!({
            "session_id": { "type": "integer", "description": "Work session ID" },
            "name": { "type": "string", "description": "Task name; also used as the task's git branch name (e.g. \"task/setup-db\")" }
        }),
        vec!["session_id", "name"]
    )
);

impl CreateTaskTool {
    pub fn execute(&self, args: &Value) -> Result<Value, String> {
        let session_id = get_id(args, "session_id")?;
        let name = get_str(args, "name")?;

        validate_session_exists(&self.repo, session_id)?;
        validate_non_empty(&name, "name")?;

        let new = NewTask { session_id, name };
        let task = self
            .repo
            .create_task(&new)
            .map_err(|e| format!("failed to create task: {e}"))?;

        Ok(text_result(serde_json::to_string(&task).unwrap()))
    }
}

// ---------------------------------------------------------------------------
// ListTasks
// ---------------------------------------------------------------------------

/// Lists the tasks of a work session, optionally filtered by status.
pub struct ListTasksTool {
    repo: Arc<SqliteSessionRepository>,
}

impl ListTasksTool {
    pub fn new(repo: Arc<SqliteSessionRepository>) -> Self {
        Self { repo }
    }
}

impl_mcp_tool!(
    ListTasksTool,
    "list_tasks",
    schema_object(
        json!({
            "session_id": { "type": "integer", "description": "Work session ID" },
            "status": { "type": "string", "description": "Filter by status: open, closed, invalid (optional)" }
        }),
        vec!["session_id"]
    )
);

impl ListTasksTool {
    pub fn execute(&self, args: &Value) -> Result<Value, String> {
        let session_id = get_id(args, "session_id")?;
        let status = get_opt_str(args, "status")
            .map(|s| TaskStatus::try_from(s.as_str()).map_err(|e| format!("{e}")))
            .transpose()?;

        validate_session_exists(&self.repo, session_id)?;

        let tasks = match status {
            Some(s) => self
                .repo
                .list_tasks_by_status(session_id, s)
                .map_err(|e| format!("{e}"))?,
            None => self
                .repo
                .list_tasks(session_id)
                .map_err(|e| format!("{e}"))?,
        };

        Ok(text_result(serde_json::to_string(&tasks).unwrap()))
    }
}

// ---------------------------------------------------------------------------
// GetTask
// ---------------------------------------------------------------------------

/// Returns a task by ID.
pub struct GetTaskTool {
    repo: Arc<SqliteSessionRepository>,
}

impl GetTaskTool {
    pub fn new(repo: Arc<SqliteSessionRepository>) -> Self {
        Self { repo }
    }
}

impl_mcp_tool!(
    GetTaskTool,
    "get_task",
    schema_object(
        json!({
            "id": { "type": "integer", "description": "Task ID" }
        }),
        vec!["id"]
    )
);

impl GetTaskTool {
    pub fn execute(&self, args: &Value) -> Result<Value, String> {
        let id = get_id(args, "id")?;

        match self.repo.get_task(id).map_err(|e| format!("{e}"))? {
            Some(task) => Ok(text_result(serde_json::to_string(&task).unwrap())),
            None => Err(format!("task {id} not found")),
        }
    }
}

// ---------------------------------------------------------------------------
// CloseTask
// ---------------------------------------------------------------------------

/// Closes an `open` task attaching its mandatory final report (1:1).
pub struct CloseTaskTool {
    repo: Arc<SqliteSessionRepository>,
}

impl CloseTaskTool {
    pub fn new(repo: Arc<SqliteSessionRepository>) -> Self {
        Self { repo }
    }
}

impl_mcp_tool!(
    CloseTaskTool,
    "close_task",
    schema_object(
        json!({
            "id": { "type": "integer", "description": "Task ID" },
            "report": { "type": "string", "description": "Final report of the task (required to close it)" }
        }),
        vec!["id", "report"]
    )
);

impl CloseTaskTool {
    pub fn execute(&self, args: &Value) -> Result<Value, String> {
        let id = get_id(args, "id")?;
        let report = get_str(args, "report")?;

        validate_non_empty(&report, "report")?;

        match self
            .repo
            .close_task(id, &report)
            .map_err(|e| format!("{e}"))?
        {
            Some(task) => Ok(text_result(serde_json::to_string(&task).unwrap())),
            None => Err(format!("task {id} not found")),
        }
    }
}

// ---------------------------------------------------------------------------
// InvalidateTask
// ---------------------------------------------------------------------------

/// Marks an `open` task as `invalid` (e.g. it no longer has value after
/// another task's outcome). An optional `reason` is stored in the task note.
pub struct InvalidateTaskTool {
    repo: Arc<SqliteSessionRepository>,
}

impl InvalidateTaskTool {
    pub fn new(repo: Arc<SqliteSessionRepository>) -> Self {
        Self { repo }
    }
}

impl_mcp_tool!(
    InvalidateTaskTool,
    "invalidate_task",
    schema_object(
        json!({
            "id": { "type": "integer", "description": "Task ID" },
            "reason": { "type": "string", "description": "Why the task is no longer valid (optional)" }
        }),
        vec!["id"]
    )
);

impl InvalidateTaskTool {
    pub fn execute(&self, args: &Value) -> Result<Value, String> {
        let id = get_id(args, "id")?;
        let reason = get_opt_str(args, "reason");

        match self
            .repo
            .invalidate_task(id, reason.as_deref())
            .map_err(|e| format!("{e}"))?
        {
            Some(task) => Ok(text_result(serde_json::to_string(&task).unwrap())),
            None => Err(format!("task {id} not found")),
        }
    }
}

// ---------------------------------------------------------------------------
// GetTaskReport
// ---------------------------------------------------------------------------

/// Returns the final report attached to a task (1:1, present once closed).
pub struct GetTaskReportTool {
    repo: Arc<SqliteSessionRepository>,
}

impl GetTaskReportTool {
    pub fn new(repo: Arc<SqliteSessionRepository>) -> Self {
        Self { repo }
    }
}

impl_mcp_tool!(
    GetTaskReportTool,
    "get_task_report",
    schema_object(
        json!({
            "task_id": { "type": "integer", "description": "Task ID" }
        }),
        vec!["task_id"]
    )
);

impl GetTaskReportTool {
    pub fn execute(&self, args: &Value) -> Result<Value, String> {
        let task_id = get_id(args, "task_id")?;

        if self
            .repo
            .get_task(task_id)
            .map_err(|e| format!("{e}"))?
            .is_none()
        {
            return Err(format!("task {task_id} not found"));
        }

        match self
            .repo
            .get_task_report(task_id)
            .map_err(|e| format!("{e}"))?
        {
            Some(report) => Ok(text_result(serde_json::to_string(&report).unwrap())),
            None => Err(format!("task {task_id} has no report")),
        }
    }
}
