//! MCP tools for the new lifecycle schema (port from `nxm-mcp-tracker-ee`).
//!
//! Exposes the new domain: session lifecycle, audit log, versioned tasks,
//! handoff revisions, prompt registry, and consume.
//!
//! Each tool holds an [`Arc<crate::repo::SqliteRepo>`] for database access
//! and implements [`nxm_mcp_server::mcp::tool::McpTool`].

use std::sync::Arc;

use crate::domain::action_context::{ActionContext, AuthMethod};
use crate::domain::task_status::TaskStatus;
use crate::domain::transition::{is_transition_allowed, Trigger};
use crate::domain::Status;
use crate::repo::{LifecycleRepo, SqliteRepo};
use crate::writes::consume::mark_consumed;
use crate::writes::create::create_session;
use crate::writes::handoff::{update_session, HandoffInput};
use crate::writes::prompt::register_prompt;
use crate::writes::transition::transition_session;
use crate::writes::WriteError;
use nxm_mcp_server::mcp::tool::McpTool;
use nxm_mcp_server::server::ServerContext;
use serde_json::{json, Value};

/// Helper: build a JSON Schema object for tool input.
fn schema_object(properties: Value, required: Vec<&str>) -> Value {
    json!({
        "type": "object",
        "properties": properties,
        "required": required,
    })
}

/// Helper: extract a required string argument.
fn get_str(args: &Value, key: &str) -> Result<String, String> {
    match args.get(key) {
        Some(Value::String(s)) => Ok(s.clone()),
        Some(Value::Number(n)) => Ok(n.to_string()),
        Some(_) => Err(format!("argument '{key}' must be a string or number")),
        None => Err(format!("missing required argument: {key}")),
    }
}

/// Helper: extract an optional string argument.
fn get_opt_str(args: &Value, key: &str) -> Option<String> {
    args.get(key).and_then(|v| v.as_str()).map(|s| s.to_string())
}

/// Helper: extract an optional boolean argument.
fn get_opt_bool(args: &Value, key: &str) -> Option<bool> {
    args.get(key).and_then(|v| v.as_bool())
}

/// Helper: extract an optional i64 argument.
fn get_opt_i64(args: &Value, key: &str) -> Option<i64> {
    args.get(key).and_then(|v| v.as_i64())
}

/// Helper: extract an optional Vec<String> argument.
fn get_opt_str_vec(args: &Value, key: &str) -> Vec<String> {
    args.get(key)
        .and_then(|v| v.as_array())
        .map(|arr| {
            arr.iter()
                .filter_map(|v| v.as_str().map(|s| s.to_string()))
                .collect()
        })
        .unwrap_or_default()
}

/// Helper: validate that a string is non-empty.
fn validate_non_empty(value: &str, field: &str) -> Result<(), String> {
    if value.trim().is_empty() {
        Err(format!("argument '{field}' must not be empty"))
    } else {
        Ok(())
    }
}

/// Helper: build a text content result with isError=false.
fn text_result(text: String) -> Value {
    json!({
        "content": [{"type": "text", "text": text}],
        "isError": false,
    })
}

/// Helper: build a text content result with isError=true.
fn error_result(text: String) -> Value {
    json!({
        "content": [{"type": "text", "text": text}],
        "isError": true,
    })
}

/// Helper: extract ActionContext from MCP arguments.
/// Expects `context` object with `user`, `ai_tool`, optional `ai_model`.
fn extract_action_context(args: &Value) -> Result<ActionContext, String> {
    let ctx = args.get("context").and_then(|v| v.as_object()).ok_or_else(|| {
        "missing required argument: context".to_string()
    })?;
    let user = ctx
        .get("user")
        .and_then(|v| v.as_str())
        .ok_or_else(|| "missing required argument: context.user".to_string())?
        .to_string();
    let ai_tool = ctx
        .get("ai_tool")
        .and_then(|v| v.as_str())
        .ok_or_else(|| "missing required argument: context.ai_tool".to_string())?
        .to_string();
    let ai_model = ctx.get("ai_model").and_then(|v| v.as_str()).map(|s| s.to_string());
    let auth_method = AuthMethod::parse(
        ctx.get("auth_method")
            .and_then(|v| v.as_str())
            .unwrap_or("local"),
    )
    .map_err(|e| format!("invalid auth_method: {e}"))?;
    ActionContext::new(user, ai_tool, ai_model, auth_method).map_err(|e| e.to_string())
}

