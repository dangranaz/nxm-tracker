//! Configuration tests for `nxm-tracker`, built on `nxm-config`.
//!
//! Uses temporary directories so the real `~/.nxmconfig` is never touched.

use nxm_config::ConfigPaths;
use nxm_tracker::TrackerConfig;
use std::fs;

#[test]
fn tracker_config_loads_with_defaults_from_temp_root() {
    let root = std::env::temp_dir().join(format!("nxm-tracker-config-{}", std::process::id()));
    let _ = fs::remove_dir_all(&root);

    let paths = ConfigPaths::with_root(&root);
    let config =
        TrackerConfig::load_or_create_from(&paths).expect("config should load from temp root");

    // Defaults from nxm-config
    assert_eq!(config.server_port(), 8080);
    assert_eq!(config.database_url(), "data/nxm.db");
    assert_eq!(config.database_driver().as_str(), "sqlite");

    fs::remove_dir_all(&root).expect("temporary root should be removed");
}

#[test]
fn tracker_config_save_and_load_roundtrip() {
    let root = std::env::temp_dir().join(format!("nxm-tracker-roundtrip-{}", std::process::id()));
    let _ = fs::remove_dir_all(&root);

    let paths = ConfigPaths::with_root(&root);

    // Save default config
    let config = TrackerConfig::default();
    config
        .save_to(&paths)
        .expect("config should save to temp root");

    // Reload and verify values persisted
    let reloaded = TrackerConfig::load_from(&paths).expect("config should reload");
    assert_eq!(reloaded.server_port(), 8080);
    assert_eq!(reloaded.database_url(), "data/nxm.db");

    fs::remove_dir_all(&root).expect("temporary root should be removed");
}

#[test]
fn tracker_config_custom_values_persist() {
    let root = std::env::temp_dir().join(format!("nxm-tracker-custom-{}", std::process::id()));
    let _ = fs::remove_dir_all(&root);

    let paths = ConfigPaths::with_root(&root);

    // Build a custom config
    let mut config = TrackerConfig::default();
    config.inner.server.port = 9090;
    config.inner.database.url = "postgres://nxm:secret@db:5432/nxm".to_string();
    config
        .save_to(&paths)
        .expect("config should save to temp root");

    // Reload and verify custom values persisted
    let reloaded = TrackerConfig::load_from(&paths).expect("config should reload");
    assert_eq!(reloaded.server_port(), 9090);
    assert_eq!(reloaded.database_url(), "postgres://nxm:secret@db:5432/nxm");

    fs::remove_dir_all(&root).expect("temporary root should be removed");
}
