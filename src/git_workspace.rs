//! Git workspace helpers for session integration branches.
//!
//! Provides lightweight operations to verify a working directory is a git
//! repository and to create an integration branch for a work session.
//!
//! All operations shell out to the `git` binary; no `libgit2` binding is used,
//! keeping the dependency footprint minimal.

use std::process::Command;

/// Error returned by git workspace operations.
#[derive(Debug, thiserror::Error)]
pub enum GitWorkspaceError {
    #[error("git command not available: {0}")]
    GitNotAvailable(#[from] std::io::Error),
    #[error("not a git repository: {path}")]
    NotAGitRepository { path: String },
    #[error("no base branch found (tried develop, master, main)")]
    NoBaseBranch,
    #[error("git command failed: {message}")]
    GitCommandFailed { message: String },
}

/// Sanitizes a session title into a short branch name component.
///
/// - Lowercases the string
/// - Replaces non-alphanumeric characters with `_`
/// - Collapses consecutive `_`
/// - Limits to 8 words and 50 characters total
pub fn sanitize_branch_name(title: &str) -> String {
    let sanitized: String = title
        .to_lowercase()
        .chars()
        .map(|c| if c.is_alphanumeric() { c } else { '_' })
        .collect();
    let collapsed: String = sanitized
        .split('_')
        .filter(|s| !s.is_empty())
        .take(8)
        .collect::<Vec<_>>()
        .join("_");
    collapsed.chars().take(50).collect()
}

/// Verifies that `path` is inside a git work tree.
pub fn is_git_repo(path: &str) -> Result<(), GitWorkspaceError> {
    let output = Command::new("git")
        .args(["-C", path, "rev-parse", "--is-inside-work-tree"])
        .output()
        .map_err(GitWorkspaceError::GitNotAvailable)?;
    if !output.status.success() {
        return Err(GitWorkspaceError::NotAGitRepository {
            path: path.to_string(),
        });
    }
    Ok(())
}

/// Returns the first existing base branch among `develop`, `master`, `main`.
pub fn base_branch(path: &str) -> Result<String, GitWorkspaceError> {
    for branch in ["develop", "master", "main"] {
        let output = Command::new("git")
            .args(["-C", path, "rev-parse", "--verify", branch])
            .output()
            .map_err(GitWorkspaceError::GitNotAvailable)?;
        if output.status.success() {
            return Ok(branch.to_string());
        }
    }
    Err(GitWorkspaceError::NoBaseBranch)
}

/// Creates `integration_<short>` branch from the base branch.
///
/// Returns the full branch name that was created.
pub fn create_integration_branch(
    path: &str,
    base: &str,
    session_name: &str,
) -> Result<String, GitWorkspaceError> {
    let short = sanitize_branch_name(session_name);
    let branch_name = format!("integration_{short}");

    // Ensure we are on the base branch.
    let checkout = Command::new("git")
        .args(["-C", path, "checkout", base])
        .output()
        .map_err(GitWorkspaceError::GitNotAvailable)?;
    if !checkout.status.success() {
        return Err(GitWorkspaceError::GitCommandFailed {
            message: String::from_utf8_lossy(&checkout.stderr).to_string(),
        });
    }

    // Create the integration branch.
    let create = Command::new("git")
        .args(["-C", path, "checkout", "-b", &branch_name])
        .output()
        .map_err(GitWorkspaceError::GitNotAvailable)?;
    if !create.status.success() {
        return Err(GitWorkspaceError::GitCommandFailed {
            message: String::from_utf8_lossy(&create.stderr).to_string(),
        });
    }

    Ok(branch_name)
}