/// Helper: build a JSON result.
fn json_result(value: Value) -> Value {
    json!({
        "content": [{"type": "text", "text": value.to_string()}],
        "isError": false,
    })
}

// ============================================================================
// create_session (lifecycle)
// ============================================================================

/// Creates a new lifecycle session with git integration option.
pub struct CreateLifecycleSessionTool {
    repo: Arc<SqliteRepo>,
}

impl CreateLifecycleSessionTool {
    pub fn new(repo: Arc<SqliteRepo>) -> Self {
        Self { repo }
    }
}

impl McpTool for CreateLifecycleSessionTool {
    fn name(&self) -> &str {
        "create_session"
    }
    fn schema(&self) -> Value {
        schema_object(
            json!({
                "session_id": { "type": "string", "description": "Unique session ID (slug-based)" },
                "name": { "type": "string", "description": "Session name" },
                "started_at": { "type": "string", "description": "Start timestamp (RFC3339)" },
                "working_directory": { "type": "string", "description": "Path to a git project (optional). If set, an integration branch is created before the session." },
                "context": {
                    "type": "object",
                    "description": "Action context (attribution required)",
                    "properties": {
                        "user": { "type": "string", "description": "User requesting the action" },
                        "ai_tool": { "type": "string", "description": "AI tool executing the action" },
                        "ai_model": { "type": "string", "description": "AI model used (optional)" },
                        "auth_method": { "type": "string", "description": "Authentication method (default: local)" }
                    },
                    "required": ["user", "ai_tool"]
                }
            }),
            vec!["session_id", "name", "started_at", "context"]
        )
    }
    fn call(&self, _ctx: &ServerContext, args: &Value) -> Result<Value, String> {
        let session_id = get_str(args, "session_id")?;
        let name = get_str(args, "name")?;
        let started_at = get_str(args, "started_at")?;
        let working_directory = get_opt_str(args, "working_directory");
        let ctx = extract_action_context(args)?;

        validate_non_empty(&session_id, "session_id")?;
        validate_non_empty(&name, "name")?;
        chrono::DateTime::parse_from_rfc3339(&started_at)
            .map_err(|_| format!("argument 'started_at' must be a valid RFC3339 timestamp"))?;

        let status = create_session(
            self.repo.pool().map_err(|e| e.to_string())?,
            &session_id,
            &name,
            &ctx,
            working_directory.as_deref(),
            &started_at,
        )
        .map_err(|e| format!("{e}"))?;

        Ok(text_result(format!(
            "session {session_id} created with status {status}"
        )))
    }
}

// ============================================================================
// transition_session
// ============================================================================

/// Changes the status of a lifecycle session.
pub struct TransitionSessionTool {
    repo: Arc<SqliteRepo>,
}

impl TransitionSessionTool {
    pub fn new(repo: Arc<SqliteRepo>) -> Self {
        Self { repo }
    }
}

impl McpTool for TransitionSessionTool {
    fn name(&self) -> &str {
        "transition_session"
    }
    fn schema(&self) -> Value {
        schema_object(
            json!({
                "session_id": { "type": "string", "description": "Session ID" },
                "to_state": { "type": "string", "description": "New status: active, onhold, abandoned, done, superseded" },
                "trigger": { "type": "string", "description": "Trigger: set_status or reopen" },
                "context": {
                    "type": "object",
                    "description": "Action context (attribution required)",
                    "properties": {
                        "user": { "type": "string" },
                        "ai_tool": { "type": "string" },
                        "ai_model": { "type": "string" },
                        "auth_method": { "type": "string" }
                    },
                    "required": ["user", "ai_tool"]
                }
            }),
            vec!["session_id", "to_state", "trigger", "context"]
        )
    }
    fn call(&self, _ctx: &ServerContext, args: &Value) -> Result<Value, String> {
        let session_id = get_str(args, "session_id")?;
        let to_state_str = get_str(args, "to_state")?;
        let trigger_str = get_str(args, "trigger")?;
        let ctx = extract_action_context(args)?;

        let to_state = Status::try_from(to_state_str.as_str())
            .map_err(|e| format!("invalid status: {e}"))?;
        let trigger = Trigger::try_from(trigger_str.as_str())
            .map_err(|e| format!("invalid trigger: {e}"))?;

        let now = chrono::Utc::now().to_rfc3339();

        let status = transition_session(
            self.repo.pool().map_err(|e| e.to_string())?,
            &session_id,
            None,
            to_state,
            trigger,
            &ctx,
            &now,
        )
        .map_err(|e| format!("{e}"))?;

        Ok(text_result(format!(
            "session {session_id} transitioned to {status}"
        )))
    }
}

