//! Tests for the handoff / activity-plan / task MCP tools.
//!
//! Uses temporary directories so the real `~/.nxm/shared-db` is never touched.
//! Calls the tools' `execute` method directly (bypassing the MCP transport).

use nxm_db::DbConfig;
use nxm_tracker::{
    CloseTaskTool, CreateSessionTool, CreateTaskTool, Database, GetHandoffTool, GetTaskReportTool,
    GetTaskTool, InvalidateTaskTool, ListActivityPlansTool, ListTasksTool, SqliteSessionRepository,
    UpdateHandoffTool,
};
use serde_json::{json, Value};
use std::fs;
use std::sync::Arc;

fn repo_for(suffix: &str) -> (Arc<SqliteSessionRepository>, std::path::PathBuf) {
    let root = std::env::temp_dir().join(format!(
        "nxm-tracker-track-{}-{}",
        suffix,
        std::process::id()
    ));
    let _ = fs::remove_dir_all(&root);
    let db_path = root.join("track.db");
    let config = DbConfig {
        backend: nxm_db::Backend::Sqlite,
        url: db_path.to_string_lossy().to_string(),
        tenant: "default".to_string(),
    };
    let db = Database::open_from(&config).expect("database should open");
    let set = nxm_tracker::default_migration_set().expect("migration set should be valid");
    db.migrate(&set).expect("migrations should apply");
    (Arc::new(SqliteSessionRepository::from_database(db)), root)
}

fn create_session(repo: &Arc<SqliteSessionRepository>, name: &str) -> i64 {
    let tool = CreateSessionTool::new(Arc::clone(repo));
    let result = tool
        .execute(&json!({
            "name": name,
            "started_at": "2026-11-22T09:00:00+01:00"
        }))
        .expect("create_session should succeed");
    let value: Value =
        serde_json::from_str(result["content"][0]["text"].as_str().unwrap()).unwrap();
    value["id"].as_i64().unwrap()
}

fn text_of(result: &Value) -> &str {
    result["content"][0]["text"].as_str().unwrap()
}

#[test]
fn create_session_with_handoff_and_plan() {
    let (repo, _root) = repo_for("create-with-content");
    let tool = CreateSessionTool::new(Arc::clone(&repo));
    let result = tool
        .execute(&json!({
            "name": "Full session",
            "started_at": "2026-11-22T09:00:00+01:00",
            "handoff": "Handoff: design done, see docs/",
            "plan": ["step 1: schema", "step 2: tests"]
        }))
        .expect("create_session with handoff and plan should succeed");
    let session: Value = serde_json::from_str(text_of(&result)).unwrap();
    let session_id = session["id"].as_i64().unwrap();

    let get_handoff = GetHandoffTool::new(Arc::clone(&repo));
    let handoff: Value = serde_json::from_str(text_of(
        &get_handoff
            .execute(&json!({"session_id": session_id}))
            .unwrap(),
    ))
    .unwrap();
    assert_eq!(handoff["content"], "Handoff: design done, see docs/");
    assert_eq!(handoff["session_id"], session_id);

    let list_plans = ListActivityPlansTool::new(Arc::clone(&repo));
    let plans: Value = serde_json::from_str(text_of(
        &list_plans
            .execute(&json!({"session_id": session_id}))
            .unwrap(),
    ))
    .unwrap();
    let plans = plans.as_array().unwrap();
    assert_eq!(plans.len(), 2);
    assert_eq!(plans[0]["position"], 1);
    assert_eq!(plans[0]["description"], "step 1: schema");
    assert_eq!(plans[1]["position"], 2);
}

#[test]
fn create_session_rejects_empty_handoff_and_empty_plan_step() {
    let (repo, _root) = repo_for("create-invalid-content");
    let tool = CreateSessionTool::new(Arc::clone(&repo));

    let err = tool
        .execute(&json!({
            "name": "X",
            "started_at": "2026-11-22T09:00:00+01:00",
            "handoff": "   "
        }))
        .expect_err("empty handoff must be rejected");
    assert!(err.contains("handoff"), "got: {err}");

    let err = tool
        .execute(&json!({
            "name": "X",
            "started_at": "2026-11-22T09:00:00+01:00",
            "plan": ["ok", "  "]
        }))
        .expect_err("empty plan step must be rejected");
    assert!(err.contains("plan[1]"), "got: {err}");
}

