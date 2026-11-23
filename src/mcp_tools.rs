//! MCP tools for `nxm-tracker`.
//!
//! Exposes the session model as MCP tools so clients can create, read, update,
//! and close work sessions and feature sessions through the MCP protocol.
//!
//! Each tool holds an [`Arc<SqliteSessionRepository>`] for database access and
//! implements [`nxm_mcp_server::mcp::tool::McpTool`].

use std::sync::Arc;

use crate::git_workspace::{base_branch, create_integration_branch, is_git_repo};
use crate::session::{NewFeatureSession, NewSession, SessionRepository, SessionStatus};
use crate::session_repository::SqliteSessionRepository;
use nxm_mcp_server::mcp::tool::McpTool;
use nxm_mcp_server::server::ServerContext;
use serde_json::{json, Value};

/// Helper: build a JSON Schema object for tool input.
pub(crate) fn schema_object(properties: Value, required: Vec<&str>) -> Value {
    json!({
        "type": "object",
        "properties": properties,
        "required": required,
    })
}

/// Helper: extract a required string argument (accepts JSON string or number).
pub(crate) fn get_str(args: &Value, key: &str) -> Result<String, String> {
    match args.get(key) {
        Some(Value::String(s)) => Ok(s.clone()),
        Some(Value::Number(n)) => Ok(n.to_string()),
        Some(_) => Err(format!("argument '{key}' must be a string or number")),
        None => Err(format!("missing required argument: {key}")),
    }
}

/// Helper: extract an optional string argument.
pub(crate) fn get_opt_str(args: &Value, key: &str) -> Option<String> {
    args.get(key)
        .and_then(|v| v.as_str())
        .map(|s| s.to_string())
}

/// Helper: extract an optional boolean argument.
fn get_opt_bool(args: &Value, key: &str) -> Option<bool> {
    args.get(key).and_then(|v| v.as_bool())
}

/// Helper: extract an optional array-of-strings argument (empty when absent).
pub(crate) fn get_str_array(args: &Value, key: &str) -> Result<Vec<String>, String> {
    match args.get(key) {
        None => Ok(Vec::new()),
        Some(Value::Array(items)) => {
            let mut out = Vec::with_capacity(items.len());
            for item in items {
                match item {
                    Value::String(s) => out.push(s.clone()),
                    Value::Number(n) => out.push(n.to_string()),
                    _ => return Err(format!("argument '{key}' must be an array of strings")),
                }
            }
            Ok(out)
        }
        Some(_) => Err(format!("argument '{key}' must be an array of strings")),
    }
}

/// Helper: extract a required integer argument (accepts JSON number or numeric string).
pub(crate) fn get_id(args: &Value, key: &str) -> Result<i64, String> {
    get_str(args, key)?
        .parse::<i64>()
        .map_err(|_| format!("argument '{key}' must be an integer"))
}

/// Helper: validate that a string is non-empty (after trimming whitespace).
pub(crate) fn validate_non_empty(value: &str, field: &str) -> Result<(), String> {
    if value.trim().is_empty() {
        Err(format!("argument '{field}' must not be empty"))
    } else {
        Ok(())
    }
}

/// Helper: validate that a string is a valid RFC3339 timestamp.
fn validate_rfc3339(value: &str, field: &str) -> Result<(), String> {
    chrono::DateTime::parse_from_rfc3339(value)
        .map(|_| ())
        .map_err(|_| format!("argument '{field}' must be a valid RFC3339 timestamp"))
}

/// Helper: validate that a session with the given ID exists.
pub(crate) fn validate_session_exists(
    repo: &SqliteSessionRepository,
    id: i64,
) -> Result<(), String> {
    match repo.get_session(id).map_err(|e| format!("{e}"))? {
        Some(_) => Ok(()),
        None => Err(format!("session {id} not found")),
    }
}

/// Helper: build a text content result.
pub(crate) fn text_result(text: String) -> Value {
    json!({
        "content": [{"type": "text", "text": text}],
        "isError": false,
    })
}

