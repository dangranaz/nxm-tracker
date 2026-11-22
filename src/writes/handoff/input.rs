//! Input data for `update_session`.
//!
//! Describes the fields of a handoff to write. Lists are
//! `Vec<String>` and are serialized as JSON TEXT on SQLite.

/// Input data for a session handoff.
///
/// List fields (`decisions`, `next_steps`, `failed_approaches`,
/// `relevant_files`) are serialized as JSON arrays in TEXT on SQLite.
/// `context_usage_pct` is optional (nullable).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct HandoffInput {
    /// Goal for the next work session (optional).
    pub goal_next: Option<String>,
    /// Decisions made during the session.
    pub decisions: Vec<String>,
    /// Next steps to execute.
    pub next_steps: Vec<String>,
    /// Failed approaches not to repeat.
    pub failed_approaches: Vec<String>,
    /// Files relevant for the context.
    pub relevant_files: Vec<String>,
    /// Context usage percentage (optional, nullable).
    pub context_usage_pct: Option<i64>,
}

impl Default for HandoffInput {
    fn default() -> Self {
        Self {
            goal_next: None,
            decisions: Vec::new(),
            next_steps: Vec::new(),
            failed_approaches: Vec::new(),
            relevant_files: Vec::new(),
            context_usage_pct: None,
        }
    }
}