// ============================================================================
// update_session (handoff)
// ============================================================================

/// Updates a session handoff (versioned revision + audit row).
pub struct UpdateSessionTool {
    repo: Arc<SqliteRepo>,
}

impl UpdateSessionTool {
    pub fn new(repo: Arc<SqliteRepo>) -> Self {
        Self { repo }
    }
}

impl McpTool for UpdateSessionTool {
    fn name(&self) -> &str {
        "update_session"
    }
    fn schema(&self) -> Value {
        schema_object(
            json!({
                "session_id": { "type": "string", "description": "Session ID" },
                "goal_next": { "type": "string", "description": "Goal for the next work session (optional)" },
                "decisions": { "type": "array", "description": "Decisions made (array of strings)" },
                "next_steps": { "type": "array", "description": "Next steps (array of strings)" },
                "failed_approaches": { "type": "array", "description": "Failed approaches (array of strings)" },
                "relevant_files": { "type": "array", "description": "Relevant files (array of strings)" },
                "context_usage_pct": { "type": "integer", "description": "Context usage percentage (optional)" },
                "context": {
                    "type": "object",
                    "description": "Action context (attribution required)",
                    "properties": {
                        "user": { "type": "string" },
                        "ai_tool": { "type": "string" },
                        "ai_model": { "type": "string" },
                        "auth_method": { "type": "string" }
                    },
                    "required": ["user", "ai_tool"]
                }
            }),
            vec!["session_id", "context"]
        )
    }
    fn call(&self, _ctx: &ServerContext, args: &Value) -> Result<Value, String> {
        let session_id = get_str(args, "session_id")?;
        let ctx = extract_action_context(args)?;

        let handoff = HandoffInput {
            goal_next: get_opt_str(args, "goal_next"),
            decisions: get_opt_str_vec(args, "decisions"),
            next_steps: get_opt_str_vec(args, "next_steps"),
            failed_approaches: get_opt_str_vec(args, "failed_approaches"),
            relevant_files: get_opt_str_vec(args, "relevant_files"),
            context_usage_pct: get_opt_i64(args, "context_usage_pct"),
        };

        let now = chrono::Utc::now().to_rfc3339();

        let version = update_session(
            self.repo.pool().map_err(|e| e.to_string())?,
            &session_id,
            &handoff,
            &ctx,
            &now,
        )
        .map_err(|e| format!("{e}"))?;

        Ok(json_result(json!({
            "ok": true,
            "version": version
        })))
    }
}

// ============================================================================
// mark_consumed
// ============================================================================

/// Marks a session as consumed (superseded + consumed_at/consumed_by).
pub struct MarkConsumedTool {
    repo: Arc<SqliteRepo>,
}

impl MarkConsumedTool {
    pub fn new(repo: Arc<SqliteRepo>) -> Self {
        Self { repo }
    }
}

impl McpTool for MarkConsumedTool {
    fn name(&self) -> &str {
        "mark_consumed"
    }
    fn schema(&self) -> Value {
        schema_object(
            json!({
                "session_id": { "type": "string", "description": "Session ID" },
                "consumed_by": { "type": "string", "description": "ID of the session that resumes this one (optional)" },
                "context": {
                    "type": "object",
                    "description": "Action context (attribution required)",
                    "properties": {
                        "user": { "type": "string" },
                        "ai_tool": { "type": "string" },
                        "ai_model": { "type": "string" },
                        "auth_method": { "type": "string" }
                    },
                    "required": ["user", "ai_tool"]
                }
            }),
            vec!["session_id", "context"]
        )
    }
    fn call(&self, _ctx: &ServerContext, args: &Value) -> Result<Value, String> {
        let session_id = get_str(args, "session_id")?;
        let consumed_by = get_opt_str(args, "consumed_by");
        let ctx = extract_action_context(args)?;

        let now = chrono::Utc::now().to_rfc3339();

        mark_consumed(
            self.repo.pool().map_err(|e| e.to_string())?,
            &session_id,
            consumed_by.as_deref(),
            &ctx,
            &now,
            false,
        )
        .map_err(|e| format!("{e}"))?;

        Ok(text_result(format!("session {session_id} consumed")))
    }
}

