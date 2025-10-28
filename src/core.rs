//! Core git repository operations
//!
//! This module provides the core functionality for discovering and checking
//! git repositories, including parallel processing and validation.

use crate::error::{Error, Result};
use rayon::prelude::*;
use serde::Serialize;
use std::path::{Path, PathBuf};
use std::process::Command;
use walkdir::WalkDir;

/// Maximum allowed depth for repository scanning
const MAX_DEPTH_LIMIT: usize = 100;

/// Represents the status of a git repository
#[derive(Debug, Clone, Serialize)]
#[serde(tag = "status", rename_all = "lowercase")]
pub enum RepoStatus {
    Clean {
        path: PathBuf,
        #[serde(skip_serializing_if = "Option::is_none")]
        branch: Option<String>,
    },
    Dirty {
        path: PathBuf,
        changes: String,
        #[serde(skip_serializing_if = "Option::is_none")]
        branch: Option<String>,
    },
}

impl RepoStatus {
    pub fn path(&self) -> &Path {
        match self {
            RepoStatus::Clean { path, .. } => path,
            RepoStatus::Dirty { path, .. } => path,
        }
    }

    pub fn is_dirty(&self) -> bool {
        matches!(self, RepoStatus::Dirty { .. })
    }
}

/// Validates and sanitizes a file system path
///
/// Ensures the path exists, is a directory, and returns the canonical path.
///
/// # Arguments
///
/// * `path` - The path to validate
///
/// # Returns
///
/// The canonicalized path if valid
///
/// # Errors
///
/// Returns `Error::InvalidPath` if the path doesn't exist or isn't a directory
pub fn validate_path(path: &Path) -> Result<PathBuf> {
    let canonical = path
        .canonicalize()
        .map_err(|_| Error::InvalidPath(path.to_path_buf()))?;

    if !canonical.is_dir() {
        return Err(Error::InvalidPath(canonical));
    }

    Ok(canonical)
}

/// Validates the maximum depth parameter
///
/// Ensures the depth is within acceptable bounds (1-100).
///
/// # Arguments
///
/// * `depth` - The maximum depth to validate
///
/// # Returns
///
/// The validated depth value
///
/// # Errors
///
/// Returns `Error::InvalidDepth` if depth is less than 1 or greater than 100
pub fn validate_depth(depth: usize) -> Result<usize> {
    if !(1..=MAX_DEPTH_LIMIT).contains(&depth) {
        return Err(Error::InvalidDepth(depth));
    }
    Ok(depth)
}

/// Finds all git repositories within the given root directory
///
/// Recursively searches for `.git` directories up to the specified depth
/// and returns the parent directories (the repository roots).
///
/// # Arguments
///
/// * `root` - The root directory to start searching from
/// * `max_depth` - Maximum depth to traverse (relative to root)
///
/// # Returns
///
/// A vector of paths to git repository roots
pub fn find_git_repos(root: &Path, max_depth: usize) -> Vec<PathBuf> {
    WalkDir::new(root)
        .max_depth(max_depth)
        .into_iter()
        .filter_map(|e| e.ok())
        .filter(|e| e.file_type().is_dir() && e.file_name() == ".git")
        .filter_map(|e| e.path().parent().map(|p| p.to_path_buf()))
        .collect()
}

/// Gets the current branch name for a repository
fn get_branch_name(repo_path: &Path) -> Option<String> {
    let output = Command::new("git")
        .arg("-C")
        .arg(repo_path)
        .arg("rev-parse")
        .arg("--abbrev-ref")
        .arg("HEAD")
        .output()
        .ok()?;

    if output.status.success() {
        String::from_utf8(output.stdout)
            .ok()
            .map(|s| s.trim().to_string())
    } else {
        None
    }
}

