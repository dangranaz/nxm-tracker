//! Integration tests for the session model.
//!
//! Uses temporary directories so the real `~/.nxm/shared-db` is never touched.

use nxm_db::DbConfig;
use nxm_tracker::{
    Database, NewFeatureSession, NewSession, SessionRepository, SessionStatus,
    SqliteSessionRepository,
};
use std::fs;

fn temp_db(suffix: &str) -> (DbConfig, std::path::PathBuf) {
    let root = std::env::temp_dir().join(format!(
        "nxm-tracker-session-{}-{}",
        suffix,
        std::process::id()
    ));
    let _ = fs::remove_dir_all(&root);

    let db_path = root.join("session.db");
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

#[test]
fn create_and_get_session() {
    let (db, _root) = open_temp_db("create-get");
    let repo = SqliteSessionRepository::from_database(db);

    let new = NewSession {
        name: "Q4 refactoring".to_string(),
        started_at: "2026-11-20T08:00:00+01:00".to_string(),
    };
    let created = repo
        .create_session(&new)
        .expect("create session should succeed");
    let id = created.id.expect("created session should have an id");

    let fetched = repo.get_session(id).expect("get session should succeed");
    assert!(fetched.is_some(), "session should exist");
    let fetched = fetched.unwrap();
    assert_eq!(fetched.name, "Q4 refactoring");
    assert_eq!(fetched.status, SessionStatus::Open);
    assert!(!fetched.design_phase_complete);
    assert!(fetched.ended_at.is_none());
}

#[test]
fn get_nonexistent_session_returns_none() {
    let (db, _root) = open_temp_db("get-nonexistent");
    let repo = SqliteSessionRepository::from_database(db);

    let fetched = repo.get_session(9999).expect("get should not error");
    assert!(fetched.is_none(), "nonexistent session should return None");
}

#[test]
fn list_sessions_returns_all_ordered() {
    let (db, _root) = open_temp_db("list-sessions");
    let repo = SqliteSessionRepository::from_database(db);

    repo.create_session(&NewSession {
        name: "First".to_string(),
        started_at: "2026-11-20T08:00:00+01:00".to_string(),
    })
    .expect("create first");
    repo.create_session(&NewSession {
        name: "Second".to_string(),
        started_at: "2026-11-21T08:00:00+01:00".to_string(),
    })
    .expect("create second");

    let sessions = repo.list_sessions().expect("list should succeed");
    assert_eq!(sessions.len(), 2);
    // Ordered by started_at DESC: Second first
    assert_eq!(sessions[0].name, "Second");
    assert_eq!(sessions[1].name, "First");
}

#[test]
fn set_session_status_changes_status() {
    let (db, _root) = open_temp_db("set-status");
    let repo = SqliteSessionRepository::from_database(db);

    let created = repo
        .create_session(&NewSession {
            name: "Status change".to_string(),
            started_at: "2026-11-20T08:00:00+01:00".to_string(),
        })
        .expect("create");
    let id = created.id.expect("id");

    assert!(repo
        .set_session_status(id, SessionStatus::Paused)
        .expect("status update"));
    let fetched = repo.get_session(id).expect("get").unwrap();
    assert_eq!(fetched.status, SessionStatus::Paused);

    assert!(repo
        .set_session_status(id, SessionStatus::Open)
        .expect("status update"));
    let fetched = repo.get_session(id).expect("get").unwrap();
    assert_eq!(fetched.status, SessionStatus::Open);
}

#[test]
fn set_session_status_nonexistent_returns_false() {
    let (db, _root) = open_temp_db("set-status-nonexistent");
    let repo = SqliteSessionRepository::from_database(db);

    let result = repo.set_session_status(9999, SessionStatus::Closed);
    assert!(
        !result.expect("update should not error"),
        "nonexistent should return false"
    );
}

#[test]
fn close_session_sets_ended_at_and_status() {
    let (db, _root) = open_temp_db("close-session");
    let repo = SqliteSessionRepository::from_database(db);

    let created = repo
        .create_session(&NewSession {
            name: "To close".to_string(),
            started_at: "2026-11-20T08:00:00+01:00".to_string(),
        })
        .expect("create");
    let id = created.id.expect("id");

    assert!(repo.close_session(id).expect("close should succeed"));
    let fetched = repo.get_session(id).expect("get").unwrap();
    assert_eq!(fetched.status, SessionStatus::Closed);
    assert!(fetched.ended_at.is_some(), "ended_at should be set");
}

#[test]
fn close_nonexistent_session_returns_false() {
    let (db, _root) = open_temp_db("close-nonexistent");
    let repo = SqliteSessionRepository::from_database(db);

    let result = repo.close_session(9999);
    assert!(
        !result.expect("close should not error"),
        "nonexistent should return false"
    );
}

#[test]
fn set_design_phase_complete_toggles_flag() {
    let (db, _root) = open_temp_db("design-phase");
    let repo = SqliteSessionRepository::from_database(db);

    let created = repo
        .create_session(&NewSession {
            name: "Design phase".to_string(),
            started_at: "2026-11-20T08:00:00+01:00".to_string(),
        })
        .expect("create");
    let id = created.id.expect("id");

    assert!(!created.design_phase_complete, "default should be false");

    assert!(repo
        .set_design_phase_complete(id, true)
        .expect("flag update"));
    let fetched = repo.get_session(id).expect("get").unwrap();
    assert!(fetched.design_phase_complete, "flag should be true");

    assert!(repo
        .set_design_phase_complete(id, false)
        .expect("flag update"));
    let fetched = repo.get_session(id).expect("get").unwrap();
    assert!(!fetched.design_phase_complete, "flag should be false again");
}

#[test]
fn create_and_list_feature_sessions() {
    let (db, _root) = open_temp_db("feature-sessions");
    let repo = SqliteSessionRepository::from_database(db);

    let created = repo
        .create_session(&NewSession {
            name: "Parent session".to_string(),
            started_at: "2026-11-20T08:00:00+01:00".to_string(),
        })
        .expect("create parent");
    let session_id = created.id.expect("id");

    let feature = repo
        .create_feature_session(&NewFeatureSession {
            session_id,
            name: "Implement OAuth".to_string(),
            description: Some("Add OIDC login flow".to_string()),
        })
        .expect("create feature");
    assert_eq!(feature.session_id, session_id);
    assert_eq!(feature.name, "Implement OAuth");
    assert_eq!(feature.description.as_deref(), Some("Add OIDC login flow"));
    assert_eq!(feature.status, "open");

    let features = repo
        .list_feature_sessions(session_id)
        .expect("list features");
    assert_eq!(features.len(), 1);
    assert_eq!(features[0].id, feature.id);
}

#[test]
fn list_feature_sessions_empty_for_session_without_features() {
    let (db, _root) = open_temp_db("feature-sessions-empty");
    let repo = SqliteSessionRepository::from_database(db);

    let created = repo
        .create_session(&NewSession {
            name: "No features".to_string(),
            started_at: "2026-11-20T08:00:00+01:00".to_string(),
        })
        .expect("create");
    let session_id = created.id.expect("id");

    let features = repo
        .list_feature_sessions(session_id)
        .expect("list features");
    assert!(features.is_empty(), "no feature sessions yet");
}

#[test]
fn feature_session_belongs_to_correct_session() {
    let (db, _root) = open_temp_db("feature-session-belongs");
    let repo = SqliteSessionRepository::from_database(db);

    let s1 = repo
        .create_session(&NewSession {
            name: "Session 1".to_string(),
            started_at: "2026-11-20T08:00:00+01:00".to_string(),
        })
        .expect("create s1");
    let s2 = repo
        .create_session(&NewSession {
            name: "Session 2".to_string(),
            started_at: "2026-11-21T08:00:00+01:00".to_string(),
        })
        .expect("create s2");

    repo.create_feature_session(&NewFeatureSession {
        session_id: s1.id.expect("id"),
        name: "Feature for s1".to_string(),
        description: None,
    })
    .expect("create f1");
    repo.create_feature_session(&NewFeatureSession {
        session_id: s2.id.expect("id"),
        name: "Feature for s2".to_string(),
        description: None,
    })
    .expect("create f2");

    let f1 = repo
        .list_feature_sessions(s1.id.expect("id"))
        .expect("list f1");
    assert_eq!(f1.len(), 1);
    assert_eq!(f1[0].name, "Feature for s1");

    let f2 = repo
        .list_feature_sessions(s2.id.expect("id"))
        .expect("list f2");
    assert_eq!(f2.len(), 1);
    assert_eq!(f2[0].name, "Feature for s2");
}

#[test]
fn search_sessions_by_name() {
    let (db, _root) = open_temp_db("search-by-name");
    let repo = SqliteSessionRepository::from_database(db);

    repo.create_session(&NewSession {
        name: "Refactoring Q4".to_string(),
        started_at: "2026-11-20T08:00:00+01:00".to_string(),
    })
    .expect("create refactoring");
    repo.create_session(&NewSession {
        name: "Q4 planning".to_string(),
        started_at: "2026-11-21T08:00:00+01:00".to_string(),
    })
    .expect("create planning");
    repo.create_session(&NewSession {
        name: "Release notes".to_string(),
        started_at: "2026-11-22T08:00:00+01:00".to_string(),
    })
    .expect("create release");

    let results = repo
        .search_sessions_by_name("Q4")
        .expect("search should succeed");
    assert_eq!(results.len(), 2, "should find 2 sessions containing 'Q4'");
    assert!(results.iter().any(|s| s.name == "Refactoring Q4"));
    assert!(results.iter().any(|s| s.name == "Q4 planning"));

    let results = repo
        .search_sessions_by_name("Release")
        .expect("search should succeed");
    assert_eq!(results.len(), 1);
    assert_eq!(results[0].name, "Release notes");

    let results = repo
        .search_sessions_by_name("nonexistent")
        .expect("search should succeed");
    assert!(results.is_empty(), "no sessions should match");
}

#[test]
fn list_sessions_by_status() {
    let (db, _root) = open_temp_db("list-by-status");
    let repo = SqliteSessionRepository::from_database(db);

    let s1 = repo
        .create_session(&NewSession {
            name: "Open session".to_string(),
            started_at: "2026-11-20T08:00:00+01:00".to_string(),
        })
        .expect("create s1");
    let s2 = repo
        .create_session(&NewSession {
            name: "Paused session".to_string(),
            started_at: "2026-11-21T08:00:00+01:00".to_string(),
        })
        .expect("create s2");
    repo.set_session_status(s2.id.expect("id"), SessionStatus::Paused)
        .expect("pause s2");

    let open = repo
        .list_sessions_by_status(SessionStatus::Open)
        .expect("list open");
    assert_eq!(open.len(), 1);
    assert_eq!(open[0].id, Some(s1.id.expect("id")));
    assert_eq!(open[0].name, "Open session");

    let paused = repo
        .list_sessions_by_status(SessionStatus::Paused)
        .expect("list paused");
    assert_eq!(paused.len(), 1);
    assert_eq!(paused[0].name, "Paused session");

    let closed = repo
        .list_sessions_by_status(SessionStatus::Closed)
        .expect("list closed");
    assert!(closed.is_empty(), "no closed sessions");
}

#[test]
fn list_sessions_paginated() {
    let (db, _root) = open_temp_db("paginated");
    let repo = SqliteSessionRepository::from_database(db);

    for i in 1..=5 {
        repo.create_session(&NewSession {
            name: format!("Session {}", i),
            started_at: format!("2026-11-2{}T08:00:00+01:00", i),
        })
        .expect("create session");
    }

    // First page: limit 2, offset 0
    let page1 = repo.list_sessions_paginated(0, 2).expect("page 1");
    assert_eq!(page1.len(), 2);
    // Ordered DESC: Session 5 first
    assert_eq!(page1[0].name, "Session 5");
    assert_eq!(page1[1].name, "Session 4");

    // Second page: limit 2, offset 2
    let page2 = repo.list_sessions_paginated(2, 2).expect("page 2");
    assert_eq!(page2.len(), 2);
    assert_eq!(page2[0].name, "Session 3");
    assert_eq!(page2[1].name, "Session 2");

    // Third page: limit 2, offset 4
    let page3 = repo.list_sessions_paginated(4, 2).expect("page 3");
    assert_eq!(page3.len(), 1);
    assert_eq!(page3[0].name, "Session 1");

    // Beyond last page: empty
    let page4 = repo.list_sessions_paginated(10, 2).expect("page 4");
    assert!(page4.is_empty());
}