// ============================================================================
// register_prompt
// ============================================================================

/// Registers a prompt reference (upsert per session_id, task_no, lang).
pub struct RegisterPromptTool {
    repo: Arc<SqliteRepo>,
}

impl RegisterPromptTool {
    pub fn new(repo: Arc<SqliteRepo>) -> Self {
        Self { repo }
    }
}

impl McpTool for RegisterPromptTool {
    fn name(&self) -> &str {
        "register_prompt"
    }
    fn schema(&self) -> Value {
        schema_object(
            json!({
                "session_id": { "type": "string", "description": "Session ID" },
                "task_no": { "type": "string", "description": "Task number (e.g. \"01\")" },
                "slug": { "type": "string", "description": "Slug of the brief" },
                "lang": { "type": "string", "description": "Language: ita or eng" },
                "path": { "type": "string", "description": "Path to the brief file" }
            }),
            vec!["session_id", "task_no", "slug", "lang", "path"]
        )
    }
    fn call(&self, _ctx: &ServerContext, args: &Value) -> Result<Value, String> {
        let session_id = get_str(args, "session_id")?;
        let task_no = get_str(args, "task_no")?;
        let slug = get_str(args, "slug")?;
        let lang = get_str(args, "lang")?;
        let path = get_str(args, "path")?;

        register_prompt(
            self.repo.pool().map_err(|e| e.to_string())?,
            &session_id,
            &task_no,
            &slug,
            &lang,
            &path,
        )
        .map_err(|e| format!("{e}"))?;

        Ok(text_result(format!("prompt registered for session {session_id} task {task_no}")))
    }
}

// ============================================================================
// create_task
// ============================================================================

/// Creates a new task with first revision (version 0).
pub struct CreateTaskTool {
    repo: Arc<SqliteRepo>,
}

impl CreateTaskTool {
    pub fn new(repo: Arc<SqliteRepo>) -> Self {
        Self { repo }
    }
}

impl McpTool for CreateTaskTool {
    fn name(&self) -> &str {
        "create_task"
    }
    fn schema(&self) -> Value {
        schema_object(
            json!({
                "session_id": { "type": "string", "description": "Parent session ID" },
                "task_id": { "type": "integer", "description": "Progressive task ID within the session" },
                "title": { "type": "string", "description": "Task title" },
                "body": { "type": "string", "description": "Task body (optional)" },
                "exec_order": { "type": "integer", "description": "Execution order" },
                "context": {
                    "type": "object",
                    "description": "Action context (attribution required)",
                    "properties": {
                        "user": { "type": "string" },
                        "ai_tool": { "type": "string" },
                        "ai_model": { "type": "string" },
                        "auth_method": { "type": "string" }
                    },
                    "required": ["user", "ai_tool"]
                }
            }),
            vec!["session_id", "task_id", "title", "exec_order", "context"]
        )
    }
    fn call(&self, _ctx: &ServerContext, args: &Value) -> Result<Value, String> {
        let session_id = get_str(args, "session_id")?;
        let task_id = get_str(args, "task_id")?
            .parse::<i64>()
            .map_err(|_| "argument 'task_id' must be an integer".to_string())?;
        let title = get_str(args, "title")?;
        let body = get_opt_str(args, "body");
        let exec_order = get_str(args, "exec_order")?
            .parse::<i64>()
            .map_err(|_| "argument 'exec_order' must be an integer".to_string())?;
        let ctx = extract_action_context(args)?;

        let now = chrono::Utc::now().to_rfc3339();

        let id = crate::tasks::writes::create_task(
            self.repo.pool().map_err(|e| e.to_string())?,
            &session_id,
            task_id,
            &title,
            body.as_deref(),
            exec_order,
            &ctx,
            &now,
        )
        .map_err(|e| format!("{e}"))?;

        Ok(json_result(json!({
            "ok": true,
            "task_id": id
        })))
    }
}

