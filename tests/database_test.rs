//! Database tests for `nxm-tracker`, built on `nxm-db`.
//!
//! Uses temporary directories so the real `~/.nxm/shared-db` is never touched.

use nxm_db::DbConfig;
use nxm_tracker::{default_migration_set, Database};
use std::fs;

#[test]
fn database_opens_in_temp_directory() {
    let root = std::env::temp_dir().join(format!("nxm-tracker-db-{}", std::process::id()));
    let _ = fs::remove_dir_all(&root);

    let db_path = root.join("test.db");
    let config = DbConfig {
        backend: nxm_db::Backend::Sqlite,
        url: db_path.to_string_lossy().to_string(),
        tenant: "default".to_string(),
    };

    let _db = Database::open_from(&config).expect("database should open in temp directory");
    assert!(db_path.exists(), "database file should be created");

    fs::remove_dir_all(&root).expect("temporary root should be removed");
}

#[test]
fn database_migration_set_applies() {
    let root = std::env::temp_dir().join(format!("nxm-tracker-migrate-{}", std::process::id()));
    let _ = fs::remove_dir_all(&root);

    let db_path = root.join("migrate.db");
    let config = DbConfig {
        backend: nxm_db::Backend::Sqlite,
        url: db_path.to_string_lossy().to_string(),
        tenant: "default".to_string(),
    };

    let db = Database::open_from(&config).expect("database should open");
    let set = default_migration_set().expect("default migration set should be valid");
    db.migrate(&set)
        .expect("migration should apply successfully");

    // Re-running should be a no-op (already applied)
    db.migrate(&set)
        .expect("re-applying migration should succeed");

    fs::remove_dir_all(&root).expect("temporary root should be removed");
}

#[test]
fn default_migration_set_has_fifteen_migrations() {
    let set = default_migration_set().expect("default migration set should be valid");
    assert_eq!(set.migrations().len(), 15);
    assert_eq!(set.migrations()[0].version, 1);
    assert_eq!(set.migrations()[0].name, "create_sessions");
    assert_eq!(set.migrations()[1].version, 2);
    assert_eq!(set.migrations()[1].name, "create_feature_sessions");
    assert_eq!(set.migrations()[2].version, 3);
    assert_eq!(set.migrations()[2].name, "create_lifecycle_state");
    assert_eq!(set.migrations()[3].version, 4);
    assert_eq!(set.migrations()[3].name, "seed_lifecycle_state");
    assert_eq!(set.migrations()[4].version, 5);
    assert_eq!(set.migrations()[4].name, "create_lifecycle_transition");
    assert_eq!(set.migrations()[5].version, 6);
    assert_eq!(set.migrations()[5].name, "seed_lifecycle_transition");
    assert_eq!(set.migrations()[6].version, 7);
    assert_eq!(set.migrations()[6].name, "create_session");
    assert_eq!(set.migrations()[7].version, 8);
    assert_eq!(set.migrations()[7].name, "create_audit_log");
    assert_eq!(set.migrations()[8].version, 9);
    assert_eq!(set.migrations()[8].name, "create_task_table");
    assert_eq!(set.migrations()[9].version, 10);
    assert_eq!(set.migrations()[9].name, "create_task_revision_table");
    assert_eq!(set.migrations()[10].version, 11);
    assert_eq!(set.migrations()[10].name, "create_task_audit_table");
    assert_eq!(set.migrations()[11].version, 12);
    assert_eq!(set.migrations()[11].name, "create_session_handoff");
    assert_eq!(set.migrations()[12].version, 13);
    assert_eq!(set.migrations()[12].name, "create_slug_counter");
    assert_eq!(set.migrations()[13].version, 14);
    assert_eq!(set.migrations()[13].name, "create_prompt");
    assert_eq!(set.migrations()[14].version, 15);
    assert_eq!(set.migrations()[14].name, "create_retention_log");
}
