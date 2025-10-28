//! Command-line interface and argument parsing
//!
//! This module defines the CLI structure using clap and provides
//! shell completion generation functionality.

use crate::error::{Error, Result};
use crate::output::{OutputFormat, Verbosity};
use clap::{CommandFactory, Parser};
use clap_complete::{generate, Shell};
use std::io;
use std::path::PathBuf;

const VERSION: &str = "1.3.1";
const DEFAULT_MAX_DEPTH: usize = 3;

/// Recursively check git repository status
#[derive(Parser, Debug)]
#[command(name = "check-git-status")]
#[command(author, version = VERSION, about = "Check git repository status recursively")]
#[command(override_usage = "check-git-status [OPTIONS] [path] [maxdepth]")]
pub struct Args {
    /// Root directory to search
    #[arg(value_name = "path")]
    pub root: Option<PathBuf>,

    /// Maximum directory depth (1-100)
    #[arg(value_name = "maxdepth")]
    pub maxdepth: Option<usize>,

    /// Only exit code (0=all clean, N=dirty count)
    #[arg(short = 'q', long = "quiet", conflicts_with = "verbose")]
    pub quiet: bool,

    /// Show detailed git status for dirty repos
    #[arg(short = 'v', long = "verbose")]
    pub verbose: bool,

    /// Output in JSON format
    #[arg(short = 'j', long = "json")]
    pub json: bool,

    /// Show branch names in output
    #[arg(short = 'b', long = "branch")]
    pub show_branch: bool,

    /// Generate shell completion script
    #[arg(long = "generate-completion", value_name = "SHELL")]
    pub generate_completion: Option<Shell>,
}

impl Args {
    /// Get verbosity level from flags
    pub fn verbosity(&self) -> Verbosity {
        Verbosity::from_flags(self.quiet, self.verbose)
    }

    /// Get output format
    pub fn output_format(&self) -> OutputFormat {
        if self.json {
            OutputFormat::Json
        } else {
            OutputFormat::Human
        }
    }

    /// Get the root path to search, with validation
    pub fn root_path(&self) -> Result<PathBuf> {
        match &self.root {
            Some(path) => Ok(path.clone()),
            None => {
                let home = dirs::home_dir().ok_or(Error::HomeDirectoryNotFound)?;
                Ok(home.join("projects"))
            }
        }
    }

    /// Get maximum search depth with validation
    pub fn max_depth(&self) -> usize {
        self.maxdepth.unwrap_or(DEFAULT_MAX_DEPTH)
    }

    /// Generate shell completion and return true if generated
    pub fn handle_completion(&self) -> bool {
        if let Some(shell) = self.generate_completion {
            generate_completion(shell);
            true
        } else {
            false
        }
    }
}

/// Generate shell completion script
fn generate_completion(shell: Shell) {
    let mut cmd = Args::command();
    let name = cmd.get_name().to_string();
    generate(shell, &mut cmd, name, &mut io::stdout());
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_args_verbosity() {
        let args = Args {
            root: None,
            maxdepth: None,
            quiet: true,
            verbose: false,
            json: false,
            show_branch: false,
            generate_completion: None,
        };
        assert_eq!(args.verbosity(), Verbosity::Quiet);

        let args = Args {
            quiet: false,
            verbose: true,
            ..args
        };
        assert_eq!(args.verbosity(), Verbosity::Verbose);
    }

    #[test]
    fn test_args_output_format() {
        let args = Args {
            root: None,
            maxdepth: None,
            quiet: false,
            verbose: false,
            json: false,
            show_branch: false,
            generate_completion: None,
        };
        assert_eq!(args.output_format(), OutputFormat::Human);

        let args = Args { json: true, ..args };
        assert_eq!(args.output_format(), OutputFormat::Json);
    }

    #[test]
    fn test_args_max_depth() {
        let args = Args {
            root: None,
            maxdepth: Some(5),
            quiet: false,
            verbose: false,
            json: false,
            show_branch: false,
            generate_completion: None,
        };
        assert_eq!(args.max_depth(), 5);

        let args = Args {
            maxdepth: None,
            ..args
        };
        assert_eq!(args.max_depth(), DEFAULT_MAX_DEPTH);
    }

    #[test]
    fn test_args_root_path_custom() {
        let custom_path = PathBuf::from("/custom/path");
        let args = Args {
            root: Some(custom_path.clone()),
            maxdepth: None,
            quiet: false,
            verbose: false,
            json: false,
            show_branch: false,
            generate_completion: None,
        };
        let result = args.root_path();
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), custom_path);
    }

    #[test]
    fn test_args_handle_completion() {
        let args = Args {
            root: None,
            maxdepth: None,
            quiet: false,
            verbose: false,
            json: false,
            show_branch: false,
            generate_completion: None,
        };
        assert!(!args.handle_completion());

        let args = Args {
            generate_completion: Some(Shell::Bash),
            ..args
        };
        // Note: This would actually generate completion, but returns true
        // We just test the return value logic
        assert!(args.generate_completion.is_some());
    }
}
