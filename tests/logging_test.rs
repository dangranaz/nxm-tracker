//! Logging tests for `nxm-tracker`, built on `nxm-logging`.

use nxm_tracker::init_logging;

#[test]
fn init_logging_first_wins() {
    // First call succeeds (installs the global subscriber).
    let first = init_logging();
    assert!(first.is_ok(), "first init_logging should succeed");

    // "first wins": subsequent calls return an error but do not panic.
    let second = init_logging();
    assert!(
        second.is_err(),
        "second init_logging should return an error (subscriber already set)"
    );
}
