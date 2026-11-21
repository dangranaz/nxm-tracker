//! Tests for MCP tools.
//!
//! Uses temporary directories so the real `~/.nxm/shared-db` is never touched.
//! Calls the tools' `execute` method directly (bypassing the MCP transport).

use std::sync::Arc;

use nxm_db::DbConfig;
use nxm_tracker::{
    CloseSessionTool, CreateFeatureSessionTool, CreateSessionTool, Database, GetSessionTool,
    ListFeatureSessionsTool, ListSessionsTool, SetDesignPhaseCompleteTool, SqliteSessionRepository,
    UpdateSessionTool,
};
use serde_json::{json, Value};
use std::fs;

fn temp_db(suffix: &str) -> (DbConfig, std::path::PathBuf) {
    let root =
        std::env::temp_dir().join(format!("nxm-tracker-mcp-{}-{}", suffix, std::process::id()));
    let _ = fs::remove_dir_all(&root);

    let db_path = root.join("mcp.db");
    let config = DbConfig {
        backend: nxm_db::Backend::Sqlite,
        url: db_path.to_string_lossy().to_string(),
        tenant: "default".to_string(),
    };
    (config, root)
}

fn open_temp_db(suffix: &str) -> (Database, std::path::PathBuf) {
    let (config, root) = temp_db(suffix);
    let db = Database::open_from(&config).expect("database should open");
    let set = nxm_tracker::default_migration_set().expect("migration set should be valid");
    db.migrate(&set).expect("migrations should apply");
    (db, root)
}

fn repo_for(suffix: &str) -> (std::sync::Arc<SqliteSessionRepository>, std::path::PathBuf) {
    let (db, root) = open_temp_db(suffix);
    let repo = std::sync::Arc::new(SqliteSessionRepository::from_database(db));
    (repo, root)
}

#[test]
fn create_session_tool() {
    let (repo, _root) = repo_for("create-session");
    let tool = CreateSessionTool::new(Arc::clone(&repo));

    let result = tool
        .execute(&json!({
            "name": "MCP test session",
            "started_at": "2026-11-21T09:00:00+01:00"
        }))
        .expect("create_session should succeed");

    let value: Value =
        serde_json::from_str(result["content"][0]["text"].as_str().unwrap()).unwrap();
    assert_eq!(value["name"], "MCP test session");
    assert_eq!(value["status"], "open");
    assert!(value["id"].is_number());
    assert!(value["design_phase_complete"] == false || value["design_phase_complete"] == 0);
}

#[test]
fn get_session_tool() {
    let (repo, _root) = repo_for("get-session");
    let tool = CreateSessionTool::new(Arc::clone(&repo));

    let create_result = tool
        .execute(&json!({
            "name": "To retrieve",
            "started_at": "2026-11-21T09:00:00+01:00"
        }))
        .expect("create should succeed");
    let created: Value =
        serde_json::from_str(create_result["content"][0]["text"].as_str().unwrap()).unwrap();
    let id = created["id"].as_i64().unwrap();

    let get_tool = GetSessionTool::new(Arc::clone(&repo));
    let result = get_tool
        .execute(&json!({ "id": id }))
        .expect("get_session should succeed");

    let text = result["content"][0]["text"].as_str().unwrap();
    if text == format!("session {id} not found") {
        panic!("session should exist");
    }
    let value: Value = serde_json::from_str(text).unwrap();
    assert_eq!(value["id"], id);
    assert_eq!(value["name"], "To retrieve");
}

#[test]
fn get_nonexistent_session_tool() {
    let (repo, _root) = repo_for("get-nonexistent");
    let tool = GetSessionTool::new(Arc::clone(&repo));

    let result = tool
        .execute(&json!({ "id": 9999 }))
        .expect("get_session should not error");
    assert_eq!(result["content"][0]["text"], "session 9999 not found");
}

