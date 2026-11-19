//! Smoke tests for the `nxm-tracker` scaffold.

#[test]
fn product_name_is_the_canonical_binary_name() {
    assert_eq!(nxm_tracker::product_name(), "nxm-tracker");
}

#[test]
fn product_name_matches_the_cargo_package_name() {
    assert_eq!(
        nxm_tracker::product_name(),
        env!("CARGO_PKG_NAME").replace('_', "-")
    );
}
