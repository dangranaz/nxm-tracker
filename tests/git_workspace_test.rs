//! Tests for `src/git_workspace.rs`.
//!
//! These tests create temporary git repositories to verify branch creation
//! logic without touching the user's real repositories.

use nxm_tracker::git_workspace::{
    base_branch, create_integration_branch, is_git_repo, sanitize_branch_name,
};
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

fn temp_git_dir(suffix: &str) -> PathBuf {
    let dir = std::env::temp_dir().join(format!("nxm-git-test-{}-{}", suffix, std::process::id()));
    let _ = fs::remove_dir_all(&dir);
    fs::create_dir_all(&dir).expect("create temp dir");
    dir
}

fn init_git_repo(dir: &Path, default_branch: &str) {
    Command::new("git")
        .args(["-C", dir.to_str().unwrap(), "init"])
        .output()
        .expect("git init");
    Command::new("git")
        .args([
            "-C",
            dir.to_str().unwrap(),
            "config",
            "user.email",
            "test@test.test",
        ])
        .output()
        .expect("git config email");
    Command::new("git")
        .args(["-C", dir.to_str().unwrap(), "config", "user.name", "Test"])
        .output()
        .expect("git config name");
    // Set the default branch before the first commit so it exists.
    Command::new("git")
        .args([
            "-C",
            dir.to_str().unwrap(),
            "symbolic-ref",
            "HEAD",
            &format!("refs/heads/{default_branch}"),
        ])
        .output()
        .expect("git symbolic-ref HEAD");
    Command::new("git")
        .args([
            "-C",
            dir.to_str().unwrap(),
            "commit",
            "--allow-empty",
            "-m",
            "init",
            "--quiet",
        ])
        .output()
        .expect("git initial commit");
}

#[test]
fn sanitize_branch_name_basic() {
    assert_eq!(sanitize_branch_name("Refactoring Q4"), "refactoring_q4");
    assert_eq!(
        sanitize_branch_name("Implement OAuth Login!"),
        "implement_oauth_login"
    );
    assert_eq!(sanitize_branch_name("A   B"), "a_b");
    assert_eq!(sanitize_branch_name("___"), "");
    assert_eq!(sanitize_branch_name("Simple"), "simple");
}

#[test]
fn sanitize_branch_name_length_limit() {
    let long = "a".repeat(200);
    let result = sanitize_branch_name(&long);
    assert!(
        result.len() <= 50,
        "result should be at most 50 chars, got {}: {}",
        result.len(),
        result
    );
}

#[test]
fn is_git_repo_fails_on_non_git_directory() {
    let dir = temp_git_dir("not-git");
    let result = is_git_repo(dir.to_str().unwrap());
    assert!(result.is_err(), "non-git directory should fail");
}

#[test]
fn is_git_repo_succeeds_on_git_directory() {
    let dir = temp_git_dir("is-git");
    init_git_repo(&dir, "develop");
    let result = is_git_repo(dir.to_str().unwrap());
    assert!(result.is_ok(), "git directory should pass");
}

#[test]
fn base_branch_returns_develop_first() {
    let dir = temp_git_dir("base-develop");
    init_git_repo(&dir, "develop");
    let result = base_branch(dir.to_str().unwrap()).expect("base branch should be found");
    assert_eq!(result, "develop");
}

#[test]
fn base_branch_returns_master_when_no_develop() {
    let dir = temp_git_dir("base-master");
    init_git_repo(&dir, "master");
    let result = base_branch(dir.to_str().unwrap()).expect("base branch should be found");
    assert_eq!(result, "master");
}

#[test]
fn base_branch_returns_main_when_no_develop_or_master() {
    let dir = temp_git_dir("base-main");
    init_git_repo(&dir, "main");
    let result = base_branch(dir.to_str().unwrap()).expect("base branch should be found");
    assert_eq!(result, "main");
}

#[test]
fn base_branch_fails_without_develop_master_main() {
    let dir = temp_git_dir("base-custom");
    init_git_repo(&dir, "feature/custom");
    // Only feature/custom exists; develop/master/main should not.
    let result = base_branch(dir.to_str().unwrap());
    assert!(
        result.is_err(),
        "repo without develop/master/main should fail"
    );
}

#[test]
fn create_integration_branch_creates_named_branch() {
    let dir = temp_git_dir("create-branch");
    init_git_repo(&dir, "develop");

    let branch_name = create_integration_branch(dir.to_str().unwrap(), "develop", "Refactoring Q4")
        .expect("branch creation should succeed");

    assert_eq!(branch_name, "integration_refactoring_q4");

    // Verify the branch exists.
    let output = Command::new("git")
        .args([
            "-C",
            dir.to_str().unwrap(),
            "branch",
            "--list",
            "integration_refactoring_q4",
        ])
        .output()
        .expect("git branch list");
    let branches = String::from_utf8_lossy(&output.stdout);
    assert!(
        branches.contains("integration_refactoring_q4"),
        "branch should exist: {branches}"
    );
}

#[test]
fn create_integration_branch_from_master() {
    let dir = temp_git_dir("create-from-master");
    init_git_repo(&dir, "master");

    let branch_name = create_integration_branch(dir.to_str().unwrap(), "master", "Implement OAuth")
        .expect("branch creation should succeed");

    assert_eq!(branch_name, "integration_implement_oauth");

    let output = Command::new("git")
        .args([
            "-C",
            dir.to_str().unwrap(),
            "branch",
            "--list",
            "integration_implement_oauth",
        ])
        .output()
        .expect("git branch list");
    let branches = String::from_utf8_lossy(&output.stdout);
    assert!(
        branches.contains("integration_implement_oauth"),
        "branch should exist: {branches}"
    );
}
