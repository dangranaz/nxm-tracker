//! Esempio: flusso completo end-to-end.
//!
//! Copre: crea sessione → leggi → aggiorna (nome/stato/design_phase_complete)
//! → chiudi → crea feature session → lista feature session.
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
    SessionStatus, SqliteSessionRepository,
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

    // 1. Crea una nuova sessione di lavoro.
    let session = repo.create_session(&NewSession {
        name: "Migrazione database".to_string(),
        started_at: "2026-11-20T08:00:00+01:00".to_string(),
    })?;
    let id = session.id.expect("id atteso");
    println!("1. Creata sessione: id={}, nome={}", id, session.name);

    // 2. Leggi la sessione appena creata.
    let fetched = repo.get_session(id)?.expect("sessione deve esistere");
    assert_eq!(fetched.name, "Migrazione database");
    assert_eq!(fetched.status, SessionStatus::Open);
    assert!(!fetched.design_phase_complete);
    println!("2. Letta sessione: stato={:?}, design_phase_complete={}", fetched.status, fetched.design_phase_complete);

    // 3. Aggiorna: cambia nome, imposta in pausa, segna design phase completa.
    let updated = repo.update_session(
        id,
        Some("Migrazione database (in pausa)"),
        Some(SessionStatus::Paused),
        Some(true),
    )?;
    assert!(updated);
    let fetched = repo.get_session(id)?.expect("sessione deve esistere");
    assert_eq!(fetched.name, "Migrazione database (in pausa)");
    assert_eq!(fetched.status, SessionStatus::Paused);
    assert!(fetched.design_phase_complete);
    println!("3. Aggiornata sessione: nome={}, stato={:?}, design_phase_complete={}", fetched.name, fetched.status, fetched.design_phase_complete);

    // 4. Chiudi la sessione.
    let closed = repo.close_session(id)?;
    assert!(closed);
    let fetched = repo.get_session(id)?.expect("sessione deve esistere");
    assert_eq!(fetched.status, SessionStatus::Closed);
    assert!(fetched.ended_at.is_some());
    println!("4. Chiusa sessione: stato={:?}, ended_at={:?}", fetched.status, fetched.ended_at);

    // 5. Crea una feature session collegata (dopo la design phase).
    let feature = repo.create_feature_session(&NewFeatureSession {
        session_id: id,
        name: "Script di migrazione SQL".to_string(),
        description: Some("Genera script SQL per la migrazione dello schema".to_string()),
    })?;
    println!(
        "5. Creata feature session: id={:?}, nome={}",
        feature.id, feature.name
    );

    // 6. Lista tutte le feature session della work session.
    let features = repo.list_feature_sessions(id)?;
    assert_eq!(features.len(), 1);
    assert_eq!(features[0].name, "Script di migrazione SQL");
    println!("6. Lista feature session: {} trovata/e", features.len());

    println!("\nFlusso completo completato con successo.");
    Ok(())
}
