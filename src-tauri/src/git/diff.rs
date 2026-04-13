use serde::{Deserialize, Serialize};

/// A single line in a diff, tagged by its kind.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum DiffLineKind {
    Added,
    Removed,
    Context,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DiffLine {
    pub kind: DiffLineKind,
    pub content: String,
}

/// A contiguous hunk within a file diff.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Hunk {
    pub header: String,
    pub lines: Vec<DiffLine>,
}

/// The diff for a single file.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FileDiff {
    pub path: String,
    /// Number of lines added.
    pub added: usize,
    /// Number of lines removed.
    pub removed: usize,
    pub hunks: Vec<Hunk>,
}

/// Parse the output of `git diff` (unified format) into structured FileDiffs.
pub fn parse_unified(output: &str) -> Vec<FileDiff> {
    let mut files: Vec<FileDiff> = Vec::new();
    let mut current_file: Option<FileDiff> = None;
    let mut current_hunk: Option<Hunk> = None;

    for line in output.lines() {
        if line.starts_with("diff --git ") {
            // Commit any pending hunk/file.
            flush_hunk(&mut current_file, &mut current_hunk);
            if let Some(f) = current_file.take() {
                files.push(f);
            }
            // Extract path: "diff --git a/foo b/foo" → "foo"
            let path = line
                .split_whitespace()
                .last()
                .unwrap_or("")
                .trim_start_matches("b/")
                .to_string();
            current_file = Some(FileDiff {
                path,
                added: 0,
                removed: 0,
                hunks: Vec::new(),
            });
        } else if line.starts_with("+++ ") || line.starts_with("--- ") || line.starts_with("index ") || line.starts_with("new file") || line.starts_with("deleted file") || line.starts_with("Binary") {
            // Header lines — skip.
        } else if line.starts_with("@@ ") {
            flush_hunk(&mut current_file, &mut current_hunk);
            current_hunk = Some(Hunk {
                header: line.to_string(),
                lines: Vec::new(),
            });
        } else if let Some(hunk) = current_hunk.as_mut() {
            if let Some(rest) = line.strip_prefix('+') {
                hunk.lines.push(DiffLine { kind: DiffLineKind::Added, content: rest.to_string() });
                if let Some(f) = current_file.as_mut() { f.added += 1; }
            } else if let Some(rest) = line.strip_prefix('-') {
                hunk.lines.push(DiffLine { kind: DiffLineKind::Removed, content: rest.to_string() });
                if let Some(f) = current_file.as_mut() { f.removed += 1; }
            } else {
                let content = line.strip_prefix(' ').unwrap_or(line).to_string();
                hunk.lines.push(DiffLine { kind: DiffLineKind::Context, content });
            }
        }
    }
    flush_hunk(&mut current_file, &mut current_hunk);
    if let Some(f) = current_file.take() {
        files.push(f);
    }
    files
}

fn flush_hunk(file: &mut Option<FileDiff>, hunk: &mut Option<Hunk>) {
    if let (Some(f), Some(h)) = (file.as_mut(), hunk.take()) {
        f.hunks.push(h);
    }
}