// ============================================================================
// revise_task
// ============================================================================

/// Creates a new revision of an existing task.
pub struct ReviseTaskTool {
    repo: Arc<SqliteRepo>,
}

impl ReviseTaskTool {
    pub fn new(repo: Arc<SqliteRepo>) -> Self {
        Self { repo }
    }
}

impl McpTool for ReviseTaskTool {
    fn name(&self) -> &str {
        "revise_task"
    }
    fn schema(&self) -> Value {
        schema_object(
            json!({
                "session_id": { "type": "string", "description": "Parent session ID" },
                "task_id": { "type": "integer", "description": "Task ID" },
                "title": { "type": "string", "description": "New title" },
                "body": { "type": "string", "description": "New body (optional)" },
                "exec_order": { "type": "integer", "description": "New execution order" },
                "context": {
                    "type": "object",
                    "description": "Action context (attribution required)",
                    "properties": {
                        "user": { "type": "string" },
                        "ai_tool": { "type": "string" },
                        "ai_model": { "type": "string" },
                        "auth_method": { "type": "string" }
                    },
                    "required": ["user", "ai_tool"]
                }
            }),
            vec!["session_id", "task_id", "title", "exec_order", "context"]
        )
    }
    fn call(&self, _ctx: &ServerContext, args: &Value) -> Result<Value, String> {
        let session_id = get_str(args, "session_id")?;
        let task_id = get_str(args, "task_id")?
            .parse::<i64>()
            .map_err(|_| "argument 'task_id' must be an integer".to_string())?;
        let title = get_str(args, "title")?;
        let body = get_opt_str(args, "body");
        let exec_order = get_str(args, "exec_order")?
            .parse::<i64>()
            .map_err(|_| "argument 'exec_order' must be an integer".to_string())?;
        let ctx = extract_action_context(args)?;

        let now = chrono::Utc::now().to_rfc3339();

        let version = crate::tasks::writes::revise_task(
            self.repo.pool().map_err(|e| e.to_string())?,
            &session_id,
            task_id,
            &title,
            body.as_deref(),
            exec_order,
            &ctx,
            &now,
        )
        .map_err(|e| format!("{e}"))?;

        Ok(json_result(json!({
            "ok": true,
            "version": version
        })))
    }
}

// ============================================================================
// set_task_status
// ============================================================================

/// Changes the status of a task (new revision).
pub struct SetTaskStatusTool {
    repo: Arc<SqliteRepo>,
}

impl SetTaskStatusTool {
    pub fn new(repo: Arc<SqliteRepo>) -> Self {
        Self { repo }
    }
}

impl McpTool for SetTaskStatusTool {
    fn name(&self) -> &str {
        "set_task_status"
    }
    fn schema(&self) -> Value {
        schema_object(
            json!({
                "session_id": { "type": "string", "description": "Parent session ID" },
                "task_id": { "type": "integer", "description": "Task ID" },
                "status": { "type": "string", "description": "New status: open, in_progress, done, superseded, blocked, cancelled" },
                "context": {
                    "type": "object",
                    "description": "Action context (attribution required)",
                    "properties": {
                        "user": { "type": "string" },
                        "ai_tool": { "type": "string" },
                        "ai_model": { "type": "string" },
                        "auth_method": { "type": "string" }
                    },
                    "required": ["user", "ai_tool"]
                }
            }),
            vec!["session_id", "task_id", "status", "context"]
        )
    }
    fn call(&self, _ctx: &ServerContext, args: &Value) -> Result<Value, String> {
        let session_id = get_str(args, "session_id")?;
        let task_id = get_str(args, "task_id")?
            .parse::<i64>()
            .map_err(|_| "argument 'task_id' must be an integer".to_string())?;
        let status_str = get_str(args, "status")?;
        let status = TaskStatus::try_from(status_str.as_str())
            .map_err(|e| format!("invalid status: {e}"))?;
        let ctx = extract_action_context(args)?;

        let now = chrono::Utc::now().to_rfc3339();

        let version = crate::tasks::writes::set_task_status(
            self.repo.pool().map_err(|e| e.to_string())?,
            &session_id,
            task_id,
            status,
            &ctx,
            &now,
        )
        .map_err(|e| format!("{e}"))?;

        Ok(json_result(json!({
            "ok": true,
            "version": version
        })))
    }
}