/// Checks the status of a single git repository
///
/// Executes `git status --porcelain` to determine if the repository has
/// uncommitted changes, and optionally retrieves the current branch name.
///
/// # Arguments
///
/// * `repo_path` - Path to the git repository
/// * `include_branch` - Whether to include branch name in the result
///
/// # Returns
///
/// A `RepoStatus` indicating whether the repository is clean or dirty
///
/// # Errors
///
/// Returns `Error::GitCommandFailed` if git command execution fails
pub fn check_repo_status(repo_path: &Path, include_branch: bool) -> Result<RepoStatus> {
    let output = Command::new("git")
        .arg("-C")
        .arg(repo_path)
        .arg("status")
        .arg("--porcelain")
        .output()
        .map_err(|e| Error::GitCommandFailed {
            repo: repo_path.to_path_buf(),
            message: e.to_string(),
        })?;

    if !output.status.success() {
        return Err(Error::GitCommandFailed {
            repo: repo_path.to_path_buf(),
            message: String::from_utf8_lossy(&output.stderr).to_string(),
        });
    }

    let status_output = String::from_utf8_lossy(&output.stdout);
    let branch = if include_branch {
        get_branch_name(repo_path)
    } else {
        None
    };

    if status_output.trim().is_empty() {
        Ok(RepoStatus::Clean {
            path: repo_path.to_path_buf(),
            branch,
        })
    } else {
        Ok(RepoStatus::Dirty {
            path: repo_path.to_path_buf(),
            changes: status_output.into_owned(),
            branch,
        })
    }
}

/// Checks multiple repositories in parallel using rayon
///
/// Leverages parallel processing to check repository status concurrently,
/// improving performance on systems with multiple cores.
///
/// # Arguments
///
/// * `repos` - Slice of repository paths to check
/// * `include_branch` - Whether to include branch names in results
///
/// # Returns
///
/// A tuple containing:
/// - A vector of `RepoStatus` for successfully checked repositories
/// - A vector of `Error` for failed repository checks
pub fn check_repos_parallel(
    repos: &[PathBuf],
    include_branch: bool,
) -> (Vec<RepoStatus>, Vec<Error>) {
    let results: Vec<Result<RepoStatus>> = repos
        .par_iter()
        .map(|repo| check_repo_status(repo, include_branch))
        .collect();

    let mut statuses = Vec::new();
    let mut errors = Vec::new();

    for result in results {
        match result {
            Ok(status) => statuses.push(status),
            Err(e) => errors.push(e),
        }
    }

    (statuses, errors)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;

    #[test]
    fn test_validate_depth() {
        assert!(validate_depth(1).is_ok());
        assert!(validate_depth(50).is_ok());
        assert!(validate_depth(100).is_ok());
        assert!(validate_depth(0).is_err());
        assert!(validate_depth(101).is_err());
    }

    #[test]
    fn test_repo_status_methods() {
        let clean = RepoStatus::Clean {
            path: PathBuf::from("/test"),
            branch: Some("main".to_string()),
        };
        assert!(!clean.is_dirty());

        let dirty = RepoStatus::Dirty {
            path: PathBuf::from("/test"),
            changes: "M file.txt".to_string(),
            branch: Some("dev".to_string()),
        };
        assert!(dirty.is_dirty());
    }

    #[test]
    fn test_validate_path_nonexistent() {
        let result = validate_path(Path::new("/nonexistent/path/that/does/not/exist"));
        assert!(result.is_err());
    }

    #[test]
    fn test_find_git_repos_no_repos() {
        // Create temp directory without git repos
        let temp_dir = std::env::temp_dir().join("test_no_repos");
        let _ = fs::create_dir_all(&temp_dir);

        let repos = find_git_repos(&temp_dir, 3);
        assert_eq!(repos.len(), 0);

        let _ = fs::remove_dir_all(&temp_dir);
    }

    #[test]
    fn test_check_repo_status_invalid_path() {
        let result = check_repo_status(Path::new("/invalid/path"), false);
        assert!(result.is_err());
    }
}
