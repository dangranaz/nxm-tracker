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
fn default_migration_set_has_one_migration() {
    let set = default_migration_set().expect("default migration set should be valid");
    assert_eq!(set.migrations().len(), 1);
    assert_eq!(set.migrations()[0].version, 1);
    assert_eq!(set.migrations()[0].name, "create_sessions");
}
