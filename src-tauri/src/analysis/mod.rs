pub mod call_graph;
pub mod metrics;
pub mod parser;

use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::collections::HashSet;
use std::path::Path;

use crate::config::Thresholds;
use crate::git::GitState;
use call_graph::CallGraph;
use metrics::{FunctionMetrics, FunctionMetricsDelta};

/// The full analysis result for the current git state.
#[derive(Debug, Default, Clone, Serialize, Deserialize)]
pub struct Analysis {
    pub metrics: Vec<FunctionMetricsDelta>,
    pub call_graph: CallGraph,
}

/// Run analysis on all changed files in `state`.
pub fn run(state: &GitState) -> Result<Analysis> {
    // Collect (path, current_source, head_source_or_none) for all changed files on disk.
    let mut files: Vec<(String, String)> = Vec::new();
    let mut before_metrics: Vec<FunctionMetrics> = Vec::new();

    for file_diff in &state.diffs {
        let full_path = state.repo_root.join(&file_diff.path);
        if full_path.exists() {
            if let Ok(source) = std::fs::read_to_string(&full_path) {
                // Compute "before" metrics from HEAD version, if available.
                if let Some(head_source) =
                    crate::git::file_at_head(&state.repo_root, &file_diff.path)
                {
                    let ext = Path::new(&file_diff.path)
                        .extension()
                        .and_then(|e| e.to_str())
                        .unwrap_or("");
                    if let Some(language) = parser::language_for_extension(ext) {
                        let head_metrics =
                            metrics::analyse_file(&file_diff.path, &head_source, ext, &language);
                        before_metrics.extend(head_metrics);
                    }
                }
                files.push((file_diff.path.clone(), source));
            }
        }
    }

    // Also analyze untracked (new) source files so their metrics are visible.
    // These have no HEAD baseline — deltas will be None.
    let diffed_paths: std::collections::HashSet<&str> =
        state.diffs.iter().map(|d| d.path.as_str()).collect();
    for untracked in &state.untracked {
        if diffed_paths.contains(untracked.path.as_str()) {
            continue; // already covered above
        }
        let ext = std::path::Path::new(&untracked.path)
            .extension()
            .and_then(|e| e.to_str())
            .unwrap_or("");
        if parser::language_for_extension(ext).is_none() {
            continue; // not a supported language
        }
        let full_path = state.repo_root.join(&untracked.path);
        if full_path.exists() {
            if let Ok(source) = std::fs::read_to_string(&full_path) {
                files.push((untracked.path.clone(), source));
            }
        }
    }

    let mut after_metrics: Vec<FunctionMetrics> = Vec::new();
    for (path, source) in &files {
        let ext = Path::new(path)
            .extension()
            .and_then(|e| e.to_str())
            .unwrap_or("");
        if let Some(language) = parser::language_for_extension(ext) {
            let file_metrics = metrics::analyse_file(path, source, ext, &language);
            after_metrics.extend(file_metrics);
        }
    }

    let all_metrics = metrics::compute_deltas(&before_metrics, &after_metrics);

    // Paths of changed/added files (for scoping the call graph).
    let changed_paths: HashSet<String> = state.diffs.iter().map(|d| d.path.clone())
        .chain(state.untracked.iter().map(|u| u.path.clone()))
        .collect();

    // Collect every source file reachable from the repo root so the call graph
    // can show callers and callees from outside the changed set.
    let all_source_files = collect_all_source_files(&state.repo_root);

    let call_graph = call_graph::build(&all_source_files, &changed_paths);

    Ok(Analysis {
        metrics: all_metrics,
        call_graph,
    })
}

/// Walk `repo_root` and return `(relative_path, source)` for every supported
/// source file, skipping common non-source directories.
fn collect_all_source_files(repo_root: &Path) -> Vec<(String, String)> {
    const SKIP: &[&str] = &[
        ".git", "target", "node_modules", ".cargo",
        "dist", "build", "__pycache__", ".tox", "venv", ".venv",
    ];
    let mut files = Vec::new();
    collect_dir(repo_root, repo_root, SKIP, &mut files);
    files
}

fn collect_dir(
    root: &Path,
    dir: &Path,
    skip: &[&str],
    out: &mut Vec<(String, String)>,
) {
    let entries = match std::fs::read_dir(dir) {
        Ok(e) => e,
        Err(_) => return,
    };
    for entry in entries.flatten() {
        let path = entry.path();
        if path.is_dir() {
            let name = path.file_name().and_then(|n| n.to_str()).unwrap_or("");
            if skip.contains(&name) { continue; }
            collect_dir(root, &path, skip, out);
        } else if path.is_file() {
            let ext = path.extension().and_then(|e| e.to_str()).unwrap_or("");
            if parser::language_for_extension(ext).is_none() { continue; }
            if let Ok(source) = std::fs::read_to_string(&path) {
                if let Ok(rel) = path.strip_prefix(root) {
                    out.push((rel.to_string_lossy().replace('\\', "/"), source));
                }
            }
        }
    }
}

/// Check whether a metric value exceeds the configured threshold.
pub fn is_warning(value: u32, threshold: u32) -> bool {
    value > threshold
}

/// Check whether any metric in `m` exceeds its threshold.
pub fn has_warning(m: &FunctionMetricsDelta, thresholds: &Thresholds) -> bool {
    is_warning(m.cyclomatic, thresholds.cyclomatic)
        || is_warning(m.cognitive, thresholds.cognitive)
        || is_warning(m.coupling, thresholds.coupling)
}