#[test]
fn list_sessions_tool() {
    let (repo, _root) = repo_for("list-sessions");
    let tool = CreateSessionTool::new(Arc::clone(&repo));

    tool.execute(&json!({
        "name": "First",
        "started_at": "2026-11-21T09:00:00+01:00"
    }))
    .expect("create first");
    tool.execute(&json!({
        "name": "Second",
        "started_at": "2026-11-22T09:00:00+01:00"
    }))
    .expect("create second");

    let list_tool = ListSessionsTool::new(Arc::clone(&repo));
    let result = list_tool
        .execute(&json!({}))
        .expect("list_sessions should succeed");

    let text = result["content"][0]["text"].as_str().unwrap();
    let sessions: Value = serde_json::from_str(text).unwrap();
    assert_eq!(sessions.as_array().unwrap().len(), 2);
    assert_eq!(sessions[0]["name"], "Second"); // DESC order
    assert_eq!(sessions[1]["name"], "First");
}

#[test]
fn update_session_tool_changes_name_and_status() {
    let (repo, _root) = repo_for("update-session");
    let create_tool = CreateSessionTool::new(Arc::clone(&repo));

    let create_result = create_tool
        .execute(&json!({
            "name": "Original",
            "started_at": "2026-11-21T09:00:00+01:00"
        }))
        .expect("create should succeed");
    let created: Value =
        serde_json::from_str(create_result["content"][0]["text"].as_str().unwrap()).unwrap();
    let id = created["id"].as_i64().unwrap();

    let update_tool = UpdateSessionTool::new(Arc::clone(&repo));
    let result = update_tool
        .execute(&json!({
            "id": id,
            "name": "Renamed",
            "status": "paused"
        }))
        .expect("update_session should succeed");

    let text = result["content"][0]["text"].as_str().unwrap();
    let updated: Value = serde_json::from_str(text).unwrap();
    assert_eq!(updated["name"], "Renamed");
    assert_eq!(updated["status"], "paused");
}

#[test]
fn update_session_tool_sets_design_phase_complete() {
    let (repo, _root) = repo_for("update-design-phase");
    let create_tool = CreateSessionTool::new(Arc::clone(&repo));

    let create_result = create_tool
        .execute(&json!({
            "name": "Design phase",
            "started_at": "2026-11-21T09:00:00+01:00"
        }))
        .expect("create should succeed");
    let created: Value =
        serde_json::from_str(create_result["content"][0]["text"].as_str().unwrap()).unwrap();
    let id = created["id"].as_i64().unwrap();

    let update_tool = UpdateSessionTool::new(Arc::clone(&repo));
    let result = update_tool
        .execute(&json!({
            "id": id,
            "design_phase_complete": true
        }))
        .expect("update_session should succeed");

    let text = result["content"][0]["text"].as_str().unwrap();
    let updated: Value = serde_json::from_str(text).unwrap();
    assert_eq!(updated["design_phase_complete"], true);
}

#[test]
fn update_session_tool_nonexistent() {
    let (repo, _root) = repo_for("update-nonexistent");
    let tool = UpdateSessionTool::new(Arc::clone(&repo));

    let result = tool
        .execute(&json!({
            "id": 9999,
            "name": "Ghost"
        }))
        .expect("update_session should not error");
    assert_eq!(result["content"][0]["text"], "session 9999 not found");
}

#[test]
fn close_session_tool() {
    let (repo, _root) = repo_for("close-session");
    let create_tool = CreateSessionTool::new(Arc::clone(&repo));

    let create_result = create_tool
        .execute(&json!({
            "name": "To close",
            "started_at": "2026-11-21T09:00:00+01:00"
        }))
        .expect("create should succeed");
    let created: Value =
        serde_json::from_str(create_result["content"][0]["text"].as_str().unwrap()).unwrap();
    let id = created["id"].as_i64().unwrap();

    let close_tool = CloseSessionTool::new(Arc::clone(&repo));
    let result = close_tool
        .execute(&json!({ "id": id }))
        .expect("close_session should succeed");

    assert_eq!(result["content"][0]["text"], format!("session {id} closed"));

    // Verify status changed
    let get_tool = GetSessionTool::new(Arc::clone(&repo));
    let result = get_tool
        .execute(&json!({ "id": id }))
        .expect("get_session should succeed");
    let text = result["content"][0]["text"].as_str().unwrap();
    let session: Value = serde_json::from_str(text).unwrap();
    assert_eq!(session["status"], "closed");
    assert!(session["ended_at"].is_string());
}

