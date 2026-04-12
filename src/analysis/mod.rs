pub mod call_graph;
pub mod metrics;
pub mod parser;

use anyhow::Result;
use std::path::Path;

use crate::config::Thresholds;
use crate::git::GitState;
use call_graph::CallGraph;
use metrics::FunctionMetrics;

/// The full analysis result for the current git state.
#[derive(Debug, Default, Clone)]
pub struct Analysis {
    pub metrics: Vec<FunctionMetrics>,
    pub call_graph: CallGraph,
}

/// Run analysis on all changed files in `state`.
pub fn run(state: &GitState) -> Result<Analysis> {
    // Collect (path, source) for all changed files that exist on disk.
    let mut files: Vec<(String, String)> = Vec::new();

    for file_diff in &state.diffs {
        let full_path = state.repo_root.join(&file_diff.path);
        if full_path.exists() {
            if let Ok(source) = std::fs::read_to_string(&full_path) {
                files.push((file_diff.path.clone(), source));
            }
        }
    }

    let mut all_metrics = Vec::new();
    for (path, source) in &files {
        let ext = Path::new(path)
            .extension()
            .and_then(|e| e.to_str())
            .unwrap_or("");
        if let Some(language) = parser::language_for_extension(ext) {
            let file_metrics = metrics::analyse_file(path, source, ext, &language);
            all_metrics.extend(file_metrics);
        }
    }

    let call_graph = call_graph::build(&files);

    Ok(Analysis {
        metrics: all_metrics,
        call_graph,
    })
}

/// Check whether a metric value exceeds the configured threshold.
pub fn is_warning(value: u32, threshold: u32) -> bool {
    value > threshold
}

/// Check whether any metric in `m` exceeds its threshold.
pub fn has_warning(m: &FunctionMetrics, thresholds: &Thresholds) -> bool {
    is_warning(m.cyclomatic, thresholds.cyclomatic)
        || is_warning(m.cognitive, thresholds.cognitive)
        || is_warning(m.coupling, thresholds.coupling)
}
