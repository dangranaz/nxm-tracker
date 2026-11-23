//! Integration tests for execution tasks and their final reports.
//!
//! Uses temporary directories so the real `~/.nxm/shared-db` is never touched.

use nxm_db::DbConfig;
use nxm_tracker::{
    Database, NewSession, NewTask, SessionRepository, SqliteSessionRepository, TaskRepository,
    TaskStatus, TrackerError,
};
use std::fs;

fn open_temp_db(suffix: &str) -> (Database, std::path::PathBuf) {
    let root = std::env::temp_dir().join(format!(
        "nxm-tracker-task-{}-{}",
        suffix,
        std::process::id()
    ));
    let _ = fs::remove_dir_all(&root);

    let db_path = root.join("task.db");
    let config = DbConfig {
        backend: nxm_db::Backend::Sqlite,
        url: db_path.to_string_lossy().to_string(),
        tenant: "default".to_string(),
    };
    let db = Database::open_from(&config).expect("database should open");
    let set = nxm_tracker::default_migration_set().expect("migration set should be valid");
    db.migrate(&set).expect("migrations should apply");
    (db, root)
}

fn create_session(repo: &SqliteSessionRepository, name: &str) -> i64 {
    repo.create_session(&NewSession {
        name: name.to_string(),
        started_at: "2026-11-22T09:00:00+01:00".to_string(),
    })
    .expect("create session should succeed")
    .id
    .expect("session should have an id")
}

#[test]
fn create_and_get_task() {
    let (db, _root) = open_temp_db("create-get");
    let repo = SqliteSessionRepository::from_database(db);
    let session_id = create_session(&repo, "Task parent");

    let task = repo
        .create_task(&NewTask {
            session_id,
            name: "task/setup-db".to_string(),
        })
        .expect("create_task should succeed");

    assert_eq!(task.status, TaskStatus::Open);
    assert_eq!(task.name, "task/setup-db");
    assert!(task.closed_at.is_none());

    let fetched = repo
        .get_task(task.id.unwrap())
        .expect("get_task should succeed")
        .expect("task should exist");
    assert_eq!(fetched.session_id, session_id);
    assert_eq!(fetched.status, TaskStatus::Open);
}

#[test]
fn get_nonexistent_task_returns_none() {
    let (db, _root) = open_temp_db("get-nonexistent");
    let repo = SqliteSessionRepository::from_database(db);
    assert!(repo.get_task(9999).expect("get should not error").is_none());
}

#[test]
fn list_tasks_and_filter_by_status() {
    let (db, _root) = open_temp_db("list-filter");
    let repo = SqliteSessionRepository::from_database(db);
    let session_id = create_session(&repo, "List parent");

    let t1 = repo
        .create_task(&NewTask {
            session_id,
            name: "task/one".to_string(),
        })
        .unwrap();
    let t2 = repo
        .create_task(&NewTask {
            session_id,
            name: "task/two".to_string(),
        })
        .unwrap();
    let t3 = repo
        .create_task(&NewTask {
            session_id,
            name: "task/three".to_string(),
        })
        .unwrap();

    repo.invalidate_task(t3.id.unwrap(), Some("superseded"))
        .unwrap();

    let all = repo
        .list_tasks(session_id)
        .expect("list_tasks should succeed");
    assert_eq!(all.len(), 3);

    let open = repo
        .list_tasks_by_status(session_id, TaskStatus::Open)
        .expect("filter open should succeed");
    assert_eq!(open.len(), 2);
    assert!(open.iter().any(|t| t.id == t1.id));
    assert!(open.iter().any(|t| t.id == t2.id));

    let invalid = repo
        .list_tasks_by_status(session_id, TaskStatus::Invalid)
        .expect("filter invalid should succeed");
    assert_eq!(invalid.len(), 1);
    assert_eq!(invalid[0].id, t3.id);
}