#[test]
fn close_nonexistent_session_tool() {
    let (repo, _root) = repo_for("close-nonexistent");
    let tool = CloseSessionTool::new(Arc::clone(&repo));

    let result = tool
        .execute(&json!({ "id": 9999 }))
        .expect("close_session should not error");
    assert_eq!(result["content"][0]["text"], "session 9999 not found");
}

#[test]
fn set_design_phase_complete_tool() {
    let (repo, _root) = repo_for("set-design-phase");
    let create_tool = CreateSessionTool::new(Arc::clone(&repo));

    let create_result = create_tool
        .execute(&json!({
            "name": "Design",
            "started_at": "2026-11-21T09:00:00+01:00"
        }))
        .expect("create should succeed");
    let created: Value =
        serde_json::from_str(create_result["content"][0]["text"].as_str().unwrap()).unwrap();
    let id = created["id"].as_i64().unwrap();

    let tool = SetDesignPhaseCompleteTool::new(Arc::clone(&repo));
    let result = tool
        .execute(&json!({ "id": id, "complete": true }))
        .expect("set_design_phase_complete should succeed");

    assert!(result["content"][0]["text"]
        .as_str()
        .unwrap()
        .contains("true"));

    // Verify
    let get_tool = GetSessionTool::new(Arc::clone(&repo));
    let result = get_tool
        .execute(&json!({ "id": id }))
        .expect("get_session should succeed");
    let text = result["content"][0]["text"].as_str().unwrap();
    let session: Value = serde_json::from_str(text).unwrap();
    assert_eq!(session["design_phase_complete"], true);
}

#[test]
fn create_and_list_feature_sessions_tool() {
    let (repo, _root) = repo_for("create-feature-session");
    let create_tool = CreateSessionTool::new(Arc::clone(&repo));

    let create_result = create_tool
        .execute(&json!({
            "name": "Parent",
            "started_at": "2026-11-21T09:00:00+01:00"
        }))
        .expect("create should succeed");
    let created: Value =
        serde_json::from_str(create_result["content"][0]["text"].as_str().unwrap()).unwrap();
    let session_id = created["id"].as_i64().unwrap();

    let feature_tool = CreateFeatureSessionTool::new(Arc::clone(&repo));
    let result = feature_tool
        .execute(&json!({
            "session_id": session_id,
            "name": "Implement MCP tools",
            "description": "Add create, get, list, update, close session tools"
        }))
        .expect("create_feature_session should succeed");

    let text = result["content"][0]["text"].as_str().unwrap();
    let feature: Value = serde_json::from_str(text).unwrap();
    assert_eq!(feature["session_id"], session_id);
    assert_eq!(feature["name"], "Implement MCP tools");
    assert_eq!(
        feature["description"],
        "Add create, get, list, update, close session tools"
    );
    assert_eq!(feature["status"], "open");

    // List feature sessions
    let list_tool = ListFeatureSessionsTool::new(Arc::clone(&repo));
    let list_result = list_tool
        .execute(&json!({ "session_id": session_id }))
        .expect("list_feature_sessions should succeed");

    let list_text = list_result["content"][0]["text"].as_str().unwrap();
    let features: Value = serde_json::from_str(list_text).unwrap();
    assert_eq!(features.as_array().unwrap().len(), 1);
    assert_eq!(features[0]["name"], "Implement MCP tools");
}
