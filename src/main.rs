use nxm_tracker::{product_name, TrackerConfig};

fn main() {
    match TrackerConfig::load() {
        Ok(config) => {
            println!("{} {}", product_name(), env!("CARGO_PKG_VERSION"));
            println!("  server port: {}", config.server_port());
            println!("  database url: {}", config.database_url());
            println!("  database driver: {:?}", config.database_driver());
        }
        Err(e) => {
            eprintln!("failed to load configuration: {e}");
            std::process::exit(1);
        }
    }
}
