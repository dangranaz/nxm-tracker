//! `HandoffView` — vista di lettura di una handoff revision.

use serde_json::Value;

/// Vista di lettura di una revisione di handoff.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct HandoffView {
    pub id: i64,
    pub session_id: String,
    pub version: i64,
    pub goal_next: Option<String>,
    pub decisions: Vec<String>,
    pub next_steps: Vec<String>,
    pub failed_approaches: Vec<String>,
    pub relevant_files: Vec<String>,
    pub context_usage_pct: Option<i64>,
    pub at: String,
}

impl HandoffView {
    pub fn from_json_list(json_str: Option<&str>) -> Vec<String> {
        json_str
            .and_then(|s| serde_json::from_str(s).ok())
            .unwrap_or_default()
    }
}
