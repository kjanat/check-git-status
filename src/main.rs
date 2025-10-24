use anyhow::{Context, Result};
use clap::Parser;
use rayon::prelude::*;
use std::path::{Path, PathBuf};
use std::process::Command;
use walkdir::WalkDir;

const VERSION: &str = "1.3.1";
const DEFAULT_MAX_DEPTH: usize = 3;

#[derive(Parser, Debug)]
#[command(name = "check-git-status")]
#[command(author, version = VERSION, about = "Check git repository status recursively")]
#[command(override_usage = "check-git-status [OPTIONS] [path] [maxdepth]")]
struct Args {
    /// Root directory to search
    #[arg(value_name = "path")]
    root: Option<PathBuf>,

    /// Maximum directory depth
    #[arg(value_name = "maxdepth")]
    maxdepth: Option<usize>,

    /// Only exit code (0=all clean, N=dirty count)
    #[arg(short = 'q', long = "quiet")]
    quiet: bool,

    /// Show detailed git status for dirty repos
    #[arg(short = 'v', long = "verbose")]
    verbose: bool,
}

#[derive(Debug)]
enum RepoStatus {
    Clean,
    Dirty(PathBuf, String),
}

impl Args {
    fn verbosity(&self) -> u8 {
        if self.quiet {
            0
        } else if self.verbose {
            2
        } else {
            1
        }
    }

    fn root_path(&self) -> Result<PathBuf> {
        match &self.root {
            Some(path) => Ok(path.clone()),
            None => {
                let home = dirs::home_dir().context("Could not determine home directory")?;
                Ok(home.join("projects"))
            }
        }
    }

    fn max_depth(&self) -> usize {
        self.maxdepth.unwrap_or(DEFAULT_MAX_DEPTH)
    }
}

fn find_git_repos(root: &Path, max_depth: usize) -> Vec<PathBuf> {
    WalkDir::new(root)
        .max_depth(max_depth)
        .into_iter()
        .filter_map(|e| e.ok())
        .filter(|e| e.file_type().is_dir() && e.file_name() == ".git")
        .filter_map(|e| e.path().parent().map(|p| p.to_path_buf()))
        .collect()
}

fn check_repo_status(repo_path: &Path) -> Result<RepoStatus> {
    let output = Command::new("git")
        .arg("-C")
        .arg(repo_path)
        .arg("status")
        .arg("--porcelain")
        .output()
        .with_context(|| format!("Failed to execute git status in {}", repo_path.display()))?;

    let status_output = String::from_utf8_lossy(&output.stdout);

    if status_output.trim().is_empty() {
        Ok(RepoStatus::Clean)
    } else {
        Ok(RepoStatus::Dirty(
            repo_path.to_path_buf(),
            status_output.into_owned(),
        ))
    }
}

fn get_repo_name(path: &Path) -> String {
    path.file_name()
        .and_then(|n| n.to_str())
        .unwrap_or("unknown")
        .to_string()
}

fn print_verbose_status(path: &Path, status: &str) {
    let repo_name = get_repo_name(path);
    eprintln!("📦 {}", repo_name);

    for line in status.lines() {
        if !line.trim().is_empty() {
            eprintln!("  {}", line);
        }
    }
    eprintln!();
}

fn print_summary(total: usize, dirty: usize) {
    eprintln!("📦 Total repos: {}", total);
    eprintln!("📊 Dirty repos: {}", dirty);
}

fn print_header(root: &Path, max_depth: usize) {
    eprintln!(
        "🔍 Checking git repos in {} (maxdepth={})",
        root.display(),
        max_depth
    );

    let width = terminal_width();
    eprintln!("{}", "━".repeat(width));
    eprintln!();
}

fn terminal_width() -> usize {
    term_size::dimensions().map(|(w, _)| w).unwrap_or(80)
}

fn main() -> Result<()> {
    let args = Args::parse();
    let verbosity = args.verbosity();
    let root = args.root_path()?;
    let max_depth = args.max_depth();

    if verbosity >= 2 {
        print_header(&root, max_depth);
    }

    let repos = find_git_repos(&root, max_depth);
    let statuses: Vec<RepoStatus> = repos
        .par_iter()
        .filter_map(|repo| {
            check_repo_status(repo)
                .map_err(|e| {
                    eprintln!("Warning: {:#}", e);
                    e
                })
                .ok()
        })
        .collect();

    let total = statuses.len();
    let mut dirty_count = 0;

    for status in &statuses {
        match status {
            RepoStatus::Dirty(path, git_output) => {
                dirty_count += 1;
                if verbosity >= 2 {
                    print_verbose_status(path, git_output);
                }
            }
            RepoStatus::Clean => {}
        }
    }

    if verbosity >= 1 {
        print_summary(total, dirty_count);
    }

    let exit_code = if dirty_count > 255 { 255 } else { dirty_count };

    std::process::exit(exit_code as i32);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_verbosity_levels() {
        let quiet_args = Args {
            root: None,
            maxdepth: None,
            quiet: true,
            verbose: false,
        };
        assert_eq!(quiet_args.verbosity(), 0);

        let normal_args = Args {
            root: None,
            maxdepth: None,
            quiet: false,
            verbose: false,
        };
        assert_eq!(normal_args.verbosity(), 1);

        let verbose_args = Args {
            root: None,
            maxdepth: None,
            quiet: false,
            verbose: true,
        };
        assert_eq!(verbose_args.verbosity(), 2);
    }
}
