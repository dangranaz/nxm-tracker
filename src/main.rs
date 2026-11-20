use nxm_tracker::{default_migration_set, init_logging, product_name, Database, TrackerConfig};

fn main() {
    if let Err(e) = init_logging() {
        eprintln!("failed to initialize logging: {e}");
        std::process::exit(1);
    }

    match TrackerConfig::load() {
        Ok(config) => {
            tracing::info!(target: "nexum::tracker::main", "starting nxm-tracker");
            println!("{} {}", product_name(), env!("CARGO_PKG_VERSION"));
            println!("  server port: {}", config.server_port());
            println!("  database url: {}", config.database_url());
            println!("  database driver: {:?}", config.database_driver());

            match Database::open() {
                Ok(db) => {
                    println!("  database: opened successfully");
                    match default_migration_set() {
                        Ok(set) => match db.migrate(&set) {
                            Ok(()) => println!("  migrations: applied successfully"),
                            Err(e) => {
                                tracing::error!(target: "nexum::tracker::main", error = %e, "migration failed");
                                eprintln!("migration failed: {e}");
                                std::process::exit(1);
                            }
                        },
                        Err(e) => {
                            tracing::error!(target: "nexum::tracker::main", error = %e, "migration set creation failed");
                            eprintln!("migration set creation failed: {e}");
                            std::process::exit(1);
                        }
                    }
                }
                Err(e) => {
                    tracing::error!(target: "nexum::tracker::main", error = %e, "database open failed");
                    eprintln!("database open failed: {e}");
                    std::process::exit(1);
                }
            }
        }
        Err(e) => {
            tracing::error!(target: "nexum::tracker::main", error = %e, "failed to load configuration");
            eprintln!("failed to load configuration: {e}");
            std::process::exit(1);
        }
    }
}
