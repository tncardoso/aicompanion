pub mod diff;
pub mod watcher;

use anyhow::{Context, Result};
use std::path::{Path, PathBuf};
use std::process::Command;

use diff::FileDiff;

/// Represents a file that is untracked (new, not yet staged).
#[derive(Debug, Clone)]
pub struct UntrackedFile {
    pub path: String,
}

/// Combined snapshot of git state.
#[derive(Debug, Clone)]
pub struct GitState {
    pub repo_root: PathBuf,
    /// The directory the user launched from (may equal repo_root).
    pub start_dir: PathBuf,
    pub diffs: Vec<FileDiff>,
    pub untracked: Vec<UntrackedFile>,
}

/// Detect the root of the git repository by walking up from `start`.
pub fn find_repo_root(start: &Path) -> Result<PathBuf> {
    let output = Command::new("git")
        .args(["-C", &start.to_string_lossy(), "rev-parse", "--show-toplevel"])
        .output()
        .context("failed to run git")?;
    if output.status.success() {
        let path = String::from_utf8_lossy(&output.stdout).trim().to_string();
        Ok(PathBuf::from(path))
    } else {
        anyhow::bail!("not inside a git repository")
    }
}

/// Collect the current git state: changed files (diff HEAD) + untracked files.
/// `start_dir` scopes the diff to files under that directory (pass `repo_root` for full repo).
pub fn collect(repo_root: &Path, start_dir: &Path) -> Result<GitState> {
    let diffs = collect_diffs(repo_root, start_dir)?;
    let untracked = collect_untracked(repo_root, start_dir)?;
    Ok(GitState {
        repo_root: repo_root.to_path_buf(),
        start_dir: start_dir.to_path_buf(),
        diffs,
        untracked,
    })
}

fn collect_diffs(repo_root: &Path, start_dir: &Path) -> Result<Vec<FileDiff>> {
    let mut args = vec![
        "-C".to_string(),
        repo_root.to_string_lossy().into_owned(),
        "diff".to_string(),
        "HEAD".to_string(),
        "--unified=3".to_string(),
    ];
    // Scope to start_dir when it's a subdirectory of the repo.
    if start_dir != repo_root {
        args.push("--".to_string());
        args.push(start_dir.to_string_lossy().into_owned());
    }

    let output = Command::new("git")
        .args(&args)
        .output()
        .context("failed to run git diff")?;

    let text = String::from_utf8_lossy(&output.stdout);
    Ok(diff::parse_unified(&text))
}

fn collect_untracked(repo_root: &Path, start_dir: &Path) -> Result<Vec<UntrackedFile>> {
    let mut args = vec![
        "-C".to_string(),
        repo_root.to_string_lossy().into_owned(),
        "status".to_string(),
        "--short".to_string(),
        "--porcelain".to_string(),
    ];
    if start_dir != repo_root {
        args.push("--".to_string());
        args.push(start_dir.to_string_lossy().into_owned());
    }

    let output = Command::new("git")
        .args(&args)
        .output()
        .context("failed to run git status")?;

    let text = String::from_utf8_lossy(&output.stdout);
    let mut untracked = Vec::new();
    for line in text.lines() {
        if line.starts_with("??") {
            let path = line[3..].trim().to_string();
            untracked.push(UntrackedFile { path });
        }
    }
    Ok(untracked)
}
