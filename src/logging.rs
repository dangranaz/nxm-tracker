//! Logging for `nxm-tracker`, built on [`nxm-logging`].

use crate::error::TrackerError;
use nxm_logging::init_product_logging;

/// Initializes structured logging for `nxm-tracker`.
///
/// Logs are written in JSON to `~/.nxmconfig/logs/nxm-tracker.log` (with daily
/// rotation, keeping up to 7 files) and to stderr. The log level is resolved
/// from `NXM_MCP_LOG`, then `RUST_LOG`, then defaults to `"info"`.
///
/// Safe to call multiple times — only the first call installs the subscriber
/// ("first wins").
///
/// # Errors
///
/// Returns [`TrackerError::Logging`] if logging cannot be initialized.
pub fn init_logging() -> Result<(), TrackerError> {
    init_product_logging("nxm-tracker").map_err(TrackerError::Logging)
}
