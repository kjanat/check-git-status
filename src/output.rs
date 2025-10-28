//! Output formatting and display
//!
//! This module handles all output formatting including human-readable
//! colored terminal output and JSON serialization.

use crate::core::RepoStatus;
use colored::*;
use serde::Serialize;
use std::path::Path;

/// Output format options
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum OutputFormat {
    Human,
    Json,
}

/// Verbosity levels
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum Verbosity {
    Quiet = 0,
    Summary = 1,
    Verbose = 2,
}

impl Verbosity {
    pub fn from_flags(quiet: bool, verbose: bool) -> Self {
        if quiet {
            Verbosity::Quiet
        } else if verbose {
            Verbosity::Verbose
        } else {
            Verbosity::Summary
        }
    }
}

/// JSON output structure
#[derive(Debug, Serialize)]
pub struct JsonOutput {
    pub total: usize,
    pub dirty: usize,
    pub clean: usize,
    pub repositories: Vec<RepoStatus>,
}

/// Gets the repository name from a path
fn get_repo_name(path: &Path) -> String {
    path.file_name()
        .and_then(|n| n.to_str())
        .unwrap_or("unknown")
        .to_string()
}

/// Prints the header for verbose mode
pub fn print_header(root: &Path, max_depth: usize) {
    eprintln!(
        "{} Checking git repos in {} (maxdepth={})",
        "🔍".cyan(),
        root.display().to_string().bright_blue(),
        max_depth.to_string().yellow()
    );

    let width = terminal_width();
    eprintln!("{}", "━".repeat(width).bright_black());
    eprintln!();
}

/// Prints verbose status for a single repository
pub fn print_verbose_status(status: &RepoStatus) {
    let path = status.path();
    let repo_name = get_repo_name(path);

    match status {
        RepoStatus::Clean { branch, .. } => {
            let branch_str = branch
                .as_ref()
                .map(|b| format!(" ({})", b.bright_cyan()))
                .unwrap_or_default();
            eprintln!("{} {}{}", "📦".green(), repo_name.green(), branch_str);
        }
        RepoStatus::Dirty {
            changes, branch, ..
        } => {
            let branch_str = branch
                .as_ref()
                .map(|b| format!(" ({})", b.bright_cyan()))
                .unwrap_or_default();
            eprintln!(
                "{} {}{}",
                "📦".yellow(),
                repo_name.yellow().bold(),
                branch_str
            );

            for line in changes.lines() {
                if !line.trim().is_empty() {
                    eprintln!("  {}", line.bright_white());
                }
            }
            eprintln!();
        }
    }
}

/// Prints summary statistics
pub fn print_summary(total: usize, dirty: usize) {
    let clean = total - dirty;

    eprintln!("{} Total repos: {}", "📦".cyan(), total);
    eprintln!("{} Clean repos: {}", "✓".green(), clean.to_string().green());
    eprintln!(
        "{} Dirty repos: {}",
        "✗".yellow(),
        if dirty > 0 {
            dirty.to_string().yellow().bold()
        } else {
            dirty.to_string().green()
        }
    );
}

/// Outputs results in JSON format
pub fn print_json(statuses: &[RepoStatus]) -> Result<(), serde_json::Error> {
    let total = statuses.len();
    let dirty = statuses.iter().filter(|s| s.is_dirty()).count();
    let clean = total - dirty;

    let output = JsonOutput {
        total,
        dirty,
        clean,
        repositories: statuses.to_vec(),
    };

    println!("{}", serde_json::to_string_pretty(&output)?);
    Ok(())
}

/// Gets terminal width
fn terminal_width() -> usize {
    term_size::dimensions().map(|(w, _)| w).unwrap_or(80)
}

/// Print warning message
pub fn print_warning(message: &str) {
    eprintln!("{} {}", "Warning:".yellow().bold(), message);
}

/// Print error message
pub fn print_error(message: &str) {
    eprintln!("{} {}", "Error:".red().bold(), message);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_output_format_equality() {
        assert_eq!(OutputFormat::Human, OutputFormat::Human);
        assert_eq!(OutputFormat::Json, OutputFormat::Json);
        assert_ne!(OutputFormat::Human, OutputFormat::Json);
    }

    #[test]
    fn test_verbosity_ordering() {
        assert!(Verbosity::Quiet < Verbosity::Summary);
        assert!(Verbosity::Summary < Verbosity::Verbose);
    }

    #[test]
    fn test_verbosity_from_flags() {
        assert_eq!(Verbosity::from_flags(true, false), Verbosity::Quiet);
        assert_eq!(Verbosity::from_flags(false, true), Verbosity::Verbose);
        assert_eq!(Verbosity::from_flags(false, false), Verbosity::Summary);
    }

    #[test]
    fn test_json_output_structure() {
        let statuses = vec![
            RepoStatus::Clean {
                path: std::path::PathBuf::from("/test/clean"),
                branch: Some("main".to_string()),
            },
            RepoStatus::Dirty {
                path: std::path::PathBuf::from("/test/dirty"),
                changes: "M file.txt\n".to_string(),
                branch: Some("dev".to_string()),
            },
        ];

        let result = print_json(&statuses);
        assert!(result.is_ok());
    }

    #[test]
    fn test_json_output_serialization() {
        let output = JsonOutput {
            total: 10,
            dirty: 3,
            clean: 7,
            repositories: vec![],
        };

        let json = serde_json::to_string(&output).unwrap();
        assert!(json.contains("\"total\":10"));
        assert!(json.contains("\"dirty\":3"));
        assert!(json.contains("\"clean\":7"));
    }
}