// ============================================================================
// close_task
// ============================================================================

/// Closes a task (sets status to cancelled).
pub struct CloseTaskTool {
    repo: Arc<SqliteRepo>,
}

impl CloseTaskTool {
    pub fn new(repo: Arc<SqliteRepo>) -> Self {
        Self { repo }
    }
}

impl McpTool for CloseTaskTool {
    fn name(&self) -> &str {
        "close_task"
    }
    fn schema(&self) -> Value {
        schema_object(
            json!({
                "session_id": { "type": "string", "description": "Parent session ID" },
                "task_id": { "type": "integer", "description": "Task ID" },
                "context": {
                    "type": "object",
                    "description": "Action context (attribution required)",
                    "properties": {
                        "user": { "type": "string" },
                        "ai_tool": { "type": "string" },
                        "ai_model": { "type": "string" },
                        "auth_method": { "type": "string" }
                    },
                    "required": ["user", "ai_tool"]
                }
            }),
            vec!["session_id", "task_id", "context"]
        )
    }
    fn call(&self, _ctx: &ServerContext, args: &Value) -> Result<Value, String> {
        let session_id = get_str(args, "session_id")?;
        let task_id = get_str(args, "task_id")?
            .parse::<i64>()
            .map_err(|_| "argument 'task_id' must be an integer".to_string())?;
        let ctx = extract_action_context(args)?;

        let now = chrono::Utc::now().to_rfc3339();

        let version = crate::tasks::writes::close_task(
            self.repo.pool().map_err(|e| e.to_string())?,
            &session_id,
            task_id,
            &ctx,
            &now,
        )
        .map_err(|e| format!("{e}"))?;

        Ok(json_result(json!({
            "ok": true,
            "version": version
        })))
    }
}

// ============================================================================
// get_task
// ============================================================================

/// Reads a task revision by session_id and task_id.
pub struct GetTaskTool {
    repo: Arc<SqliteRepo>,
}

impl GetTaskTool {
    pub fn new(repo: Arc<SqliteRepo>) -> Self {
        Self { repo }
    }
}

impl McpTool for GetTaskTool {
    fn name(&self) -> &str {
        "get_task"
    }
    fn schema(&self) -> Value {
        schema_object(
            json!({
                "session_id": { "type": "string", "description": "Parent session ID" },
                "task_id": { "type": "integer", "description": "Task ID" }
            }),
            vec!["session_id", "task_id"]
        )
    }
    fn call(&self, _ctx: &ServerContext, args: &Value) -> Result<Value, String> {
        let session_id = get_str(args, "session_id")?;
        let task_id = get_str(args, "task_id")?
            .parse::<i64>()
            .map_err(|_| "argument 'task_id' must be an integer".to_string())?;

        let task = crate::tasks::reads::get_task(
            self.repo.pool().map_err(|e| e.to_string())?,
            &session_id,
            task_id,
        )
        .map_err(|e| format!("{e}"))?;

        Ok(json_result(json!({
            "session_id": task.session_id,
            "task_id": task.task_id,
            "version": task.version,
            "status": task.status.as_str(),
            "title": task.title,
            "body": task.body,
            "exec_order": task.exec_order,
            "result": task.result,
            "spawned_task_id": task.spawned_task_id,
            "blocked_by": task.blocked_by,
            "created_at": task.created_at,
            "at": task.at
        })))
    }
}

// ============================================================================
// list_session_tasks
// ============================================================================

/// Lists all task revisions of a session.
pub struct ListSessionTasksTool {
    repo: Arc<SqliteRepo>,
}

impl ListSessionTasksTool {
    pub fn new(repo: Arc<SqliteRepo>) -> Self {
        Self { repo }
    }
}

