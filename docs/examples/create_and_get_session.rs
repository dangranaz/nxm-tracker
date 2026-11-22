//! Esempio: crea una sessione di lavoro e la lettura.
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
    // --- Setup database temporaneo (come nei test del crate) ---
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

    // 1. Crea una nuova sessione di lavoro.
    let new = NewSession {
        name: "Refactoring Q4".to_string(),
        started_at: "2026-11-20T08:00:00+01:00".to_string(),
    };
    let created = repo.create_session(&new)?;
    let id = created.id.expect("la sessione creata deve avere un id");
    println!(
        "Sessione creata: id={}, nome={}, stato={:?}, design_phase_complete={}",
        id, created.name, created.status, created.design_phase_complete
    );

    // 2. Leggi la sessione per ID.
    let fetched = repo.get_session(id)?.expect("la sessione deve esistere");
    println!(
        "Sessione letta: nome={}, started_at={}, ended_at={:?}, design_phase_complete={}",
        fetched.name, fetched.started_at, fetched.ended_at, fetched.design_phase_complete
    );

    Ok(())
}
