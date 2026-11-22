//! Esempio: aggiorna una sessione esistente (nome, stato, design_phase_complete).
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
    default_migration_set, Database, NewSession, SessionRepository, SessionStatus,
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

    // 1. Crea una sessione.
    let new = NewSession {
        name: "Progetto originale".to_string(),
        started_at: "2026-11-20T08:00:00+01:00".to_string(),
    };
    let created = repo.create_session(&new)?;
    let id = created.id.expect("id atteso");
    println!("Sessione creata: id={}, nome={}", id, created.name);

    // 2. Aggiorna nome e stato.
    let updated = repo.update_session(
        id,
        Some("Progetto rinominato"),
        Some(SessionStatus::Paused),
        None,
    )?;
    assert!(updated, "aggiornamento deve riuscire");
    let fetched = repo.get_session(id)?.expect("sessione deve esistere");
    assert_eq!(fetched.name, "Progetto rinominato");
    assert_eq!(fetched.status, SessionStatus::Paused);
    println!(
        "Sessione aggiornata: nome={}, stato={:?}",
        fetched.name, fetched.status
    );

    // 3. Aggiorna solo il flag design_phase_complete.
    let updated = repo.update_session(id, None, None, Some(true))?;
    assert!(updated);
    let fetched = repo.get_session(id)?.expect("sessione deve esistere");
    assert!(fetched.design_phase_complete);
    println!("design_phase_complete impostato a: {}", fetched.design_phase_complete);

    Ok(())
}