macro_rules! impl_mcp_tool {
    ($tool:ident, $name:expr, $schema:expr) => {
        impl McpTool for $tool {
            fn name(&self) -> &str {
                $name
            }

            fn schema(&self) -> Value {
                $schema
            }

            fn call(&self, _ctx: &ServerContext, args: &Value) -> Result<Value, String> {
                $tool::execute(self, args)
            }
        }
    };
}

pub(crate) use impl_mcp_tool;

// ---------------------------------------------------------------------------
// CreateSession
// ---------------------------------------------------------------------------

/// Creates a new work session.
pub struct CreateSessionTool {
    repo: Arc<SqliteSessionRepository>,
}

impl CreateSessionTool {
    pub fn new(repo: Arc<SqliteSessionRepository>) -> Self {
        Self { repo }
    }
}

impl_mcp_tool!(
    CreateSessionTool,
    "create_session",
    schema_object(
        json!({
            "name": { "type": "string", "description": "Session name" },
            "started_at": { "type": "string", "description": "Start timestamp (RFC3339)" },
            "working_directory": { "type": "string", "description": "Path to a git project (optional). If set, an integration branch is created before the session." },
            "handoff": { "type": "string", "description": "Handoff description for the session, stored 1:1 and updatable later via update_handoff (optional)" },
            "plan": { "type": "array", "items": { "type": "string" }, "description": "Activity plan: ordered list of steps for the session (optional)" }
        }),
        vec!["name", "started_at"]
    )
);

impl CreateSessionTool {
    pub fn execute(&self, args: &Value) -> Result<Value, String> {
        let name = get_str(args, "name")?;
        let started_at = get_str(args, "started_at")?;
        let working_directory = get_opt_str(args, "working_directory");
        let handoff = get_opt_str(args, "handoff");
        let plan = get_str_array(args, "plan")?;

        validate_non_empty(&name, "name")?;
        validate_rfc3339(&started_at, "started_at")?;
        if let Some(ref h) = handoff {
            validate_non_empty(h, "handoff")?;
        }
        for (i, step) in plan.iter().enumerate() {
            validate_non_empty(step, &format!("plan[{i}]"))?;
        }

        // If a working directory is provided, ensure it is a git repository and
        // create an integration branch before creating the session.
        if let Some(path) = working_directory {
            is_git_repo(&path).map_err(|e| format!("invalid git workspace: {e}"))?;
            let base = base_branch(&path).map_err(|e| format!("no base branch in {path}: {e}"))?;
            create_integration_branch(&path, &base, &name)
                .map_err(|e| format!("failed to create integration branch: {e}"))?;
        }

        let new = NewSession { name, started_at };

        let (session, _handoff, _plan) = self
            .repo
            .create_session_full(&new, handoff.as_deref(), &plan)
            .map_err(|e| format!("failed to create session: {e}"))?;

        Ok(text_result(serde_json::to_string(&session).unwrap()))
    }
}

// ---------------------------------------------------------------------------
// GetSession
// ---------------------------------------------------------------------------

/// Returns a work session by ID.
pub struct GetSessionTool {
    repo: Arc<SqliteSessionRepository>,
}

impl GetSessionTool {
    pub fn new(repo: Arc<SqliteSessionRepository>) -> Self {
        Self { repo }
    }
}

impl_mcp_tool!(
    GetSessionTool,
    "get_session",
    schema_object(
        json!({
            "id": { "type": "integer", "description": "Session ID" }
        }),
        vec!["id"]
    )
);

impl GetSessionTool {
    pub fn execute(&self, args: &Value) -> Result<Value, String> {
        let id = get_str(args, "id")?
            .parse::<i64>()
            .map_err(|_| "argument 'id' must be an integer".to_string())?;

        validate_session_exists(&self.repo, id)?;

        match self.repo.get_session(id).map_err(|e| format!("{e}"))? {
            Some(session) => Ok(text_result(serde_json::to_string(&session).unwrap())),
            None => Ok(text_result(format!("session {id} not found"))),
        }
    }
}

// ---------------------------------------------------------------------------
// ListSessions
// ---------------------------------------------------------------------------

