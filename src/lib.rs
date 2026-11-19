//! `nxm_tracker` — tracks long, multi-day work sessions and the integrated
//! feature sessions created after the design phase.
//!
//! Business logic is not implemented yet: this is the minimal scaffold.

/// Returns the canonical product name of this crate.
///
/// # Examples
///
/// ```
/// use nxm_tracker::product_name;
///
/// assert_eq!(product_name(), "nxm-tracker");
/// ```
pub fn product_name() -> &'static str {
    tracing::debug!(target: "nexum::tracker::api", "product name requested");
    "nxm-tracker"
}