#[test]
fn update_handoff_tool_upserts() {
    let (repo, _root) = repo_for("update-handoff");
    let session_id = create_session(&repo, "Handoff target");

    let tool = UpdateHandoffTool::new(Arc::clone(&repo));
    let first: Value = serde_json::from_str(text_of(
        &tool
            .execute(&json!({"session_id": session_id, "content": "v1"}))
            .unwrap(),
    ))
    .unwrap();
    assert_eq!(first["content"], "v1");

    let second: Value = serde_json::from_str(text_of(
        &tool
            .execute(&json!({"session_id": session_id, "content": "v2"}))
            .unwrap(),
    ))
    .unwrap();
    assert_eq!(second["content"], "v2");
    assert_eq!(second["id"], first["id"]);

    // Empty content rejected.
    let err = tool
        .execute(&json!({"session_id": session_id, "content": " "}))
        .expect_err("empty content must be rejected");
    assert!(err.contains("content"), "got: {err}");

    // Nonexistent session rejected.
    let err = tool
        .execute(&json!({"session_id": 9999, "content": "x"}))
        .expect_err("nonexistent session must be rejected");
    assert!(err.contains("session 9999 not found"), "got: {err}");
}

#[test]
fn get_handoff_tool_errors_without_handoff() {
    let (repo, _root) = repo_for("get-handoff-missing");
    let session_id = create_session(&repo, "No handoff yet");

    let tool = GetHandoffTool::new(Arc::clone(&repo));
    let err = tool
        .execute(&json!({"session_id": session_id}))
        .expect_err("missing handoff must be an error");
    assert!(err.contains("has no handoff"), "got: {err}");
}

#[test]
fn create_list_get_task_tools() {
    let (repo, _root) = repo_for("task-crud");
    let session_id = create_session(&repo, "Task parent");

    let create_task = CreateTaskTool::new(Arc::clone(&repo));
    let t1: Value = serde_json::from_str(text_of(
        &create_task
            .execute(&json!({"session_id": session_id, "name": "task/alpha"}))
            .unwrap(),
    ))
    .unwrap();
    assert_eq!(t1["status"], "open");
    assert_eq!(t1["name"], "task/alpha");
    let t1_id = t1["id"].as_i64().unwrap();

    let t2: Value = serde_json::from_str(text_of(
        &create_task
            .execute(&json!({"session_id": session_id, "name": "task/beta"}))
            .unwrap(),
    ))
    .unwrap();

    let list = ListTasksTool::new(Arc::clone(&repo));
    let all: Value = serde_json::from_str(text_of(
        &list.execute(&json!({"session_id": session_id})).unwrap(),
    ))
    .unwrap();
    assert_eq!(all.as_array().unwrap().len(), 2);

    let open_only: Value = serde_json::from_str(text_of(
        &list
            .execute(&json!({"session_id": session_id, "status": "open"}))
            .unwrap(),
    ))
    .unwrap();
    assert_eq!(open_only.as_array().unwrap().len(), 2);

    let get = GetTaskTool::new(Arc::clone(&repo));
    let fetched: Value =
        serde_json::from_str(text_of(&get.execute(&json!({"id": t1_id})).unwrap())).unwrap();
    assert_eq!(fetched["name"], "task/alpha");

    let err = get
        .execute(&json!({"id": 9999}))
        .expect_err("nonexistent task");
    assert!(err.contains("task 9999 not found"), "got: {err}");

    // Validation: empty name, nonexistent session, bad status filter.
    let err = create_task
        .execute(&json!({"session_id": session_id, "name": " "}))
        .expect_err("empty task name rejected");
    assert!(err.contains("name"), "got: {err}");
    let err = create_task
        .execute(&json!({"session_id": 9999, "name": "task/x"}))
        .expect_err("nonexistent session rejected");
    assert!(err.contains("session 9999 not found"), "got: {err}");
    let err = list
        .execute(&json!({"session_id": session_id, "status": "bogus"}))
        .expect_err("bad status rejected");
    assert!(err.contains("invalid task status"), "got: {err}");

    drop(t2);
}

