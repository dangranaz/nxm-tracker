//! Esempio: crea una feature session e lista tutte le feature session di una work session.
//!
//! Aggiungi al tuo `Cargo.toml`:
//! ```toml
//! [dependencies]
//! nxm-tracker = { path = "/percorso/al/crate/nxm-tracker" }
//! ```
//!
//! Poi copia questo file nel tuo progetto ed eseguilo.

use nxm_db::{Backend, DbConfig};
use nxm_tracker::{
    default_migration_set, Database, NewFeatureSession, NewSession, SessionRepository,
    SqliteSessionRepository,
};
use std::fs;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // --- Setup database temporaneo ---
    let root = std::env::temp_dir().join(format!("nxm-example-{}", std::process::id()));
    let _ = fs::remove_dir_all(&root);
    let db_path = root.join("example.db");
    let config = DbConfig {
        backend: Backend::Sqlite,
        url: db_path.to_string_lossy().to_string(),
        tenant: "default".to_string(),
    };
    let db = Database::open_from(&config)?;
    let migration_set = default_migration_set()?;
    db.migrate(&migration_set)?;

    let repo = SqliteSessionRepository::from_database(db);

    // 1. Crea una work session padre.
    let session = repo.create_session(&NewSession {
        name: "Sviluppo OAuth".to_string(),
        started_at: "2026-11-20T08:00:00+01:00".to_string(),
    })?;
    let session_id = session.id.expect("id atteso");
    println!("Work session creata: id={}", session_id);

    // 2. Crea una feature session collegata.
    let feature = repo.create_feature_session(&NewFeatureSession {
        session_id,
        name: "Implementa login OIDC".to_string(),
        description: Some("Aggiungi flusso di login OpenID Connect".to_string()),
    })?;
    println!(
        "Feature session creata: id={:?}, nome={}, descrizione={:?}, stato={}",
        feature.id, feature.name, feature.description, feature.status
    );

    // 3. Crea una seconda feature session.
    let feature2 = repo.create_feature_session(&NewFeatureSession {
        session_id,
        name: "Aggiungi logout".to_string(),
        description: None,
    })?;
    println!(
        "Feature session creata: id={:?}, nome={}",
        feature2.id, feature2.name
    );

    // 4. Lista tutte le feature session della work session.
    let features = repo.list_feature_sessions(session_id)?;
    println!("Totale feature session per la work session {}: {}", session_id, features.len());
    for f in &features {
        println!("  - id={:?}, nome={}", f.id, f.name);
    }
    assert_eq!(features.len(), 2);

    Ok(())
}