impl McpTool for ListSessionTasksTool {
    fn name(&self) -> &str {
        "list_session_tasks"
    }
    fn schema(&self) -> Value {
        schema_object(
            json!({
                "session_id": { "type": "string", "description": "Parent session ID" }
            }),
            vec!["session_id"]
        )
    }
    fn call(&self, _ctx: &ServerContext, args: &Value) -> Result<Value, String> {
        let session_id = get_str(args, "session_id")?;

        let tasks = crate::tasks::reads::list_session_tasks(
            self.repo.pool().map_err(|e| e.to_string())?,
            &session_id,
        )
        .map_err(|e| format!("{e}"))?;

        let items: Vec<Value> = tasks
            .into_iter()
            .map(|t| {
                json!({
                    "task_id": t.task_id,
                    "version": t.version,
                    "status": t.status.as_str(),
                    "title": t.title,
                    "exec_order": t.exec_order,
                    "result": t.result,
                    "at": t.at
                })
            })
            .collect();

        Ok(json_result(json!({
            "session_id": session_id,
            "tasks": items
        })))
    }
}

// ============================================================================
// list_open_tasks
// ============================================================================

/// Lists all open tasks (not done/cancelled).
pub struct ListOpenTasksTool {
    repo: Arc<SqliteRepo>,
}

impl ListOpenTasksTool {
    pub fn new(repo: Arc<SqliteRepo>) -> Self {
        Self { repo }
    }
}

impl McpTool for ListOpenTasksTool {
    fn name(&self) -> &str {
        "list_open_tasks"
    }
    fn schema(&self) -> Value {
        schema_object(
            json!({}),
            vec![]
        )
    }
    fn call(&self, _ctx: &ServerContext, _args: &Value) -> Result<Value, String> {
        let tasks = crate::tasks::reads::list_open_tasks(
            self.repo.pool().map_err(|e| e.to_string())?,
        )
        .map_err(|e| format!("{e}"))?;

        let items: Vec<Value> = tasks
            .into_iter()
            .map(|t| {
                json!({
                    "session_id": t.session_id,
                    "task_id": t.task_id,
                    "version": t.version,
                    "status": t.status.as_str(),
                    "title": t.title,
                    "exec_order": t.exec_order,
                    "at": t.at
                })
            })
            .collect();

        Ok(json_result(json!({
            "tasks": items
        })))
    }
}

// ============================================================================
// list_lifecycle_states
// ============================================================================

/// Lists all lifecycle states.
pub struct ListLifecycleStatesTool {
    repo: Arc<SqliteRepo>,
}

impl ListLifecycleStatesTool {
    pub fn new(repo: Arc<SqliteRepo>) -> Self {
        Self { repo }
    }
}

impl McpTool for ListLifecycleStatesTool {
    fn name(&self) -> &str {
        "list_lifecycle_states"
    }
    fn schema(&self) -> Value {
        schema_object(
            json!({}),
            vec![]
        )
    }
    fn call(&self, _ctx: &ServerContext, _args: &Value) -> Result<Value, String> {
        let states = (*self.repo).states().map_err(|e| format!("{e}"))?;

        let items: Vec<Value> = states
            .into_iter()
            .map(|s| {
                json!({
                    "name": s.as_str(),
                    "is_visible": s.is_visible_default(),
                    "is_archivable": s.is_archivable(),
                    "is_reopenable": s.is_reopenable()
                })
            })
            .collect();

        Ok(json_result(json!({
            "states": items
        })))
    }
}

// ============================================================================
// list_lifecycle_transitions
// ============================================================================

/// Lists all valid lifecycle transitions.
pub struct ListLifecycleTransitionsTool {
    repo: Arc<SqliteRepo>,
}

impl ListLifecycleTransitionsTool {
    pub fn new(repo: Arc<SqliteRepo>) -> Self {
        Self { repo }
    }
}

impl McpTool for ListLifecycleTransitionsTool {
    fn name(&self) -> &str {
        "list_lifecycle_transitions"
    }
    fn schema(&self) -> Value {
        schema_object(
            json!({}),
            vec![]
        )
    }
    fn call(&self, _ctx: &ServerContext, _args: &Value) -> Result<Value, String> {
        let transitions = (*self.repo).allowed_transitions().map_err(|e| format!("{e}"))?;

        let items: Vec<Value> = transitions
            .into_iter()
            .map(|t| {
                json!({
                    "from": t.from.as_str(),
                    "to": t.to.as_str(),
                    "trigger": t.trigger.as_str(),
                    "description": t.description
                })
            })
            .collect();

        Ok(json_result(json!({
            "transitions": items
        })))
    }
}
