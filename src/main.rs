mod cli;
mod core;
mod error;
mod output;

use clap::Parser;
use cli::Args;
use error::Result;
use output::{OutputFormat, Verbosity};

fn main() {
    std::process::exit(match run() {
        Ok(code) => code,
        Err(e) => {
            output::print_error(&e.to_string());
            1
        }
    });
}

fn run() -> Result<i32> {
    let args = Args::parse();

    // Handle shell completion generation
    if args.handle_completion() {
        return Ok(0);
    }

    let verbosity = args.verbosity();
    let output_format = args.output_format();
    let show_branch = args.show_branch;

    // Validate and get configuration
    let root = args.root_path()?;
    let max_depth = args.max_depth();
    let validated_depth = core::validate_depth(max_depth)?;
    let validated_root = core::validate_path(&root)?;

    // Print header in verbose mode
    if verbosity >= Verbosity::Verbose {
        output::print_header(&validated_root, validated_depth);
    }

    // Find repositories
    let repos = core::find_git_repos(&validated_root, validated_depth);

    // Check repositories in parallel
    let (statuses, errors) = core::check_repos_parallel(&repos, show_branch);

    // Report errors if verbosity allows
    if verbosity >= Verbosity::Summary {
        for error in &errors {
            output::print_warning(&error.to_string());
        }
    }

    // Calculate statistics
    let total = statuses.len();
    let dirty_count = statuses.iter().filter(|s| s.is_dirty()).count();

    // Output results based on format
    match output_format {
        OutputFormat::Json => {
            output::print_json(&statuses).map_err(|e| error::Error::Other(e.to_string()))?;
        }
        OutputFormat::Human => {
            // Print detailed status for dirty repos in verbose mode
            if verbosity >= Verbosity::Verbose {
                for status in &statuses {
                    if status.is_dirty() {
                        output::print_verbose_status(status);
                    }
                }
            }

            // Print summary in summary/verbose mode
            if verbosity >= Verbosity::Summary {
                output::print_summary(total, dirty_count);
            }
        }
    }

    // Return exit code (dirty count, capped at 255)
    Ok(if dirty_count > 255 {
        255
    } else {
        dirty_count as i32
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_verbosity_levels() {
        assert_eq!(Verbosity::from_flags(true, false), Verbosity::Quiet);
        assert_eq!(Verbosity::from_flags(false, false), Verbosity::Summary);
        assert_eq!(Verbosity::from_flags(false, true), Verbosity::Verbose);
    }

    #[test]
    fn test_output_format() {
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

        let args_json = Args { json: true, ..args };
        assert_eq!(args_json.output_format(), OutputFormat::Json);
    }
}
