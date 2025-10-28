//! Error types and result type alias for check-git-status

use std::fmt;
use std::path::PathBuf;

/// Custom error types for check-git-status
///
/// Provides specific error variants for different failure modes
/// to enable better error handling and user-friendly messages.
#[derive(Debug)]
pub enum Error {
    /// Home directory could not be determined
    HomeDirectoryNotFound,

    /// Invalid path provided
    InvalidPath(PathBuf),

    /// Maximum depth exceeded allowed bounds
    InvalidDepth(usize),

    /// Git command failed
    GitCommandFailed { repo: PathBuf, message: String },

    /// IO error occurred
    Io(std::io::Error),

    /// Other error
    Other(String),
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Error::HomeDirectoryNotFound => {
                write!(f, "Could not determine home directory")
            }
            Error::InvalidPath(path) => {
                write!(f, "Invalid path: {}", path.display())
            }
            Error::InvalidDepth(depth) => {
                write!(f, "Invalid depth: {} (must be between 1 and 100)", depth)
            }
            Error::GitCommandFailed { repo, message } => {
                write!(f, "Git command failed in {}: {}", repo.display(), message)
            }
            Error::Io(e) => {
                write!(f, "IO error: {}", e)
            }
            Error::Other(msg) => {
                write!(f, "{}", msg)
            }
        }
    }
}

impl std::error::Error for Error {}

impl From<std::io::Error> for Error {
    fn from(err: std::io::Error) -> Self {
        Error::Io(err)
    }
}

pub type Result<T> = std::result::Result<T, Error>;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_error_display_home_not_found() {
        let err = Error::HomeDirectoryNotFound;
        assert_eq!(err.to_string(), "Could not determine home directory");
    }

    #[test]
    fn test_error_display_invalid_path() {
        let err = Error::InvalidPath(PathBuf::from("/invalid"));
        assert!(err.to_string().contains("Invalid path"));
        assert!(err.to_string().contains("/invalid"));
    }

    #[test]
    fn test_error_display_invalid_depth() {
        let err = Error::InvalidDepth(150);
        assert!(err.to_string().contains("Invalid depth: 150"));
        assert!(err.to_string().contains("must be between 1 and 100"));
    }

    #[test]
    fn test_error_display_git_failed() {
        let err = Error::GitCommandFailed {
            repo: PathBuf::from("/test/repo"),
            message: "command not found".to_string(),
        };
        let display = err.to_string();
        assert!(display.contains("Git command failed"));
        assert!(display.contains("/test/repo"));
        assert!(display.contains("command not found"));
    }

    #[test]
    fn test_error_from_io_error() {
        let io_err = std::io::Error::new(std::io::ErrorKind::NotFound, "file not found");
        let err: Error = io_err.into();
        match err {
            Error::Io(_) => {}
            _ => panic!("Expected Error::Io variant"),
        }
    }

    #[test]
    fn test_error_other() {
        let err = Error::Other("custom error".to_string());
        assert_eq!(err.to_string(), "custom error");
    }
}
