use nxm_tracker::{init_logging, product_name, TrackerConfig};

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
        }
        Err(e) => {
            tracing::error!(target: "nexum::tracker::main", error = %e, "failed to load configuration");
            eprintln!("failed to load configuration: {e}");
            std::process::exit(1);
        }
    }
}
