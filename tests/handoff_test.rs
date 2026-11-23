//! Integration tests for handoffs and activity plans.
//!
//! Uses temporary directories so the real `~/.nxm/shared-db` is never touched.

use nxm_db::DbConfig;
use nxm_tracker::{
    Database, HandoffRepository, NewSession, SessionRepository, SqliteSessionRepository,
};
use std::fs;

fn open_temp_db(suffix: &str) -> (Database, std::path::PathBuf) {
    let root = std::env::temp_dir().join(format!(
        "nxm-tracker-handoff-{}-{}",
        suffix,
        std::process::id()
    ));
    let _ = fs::remove_dir_all(&root);

    let db_path = root.join("handoff.db");
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
fn create_session_full_creates_handoff_and_plan() {
    let (db, _root) = open_temp_db("full-create");
    let repo = SqliteSessionRepository::from_database(db);

    let new = NewSession {
        name: "Full session".to_string(),
        started_at: "2026-11-22T09:00:00+01:00".to_string(),
    };
    let plan = vec!["setup db".to_string(), "write tests".to_string()];
    let (session, handoff, plans) = repo
        .create_session_full(&new, Some("initial handoff text"), &plan)
        .expect("create_session_full should succeed");

    let session_id = session.id.unwrap();
    assert_eq!(handoff.as_ref().unwrap().session_id, session_id);
    assert_eq!(handoff.as_ref().unwrap().content, "initial handoff text");
    assert_eq!(plans.len(), 2);
    assert_eq!(plans[0].position, 1);
    assert_eq!(plans[0].description, "setup db");
    assert_eq!(plans[1].position, 2);

    let stored_handoff = repo
        .get_handoff(session_id)
        .expect("get_handoff should succeed")
        .expect("handoff should exist");
    assert_eq!(stored_handoff.content, "initial handoff text");

    let stored_plans = repo
        .list_activity_plans(session_id)
        .expect("list_activity_plans should succeed");
    assert_eq!(stored_plans.len(), 2);
    assert_eq!(stored_plans[0].description, "setup db");
    assert_eq!(stored_plans[1].description, "write tests");
}

#[test]
fn create_session_full_without_handoff_and_plan() {
    let (db, _root) = open_temp_db("full-minimal");
    let repo = SqliteSessionRepository::from_database(db);

    let new = NewSession {
        name: "Minimal".to_string(),
        started_at: "2026-11-22T09:00:00+01:00".to_string(),
    };
    let (session, handoff, plans) = repo
        .create_session_full(&new, None, &[])
        .expect("create_session_full should succeed");

    assert!(handoff.is_none());
    assert!(plans.is_empty());
    assert!(repo
        .get_handoff(session.id.unwrap())
        .expect("get_handoff should succeed")
        .is_none());
}

#[test]
fn get_handoff_returns_none_for_session_without_handoff() {
    let (db, _root) = open_temp_db("no-handoff");
    let repo = SqliteSessionRepository::from_database(db);
    let session_id = create_session(&repo, "No handoff");

    assert!(repo
        .get_handoff(session_id)
        .expect("get_handoff should succeed")
        .is_none());
}

#[test]
fn upsert_handoff_creates_then_updates() {
    let (db, _root) = open_temp_db("upsert");
    let repo = SqliteSessionRepository::from_database(db);
    let session_id = create_session(&repo, "Upsert target");

    let created = repo
        .upsert_handoff(session_id, "first version")
        .expect("first upsert should succeed");
    assert_eq!(created.content, "first version");

    let updated = repo
        .upsert_handoff(session_id, "second version")
        .expect("second upsert should succeed");
    assert_eq!(updated.content, "second version");
    assert_eq!(
        updated.id, created.id,
        "upsert must keep the same row (1:1)"
    );

    // Still exactly one handoff row for the session.
    let stored = repo
        .get_handoff(session_id)
        .expect("get_handoff should succeed")
        .expect("handoff should exist");
    assert_eq!(stored.content, "second version");
}

#[test]
fn list_activity_plans_empty_for_session_without_plan() {
    let (db, _root) = open_temp_db("empty-plan");
    let repo = SqliteSessionRepository::from_database(db);
    let session_id = create_session(&repo, "No plan");

    let plans = repo
        .list_activity_plans(session_id)
        .expect("list_activity_plans should succeed");
    assert!(plans.is_empty());
}

#[test]
fn plans_are_isolated_per_session() {
    let (db, _root) = open_temp_db("isolated");
    let repo = SqliteSessionRepository::from_database(db);
    let s1 = create_session(&repo, "Session one");
    let s2 = create_session(&repo, "Session two");

    let new = NewSession {
        name: "With plan".to_string(),
        started_at: "2026-11-22T09:00:00+01:00".to_string(),
    };
    let (s3, _, _) = repo
        .create_session_full(&new, None, &["only step".to_string()])
        .expect("create with plan should succeed");
    let s3 = s3.id.unwrap();

    assert_eq!(
        repo.list_activity_plans(s3).unwrap().len(),
        1,
        "session with plan should have one item"
    );
    assert!(
        repo.list_activity_plans(s1).unwrap().is_empty()
            && repo.list_activity_plans(s2).unwrap().is_empty()
    );
}