#[test]
fn close_task_attaches_report_and_sets_closed_at() {
    let (db, _root) = open_temp_db("close");
    let repo = SqliteSessionRepository::from_database(db);
    let session_id = create_session(&repo, "Close parent");
    let task = repo
        .create_task(&NewTask {
            session_id,
            name: "task/close-me".to_string(),
        })
        .unwrap();
    let task_id = task.id.unwrap();

    let closed = repo
        .close_task(task_id, "done: all green")
        .expect("close_task should succeed")
        .expect("task should exist");
    assert_eq!(closed.status, TaskStatus::Closed);
    assert!(closed.closed_at.is_some());

    let report = repo
        .get_task_report(task_id)
        .expect("get_task_report should succeed")
        .expect("report should exist after close");
    assert_eq!(report.content, "done: all green");
    assert_eq!(report.task_id, task_id);
}

#[test]
fn close_task_twice_fails_with_task_not_open() {
    let (db, _root) = open_temp_db("close-twice");
    let repo = SqliteSessionRepository::from_database(db);
    let session_id = create_session(&repo, "Twice parent");
    let task = repo
        .create_task(&NewTask {
            session_id,
            name: "task/once".to_string(),
        })
        .unwrap();
    let task_id = task.id.unwrap();

    repo.close_task(task_id, "first close").unwrap();
    let err = repo
        .close_task(task_id, "second close")
        .expect_err("closing a closed task must fail");
    let ok = matches!(
        &err,
        TrackerError::TaskNotOpen { id, status } if *id == task_id && status == "closed"
    );
    assert!(ok, "expected TaskNotOpen, got: {err}");
}

#[test]
fn close_nonexistent_task_returns_none() {
    let (db, _root) = open_temp_db("close-nonexistent");
    let repo = SqliteSessionRepository::from_database(db);
    let result = repo.close_task(9999, "report").expect("should not error");
    assert!(result.is_none());
}

#[test]
fn invalidate_task_sets_status_and_reason() {
    let (db, _root) = open_temp_db("invalidate");
    let repo = SqliteSessionRepository::from_database(db);
    let session_id = create_session(&repo, "Invalidate parent");
    let task = repo
        .create_task(&NewTask {
            session_id,
            name: "task/no-longer-needed".to_string(),
        })
        .unwrap();
    let task_id = task.id.unwrap();

    let invalid = repo
        .invalidate_task(task_id, Some("superseded by task/other"))
        .expect("invalidate should succeed")
        .expect("task should exist");
    assert_eq!(invalid.status, TaskStatus::Invalid);
    assert_eq!(invalid.note.as_deref(), Some("superseded by task/other"));
    assert!(invalid.closed_at.is_none());

    // Invalid is terminal: closing afterwards must fail.
    let err = repo
        .close_task(task_id, "late report")
        .expect_err("closing an invalid task must fail");
    assert!(matches!(err, TrackerError::TaskNotOpen { status, .. } if status == "invalid"));
}

#[test]
fn invalidate_nonexistent_task_returns_none() {
    let (db, _root) = open_temp_db("invalidate-nonexistent");
    let repo = SqliteSessionRepository::from_database(db);
    let result = repo.invalidate_task(9999, None).expect("should not error");
    assert!(result.is_none());
}

#[test]
fn get_task_report_none_before_close() {
    let (db, _root) = open_temp_db("report-none");
    let repo = SqliteSessionRepository::from_database(db);
    let session_id = create_session(&repo, "Report parent");
    let task = repo
        .create_task(&NewTask {
            session_id,
            name: "task/open".to_string(),
        })
        .unwrap();

    assert!(repo
        .get_task_report(task.id.unwrap())
        .expect("get_task_report should succeed")
        .is_none());
}

#[test]
fn task_status_roundtrip_strings() {
    assert_eq!(TaskStatus::try_from("open").unwrap(), TaskStatus::Open);
    assert_eq!(TaskStatus::try_from("closed").unwrap(), TaskStatus::Closed);
    assert_eq!(
        TaskStatus::try_from("invalid").unwrap(),
        TaskStatus::Invalid
    );
    assert!(matches!(
        TaskStatus::try_from("bogus"),
        Err(TrackerError::InvalidTaskStatus(_))
    ));
    assert_eq!(TaskStatus::Open.to_string(), "open");
    assert_eq!(TaskStatus::Closed.to_string(), "closed");
    assert_eq!(TaskStatus::Invalid.to_string(), "invalid");
}