/// Lists all work sessions, ordered by started_at descending.
pub struct ListSessionsTool {
    repo: Arc<SqliteSessionRepository>,
}

impl ListSessionsTool {
    pub fn new(repo: Arc<SqliteSessionRepository>) -> Self {
        Self { repo }
    }
}

impl_mcp_tool!(
    ListSessionsTool,
    "list_sessions",
    json!({
        "type": "object",
        "properties": {},
        "required": []
    })
);

impl ListSessionsTool {
    pub fn execute(&self, _args: &Value) -> Result<Value, String> {
        let sessions = self.repo.list_sessions().map_err(|e| format!("{e}"))?;
        Ok(text_result(serde_json::to_string(&sessions).unwrap()))
    }
}

// ---------------------------------------------------------------------------
// UpdateSession
// ---------------------------------------------------------------------------

/// Updates an existing work session (name, status, design_phase_complete).
pub struct UpdateSessionTool {
    repo: Arc<SqliteSessionRepository>,
}

impl UpdateSessionTool {
    pub fn new(repo: Arc<SqliteSessionRepository>) -> Self {
        Self { repo }
    }
}

impl_mcp_tool!(
    UpdateSessionTool,
    "update_session",
    schema_object(
        json!({
            "id": { "type": "integer", "description": "Session ID" },
            "name": { "type": "string", "description": "New session name (optional)" },
            "status": { "type": "string", "description": "New status: open, paused, closed (optional)" },
            "design_phase_complete": { "type": "boolean", "description": "Design phase complete flag (optional)" }
        }),
        vec!["id"]
    )
);

impl UpdateSessionTool {
    pub fn execute(&self, args: &Value) -> Result<Value, String> {
        let id = get_str(args, "id")?
            .parse::<i64>()
            .map_err(|_| "argument 'id' must be an integer".to_string())?;

        let name = get_opt_str(args, "name");
        if let Some(ref n) = name {
            validate_non_empty(n, "name")?;
        }
        let status = get_opt_str(args, "status")
            .map(|s| SessionStatus::try_from(s.as_str()).map_err(|e| format!("{e}")))
            .transpose()?;
        let design_phase_complete = get_opt_bool(args, "design_phase_complete");

        validate_session_exists(&self.repo, id)?;

        let updated = self
            .repo
            .update_session(id, name.as_deref(), status, design_phase_complete)
            .map_err(|e| format!("{e}"))?;

        if !updated {
            return Ok(text_result(format!("session {id} not found")));
        }

        let session = self
            .repo
            .get_session(id)
            .map_err(|e| format!("{e}"))?
            .ok_or_else(|| format!("session {id} not found"))?;

        Ok(text_result(serde_json::to_string(&session).unwrap()))
    }
}

// ---------------------------------------------------------------------------
// CloseSession
// ---------------------------------------------------------------------------

/// Closes a work session, setting status to 'closed' and ended_at to now.
pub struct CloseSessionTool {
    repo: Arc<SqliteSessionRepository>,
}

impl CloseSessionTool {
    pub fn new(repo: Arc<SqliteSessionRepository>) -> Self {
        Self { repo }
    }
}

impl_mcp_tool!(
    CloseSessionTool,
    "close_session",
    schema_object(
        json!({
            "id": { "type": "integer", "description": "Session ID" }
        }),
        vec!["id"]
    )
);

impl CloseSessionTool {
    pub fn execute(&self, args: &Value) -> Result<Value, String> {
        let id = get_str(args, "id")?
            .parse::<i64>()
            .map_err(|_| "argument 'id' must be an integer".to_string())?;

        validate_session_exists(&self.repo, id)?;

        let success = self.repo.close_session(id).map_err(|e| format!("{e}"))?;

        if success {
            Ok(text_result(format!("session {id} closed")))
        } else {
            Ok(text_result(format!("session {id} not found")))
        }
    }
}

// ---------------------------------------------------------------------------
// SetDesignPhaseComplete
// ---------------------------------------------------------------------------

