use std::env;
use std::fs;
use std::process::Command;

/// Helper to run the binary with arguments
fn run_with_args(args: &[&str]) -> std::process::Output {
    let bin_path = env!("CARGO_BIN_EXE_check-git-status");
    Command::new(bin_path)
        .args(args)
        .output()
        .expect("Failed to execute binary")
}

/// Helper to create a temporary git repository
fn create_temp_git_repo(name: &str, dirty: bool) -> tempfile::TempDir {
    let temp = tempfile::tempdir().expect("Failed to create temp dir");
    let repo_path = temp.path().join(name);
    fs::create_dir_all(&repo_path).expect("Failed to create repo dir");

    // Initialize git repo
    Command::new("git")
        .args(["init"])
        .current_dir(&repo_path)
        .output()
        .expect("Failed to init git repo");

    // Configure git
    Command::new("git")
        .args(["config", "user.email", "test@test.com"])
        .current_dir(&repo_path)
        .output()
        .expect("Failed to config git");

    Command::new("git")
        .args(["config", "user.name", "Test User"])
        .current_dir(&repo_path)
        .output()
        .expect("Failed to config git");

    if dirty {
        // Create a file to make repo dirty
        fs::write(repo_path.join("test.txt"), "test content").expect("Failed to write file");
    } else {
        // Create and commit a file to make repo clean
        fs::write(repo_path.join("test.txt"), "test content").expect("Failed to write file");
        Command::new("git")
            .args(["add", "."])
            .current_dir(&repo_path)
            .output()
            .expect("Failed to add file");
        Command::new("git")
            .args(["commit", "-m", "Initial commit"])
            .current_dir(&repo_path)
            .output()
            .expect("Failed to commit");
    }

    temp
}

#[test]
fn test_help_flag() {
    let output = run_with_args(&["--help"]);
    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("Check git repository status recursively"));
    assert!(stdout.contains("Usage:"));
}

#[test]
fn test_version_flag() {
    let output = run_with_args(&["--version"]);
    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("1.3.1"));
}

#[test]
fn test_clean_repo_quiet_mode() {
    let temp = create_temp_git_repo("clean_repo", false);
    let output = run_with_args(&["-q", temp.path().to_str().unwrap(), "2"]);
    // Clean repo should return exit code 0
    assert_eq!(output.status.code(), Some(0));
}

#[test]
fn test_dirty_repo_quiet_mode() {
    let temp = create_temp_git_repo("dirty_repo", true);
    let output = run_with_args(&["-q", temp.path().to_str().unwrap(), "2"]);
    // Dirty repo should return exit code 1
    assert_eq!(output.status.code(), Some(1));
}

#[test]
fn test_json_output() {
    let temp = create_temp_git_repo("test_repo", true);
    let output = run_with_args(&["--json", temp.path().to_str().unwrap(), "2"]);

    // JSON mode exits with dirty count (1 in this case)
    let stdout = String::from_utf8_lossy(&output.stdout);

    // Verify JSON structure
    assert!(stdout.contains("\"total\":"));
    assert!(stdout.contains("\"dirty\":"));
    assert!(stdout.contains("\"clean\":"));
    assert!(stdout.contains("\"repositories\":"));
}

#[test]
fn test_verbose_output() {
    let temp = create_temp_git_repo("verbose_test", true);
    let output = run_with_args(&["-v", temp.path().to_str().unwrap(), "2"]);

    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(stderr.contains("Checking git repos"));
    assert!(stderr.contains("Total repos:"));
    assert!(stderr.contains("Dirty repos:"));
}

#[test]
fn test_branch_flag() {
    let temp = create_temp_git_repo("branch_test", false);
    let output = run_with_args(&["--json", "--branch", temp.path().to_str().unwrap(), "2"]);

    let stdout = String::from_utf8_lossy(&output.stdout);
    // Should include branch information in JSON
    assert!(stdout.contains("\"branch\""));
}

#[test]
fn test_invalid_path() {
    let output = run_with_args(&["/nonexistent/path/that/does/not/exist", "3"]);

    // Should fail with error
    assert!(!output.status.success());
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(stderr.contains("Error:") || stderr.contains("Invalid path"));
}

#[test]
fn test_invalid_depth() {
    let output = run_with_args(&[
        ".", "1000", // Exceeds max depth of 100
    ]);

    // Should fail with error
    assert!(!output.status.success());
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(stderr.contains("Invalid depth") || stderr.contains("Error:"));
}

#[test]
fn test_completion_generation_bash() {
    let output = run_with_args(&["--generate-completion", "bash"]);
    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("_check-git-status"));
    assert!(stdout.contains("COMPREPLY"));
}

#[test]
fn test_completion_generation_zsh() {
    let output = run_with_args(&["--generate-completion", "zsh"]);
    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("#compdef check-git-status"));
}

#[test]
fn test_completion_generation_fish() {
    let output = run_with_args(&["--generate-completion", "fish"]);
    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("complete -c check-git-status"));
}

#[test]
fn test_summary_mode_default() {
    let temp = create_temp_git_repo("summary_test", true);
    let output = run_with_args(&[temp.path().to_str().unwrap(), "2"]);

    let stderr = String::from_utf8_lossy(&output.stderr);
    // Default mode should show summary
    assert!(stderr.contains("Total repos:") || stderr.contains("Dirty repos:"));
}