#[test]
fn close_task_tool_requires_report() {
    let (repo, _root) = repo_for("close-task");
    let session_id = create_session(&repo, "Close parent");
    let create_task = CreateTaskTool::new(Arc::clone(&repo));
    let task: Value = serde_json::from_str(text_of(
        &create_task
            .execute(&json!({"session_id": session_id, "name": "task/x"}))
            .unwrap(),
    ))
    .unwrap();
    let task_id = task["id"].as_i64().unwrap();

    let close = CloseTaskTool::new(Arc::clone(&repo));

    // Missing / empty report rejected.
    let err = close
        .execute(&json!({"id": task_id}))
        .expect_err("missing report must be rejected");
    assert!(err.contains("report"), "got: {err}");
    let err = close
        .execute(&json!({"id": task_id, "report": "  "}))
        .expect_err("empty report must be rejected");
    assert!(err.contains("report"), "got: {err}");

    // Valid close works and attaches the report.
    let closed: Value = serde_json::from_str(text_of(
        &close
            .execute(&json!({"id": task_id, "report": "all done"}))
            .unwrap(),
    ))
    .unwrap();
    assert_eq!(closed["status"], "closed");
    assert!(closed["closed_at"].is_string());

    let report_tool = GetTaskReportTool::new(Arc::clone(&repo));
    let report: Value = serde_json::from_str(text_of(
        &report_tool.execute(&json!({"task_id": task_id})).unwrap(),
    ))
    .unwrap();
    assert_eq!(report["content"], "all done");

    // Closing again fails (terminal state).
    let err = close
        .execute(&json!({"id": task_id, "report": "again"}))
        .expect_err("closing a closed task must fail");
    assert!(err.contains("not open"), "got: {err}");

    // Nonexistent task.
    let err = close
        .execute(&json!({"id": 9999, "report": "x"}))
        .expect_err("nonexistent task must fail");
    assert!(err.contains("task 9999 not found"), "got: {err}");
}

#[test]
fn invalidate_task_tool() {
    let (repo, _root) = repo_for("invalidate-tool");
    let session_id = create_session(&repo, "Invalidate parent");
    let create_task = CreateTaskTool::new(Arc::clone(&repo));
    let task: Value = serde_json::from_str(text_of(
        &create_task
            .execute(&json!({"session_id": session_id, "name": "task/old"}))
            .unwrap(),
    ))
    .unwrap();
    let task_id = task["id"].as_i64().unwrap();

    let invalidate = InvalidateTaskTool::new(Arc::clone(&repo));
    let invalid: Value = serde_json::from_str(text_of(
        &invalidate
            .execute(&json!({"id": task_id, "reason": "no longer valuable"}))
            .unwrap(),
    ))
    .unwrap();
    assert_eq!(invalid["status"], "invalid");
    assert_eq!(invalid["note"], "no longer valuable");

    // Invalidate again fails (terminal).
    let err = invalidate
        .execute(&json!({"id": task_id}))
        .expect_err("invalidating an invalid task must fail");
    assert!(err.contains("not open"), "got: {err}");

    // Invalid task shows up in the filtered list.
    let list = ListTasksTool::new(Arc::clone(&repo));
    let invalid_list: Value = serde_json::from_str(text_of(
        &list
            .execute(&json!({"session_id": session_id, "status": "invalid"}))
            .unwrap(),
    ))
    .unwrap();
    assert_eq!(invalid_list.as_array().unwrap().len(), 1);

    // Nonexistent task.
    let err = invalidate
        .execute(&json!({"id": 9999}))
        .expect_err("nonexistent task must fail");
    assert!(err.contains("task 9999 not found"), "got: {err}");
}

#[test]
fn get_task_report_tool_errors_before_close() {
    let (repo, _root) = repo_for("report-missing");
    let session_id = create_session(&repo, "Report parent");
    let create_task = CreateTaskTool::new(Arc::clone(&repo));
    let task: Value = serde_json::from_str(text_of(
        &create_task
            .execute(&json!({"session_id": session_id, "name": "task/open"}))
            .unwrap(),
    ))
    .unwrap();
    let task_id = task["id"].as_i64().unwrap();

    let report_tool = GetTaskReportTool::new(Arc::clone(&repo));
    let err = report_tool
        .execute(&json!({"task_id": task_id}))
        .expect_err("open task has no report yet");
    assert!(err.contains("has no report"), "got: {err}");

    let err = report_tool
        .execute(&json!({"task_id": 9999}))
        .expect_err("nonexistent task must fail");
    assert!(err.contains("task 9999 not found"), "got: {err}");
}