/// Sets the design_phase_complete flag on a work session.
pub struct SetDesignPhaseCompleteTool {
    repo: Arc<SqliteSessionRepository>,
}

impl SetDesignPhaseCompleteTool {
    pub fn new(repo: Arc<SqliteSessionRepository>) -> Self {
        Self { repo }
    }
}

impl_mcp_tool!(
    SetDesignPhaseCompleteTool,
    "set_design_phase_complete",
    schema_object(
        json!({
            "id": { "type": "integer", "description": "Session ID" },
            "complete": { "type": "boolean", "description": "Design phase complete flag" }
        }),
        vec!["id", "complete"]
    )
);

impl SetDesignPhaseCompleteTool {
    pub fn execute(&self, args: &Value) -> Result<Value, String> {
        let id = get_str(args, "id")?
            .parse::<i64>()
            .map_err(|_| "argument 'id' must be an integer".to_string())?;
        let complete = get_opt_bool(args, "complete")
            .ok_or_else(|| "missing required argument: complete".to_string())?;

        validate_session_exists(&self.repo, id)?;

        let success = self
            .repo
            .set_design_phase_complete(id, complete)
            .map_err(|e| format!("{e}"))?;

        if success {
            Ok(text_result(format!(
                "session {id} design_phase_complete set to {complete}"
            )))
        } else {
            Ok(text_result(format!("session {id} not found")))
        }
    }
}

// ---------------------------------------------------------------------------
// CreateFeatureSession
// ---------------------------------------------------------------------------

/// Creates a new feature session linked to a work session.
pub struct CreateFeatureSessionTool {
    repo: Arc<SqliteSessionRepository>,
}

impl CreateFeatureSessionTool {
    pub fn new(repo: Arc<SqliteSessionRepository>) -> Self {
        Self { repo }
    }
}

impl_mcp_tool!(
    CreateFeatureSessionTool,
    "create_feature_session",
    schema_object(
        json!({
            "session_id": { "type": "integer", "description": "Parent work session ID" },
            "name": { "type": "string", "description": "Feature session name" },
            "description": { "type": "string", "description": "Feature session description (optional)" }
        }),
        vec!["session_id", "name"]
    )
);

impl CreateFeatureSessionTool {
    pub fn execute(&self, args: &Value) -> Result<Value, String> {
        let session_id = get_str(args, "session_id")?
            .parse::<i64>()
            .map_err(|_| "argument 'session_id' must be an integer".to_string())?;
        let name = get_str(args, "name")?;
        let description = get_opt_str(args, "description");

        validate_non_empty(&name, "name")?;
        validate_session_exists(&self.repo, session_id)?;

        let new = NewFeatureSession {
            session_id,
            name,
            description,
        };

        let feature = self
            .repo
            .create_feature_session(&new)
            .map_err(|e| format!("failed to create feature session: {e}"))?;

        Ok(text_result(serde_json::to_string(&feature).unwrap()))
    }
}

// ---------------------------------------------------------------------------
// ListFeatureSessions
// ---------------------------------------------------------------------------

/// Lists all feature sessions for a work session, ordered by created_at descending.
pub struct ListFeatureSessionsTool {
    repo: Arc<SqliteSessionRepository>,
}

impl ListFeatureSessionsTool {
    pub fn new(repo: Arc<SqliteSessionRepository>) -> Self {
        Self { repo }
    }
}

impl_mcp_tool!(
    ListFeatureSessionsTool,
    "list_feature_sessions",
    schema_object(
        json!({
            "session_id": { "type": "integer", "description": "Parent work session ID" }
        }),
        vec!["session_id"]
    )
);

impl ListFeatureSessionsTool {
    pub fn execute(&self, args: &Value) -> Result<Value, String> {
        let session_id = get_str(args, "session_id")?
            .parse::<i64>()
            .map_err(|_| "argument 'session_id' must be an integer".to_string())?;
        validate_session_exists(&self.repo, session_id)?;

        let features = self
            .repo
            .list_feature_sessions(session_id)
            .map_err(|e| format!("{e}"))?;

        Ok(text_result(serde_json::to_string(&features).unwrap()))
    }
}
