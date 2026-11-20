//! Configuration for `nxm-tracker`, built on [`nxm-config`].

use crate::error::TrackerError;
use nxm_config::{load_config, load_or_create_config, save_config, ConfigPaths, GeneralConfig};

/// Tracker-specific configuration wrapper around [`nxm_config::GeneralConfig`].
#[derive(Debug, Clone, Default)]
pub struct TrackerConfig {
    pub inner: GeneralConfig,
}

impl TrackerConfig {
    /// Discovers the config root (`NXM_CONFIG_HOME` or `~/.nxmconfig`) and
    /// loads the configuration, creating a default file when none exists.
    pub fn load() -> Result<Self, TrackerError> {
        let paths = ConfigPaths::discover()?;
        Self::load_or_create_from(&paths)
    }

    /// Loads (or creates) configuration from an explicit [`ConfigPaths`].
    pub fn load_or_create_from(paths: &ConfigPaths) -> Result<Self, TrackerError> {
        let inner = load_or_create_config(paths).map_err(TrackerError::Config)?;
        Ok(Self { inner })
    }

    /// Loads configuration from an explicit [`ConfigPaths`] without creating
    /// a default file.
    pub fn load_from(paths: &ConfigPaths) -> Result<Self, TrackerError> {
        let inner = load_config(paths).map_err(TrackerError::Config)?;
        Ok(Self { inner })
    }

    /// Saves the current configuration to the discovered config root.
    pub fn save(&self) -> Result<(), TrackerError> {
        let paths = ConfigPaths::discover()?;
        self.save_to(&paths)
    }

    /// Saves the current configuration to an explicit [`ConfigPaths`].
    pub fn save_to(&self, paths: &ConfigPaths) -> Result<(), TrackerError> {
        save_config(paths, &self.inner).map_err(TrackerError::Config)
    }

    /// Returns the MCP/HTTP server port.
    pub fn server_port(&self) -> u16 {
        self.inner.server.port
    }

    /// Returns the database connection URL.
    pub fn database_url(&self) -> &str {
        &self.inner.database.url
    }

    /// Returns the database driver kind.
    pub fn database_driver(&self) -> nxm_config::DatabaseKind {
        self.inner.database.driver
    }
}
