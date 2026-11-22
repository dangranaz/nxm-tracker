//! Esempio: chiude una sessione di lavoro (imposta status=closed e ended_at=now).
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
    default_migration_set, Database, NewSession, SessionRepository, SqliteSessionRepository,
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

    // 1. Crea una sessione aperta.
    let new = NewSession {
        name: "Sessione da chiudere".to_string(),
        started_at: "2026-11-20T08:00:00+01:00".to_string(),
    };
    let created = repo.create_session(&new)?;
    let id = created.id.expect("id atteso");
    assert_eq!(created.status, nxm_tracker::SessionStatus::Open);
    assert!(created.ended_at.is_none());
    println!("Sessione creata (aperta): id={}", id);

    // 2. Chiudi la sessione.
    let closed = repo.close_session(id)?;
    assert!(closed, "chiusura deve riuscire");

    // 3. Verifica lo stato.
    let fetched = repo.get_session(id)?.expect("sessione deve esistere");
    assert_eq!(fetched.status, nxm_tracker::SessionStatus::Closed);
    assert!(fetched.ended_at.is_some(), "ended_at deve essere impostato");
    println!(
        "Sessione chiusa: stato={:?}, ended_at={:?}",
        fetched.status, fetched.ended_at
    );

    Ok(())
}
