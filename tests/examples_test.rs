//! Test di verifica degli esempi in `docs/examples/`.
//!
//! Verifica che ogni file di esempio esista, non sia vuoto e contenga
//! una funzione `main()` eseguibile. La logica degli esempi è già coperta
//! dai test in `tests/session_test.rs` e `tests/mcp_tools_test.rs`.

use std::fs;
use std::path::Path;

const EXAMPLES_DIR: &str = "docs/examples";

const REQUIRED_EXAMPLES: &[&str] = &[
    "create_and_get_session.rs",
    "update_session.rs",
    "close_session.rs",
    "feature_sessions.rs",
    "full_workflow.rs",
    "mcp_payloads.md",
];

#[test]
fn docs_examples_directory_exists() {
    assert!(
        Path::new(EXAMPLES_DIR).is_dir(),
        "directory {EXAMPLES_DIR} should exist"
    );
}

#[test]
fn all_required_examples_exist() {
    for filename in REQUIRED_EXAMPLES {
        let path = Path::new(EXAMPLES_DIR).join(filename);
        assert!(
            path.is_file(),
            "example file {filename} should exist in {EXAMPLES_DIR}"
        );
    }
}

#[test]
fn rust_examples_contain_main_function() {
    for filename in REQUIRED_EXAMPLES {
        if !filename.ends_with(".rs") {
            continue; // mcp_payloads.md non è Rust
        }
        let path = Path::new(EXAMPLES_DIR).join(filename);
        let content =
            fs::read_to_string(path).unwrap_or_else(|_| panic!("could not read {filename}"));
        assert!(!content.is_empty(), "{filename} should not be empty");
        assert!(
            content.contains("fn main"),
            "{filename} should contain a fn main() entry point"
        );
        assert!(
            content.contains("nxm_tracker"),
            "{filename} should use the nxm_tracker crate"
        );
    }
}

#[test]
fn mcp_payloads_covers_all_eight_tools() {
    let path = Path::new(EXAMPLES_DIR).join("mcp_payloads.md");
    let content =
        fs::read_to_string(path).unwrap_or_else(|_| panic!("could not read mcp_payloads.md"));
    for tool in [
        "create_session",
        "get_session",
        "list_sessions",
        "update_session",
        "close_session",
        "set_design_phase_complete",
        "create_feature_session",
        "list_feature_sessions",
    ] {
        assert!(
            content.contains(tool),
            "mcp_payloads.md should document the `{tool}` tool"
        );
    }
}
