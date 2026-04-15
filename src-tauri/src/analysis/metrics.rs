use serde::{Deserialize, Serialize};
use streaming_iterator::StreamingIterator;
use tree_sitter::{Language, Node, Parser, Query, QueryCursor};

use super::parser::{call_query, function_def_query};

/// Complexity metrics for a single function.
#[derive(Debug, Clone)]
pub struct FunctionMetrics {
    pub file: String,
    pub name: String,
    pub line: u32,
    pub cyclomatic: u32,
    pub cognitive: u32,
    pub coupling: u32,
}

/// Metrics for a function with optional deltas vs. the HEAD (before) version.
/// `*_delta` is `None` when the function is new (no baseline in HEAD).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FunctionMetricsDelta {
    pub file: String,
    pub name: String,
    pub line: u32,
    pub cyclomatic: u32,
    pub cognitive: u32,
    pub coupling: u32,
    pub cyclomatic_delta: Option<i64>,
    pub cognitive_delta: Option<i64>,
    pub coupling_delta: Option<i64>,
}

/// Join `after` metrics with `before` metrics (keyed on `(file, name)`) to produce deltas.
pub fn compute_deltas(
    before: &[FunctionMetrics],
    after: &[FunctionMetrics],
) -> Vec<FunctionMetricsDelta> {
    // Build lookup: (file, name) → before metrics
    let lookup: std::collections::HashMap<(&str, &str), &FunctionMetrics> = before
        .iter()
        .map(|m| ((m.file.as_str(), m.name.as_str()), m))
        .collect();

    after
        .iter()
        .map(|m| {
            let (cyclomatic_delta, cognitive_delta, coupling_delta) =
                if let Some(b) = lookup.get(&(m.file.as_str(), m.name.as_str())) {
                    (
                        Some(m.cyclomatic as i64 - b.cyclomatic as i64),
                        Some(m.cognitive as i64 - b.cognitive as i64),
                        Some(m.coupling as i64 - b.coupling as i64),
                    )
                } else {
                    (None, None, None)
                };
            FunctionMetricsDelta {
                file: m.file.clone(),
                name: m.name.clone(),
                line: m.line,
                cyclomatic: m.cyclomatic,
                cognitive: m.cognitive,
                coupling: m.coupling,
                cyclomatic_delta,
                cognitive_delta,
                coupling_delta,
            }
        })
        .collect()
}

/// Analyse all functions in `source` (a complete file) and return per-function metrics.
pub fn analyse_file(file: &str, source: &str, ext: &str, language: &Language) -> Vec<FunctionMetrics> {
    let mut parser = Parser::new();
    if parser.set_language(language).is_err() {
        return Vec::new();
    }
    let tree = match parser.parse(source.as_bytes(), None) {
        Some(t) => t,
        None => return Vec::new(),
    };
    let root = tree.root_node();

    let fn_query_src = match function_def_query(ext) {
        Some(q) => q,
        None => return Vec::new(),
    };
    let fn_query = match Query::new(language, fn_query_src) {
        Ok(q) => q,
        Err(_) => return Vec::new(),
    };

    let source_bytes = source.as_bytes();
    let mut cursor = QueryCursor::new();
    let name_capture_idx = fn_query.capture_index_for_name("name").unwrap_or(0);

    // Collect (fn_name, fn_node_id) pairs first so we don't hold a borrow.
    let mut fn_infos: Vec<(String, usize)> = Vec::new(); // (name, node_id)
    {
        let mut matches = cursor.matches(&fn_query, root, source_bytes);
        while let Some(m) = matches.next() {
            let name_cap = m.captures.iter().find(|c| c.index == name_capture_idx);
            if let Some(cap) = name_cap {
                if let Ok(name) = cap.node.utf8_text(source_bytes) {
                    let parent_id = cap.node.parent().map(|n| n.id()).unwrap_or(0);
                    fn_infos.push((name.to_string(), parent_id));
                }
            }
        }
    }

    // We need to re-query to get stable node references for each function body.
    // Use a fresh cursor per function to avoid lifetime issues with the streaming iterator.
    let mut results = Vec::new();
    for (fn_name, _) in &fn_infos {
        let mut cursor2 = QueryCursor::new();
        let mut matches2 = cursor2.matches(&fn_query, root, source_bytes);
        while let Some(m) = matches2.next() {
            let name_cap = m.captures.iter().find(|c| c.index == name_capture_idx);
            if let Some(cap) = name_cap {
                let name_text = cap.node.utf8_text(source_bytes).unwrap_or("");
                if name_text != fn_name {
                    continue;
                }
                let fn_node = cap.node.parent().unwrap_or(root);
                let line = fn_node.start_position().row as u32 + 1;
                let cyclomatic = compute_cyclomatic(fn_node, source_bytes, ext);
                let cognitive = compute_cognitive(fn_node, 0, source_bytes, ext);
                let coupling = compute_coupling(fn_node, source_bytes, ext, language);
                results.push(FunctionMetrics {
                    file: file.to_string(),
                    name: fn_name.clone(),
                    line,
                    cyclomatic,
                    cognitive,
                    coupling,
                });
                break; // found this function, move on
            }
        }
    }
    results
}

/// Cyclomatic complexity: count decision points + 1.
fn compute_cyclomatic(node: Node<'_>, src: &[u8], ext: &str) -> u32 {
    let mut count = 1u32;
    count_decisions(node, src, ext, &mut count);
    count
}

fn count_decisions(node: Node<'_>, src: &[u8], ext: &str, count: &mut u32) {
    let decision_nodes: &[&str] = match ext {
        "rs" => &["if_expression", "while_expression", "for_expression",
                   "loop_expression", "match_arm", "binary_expression"],
        "py" => &["if_statement", "elif_clause", "while_statement",
                   "for_statement", "boolean_operator"],
        "go" => &["if_statement", "for_statement", "switch_statement",
                   "case_clause", "binary_expression"],
        _ => &["if_statement", "while_statement", "for_statement",
               "switch_statement", "case", "conditional_expression",
               "binary_expression"],
    };

    let kind = node.kind();

    if kind == "binary_expression" || kind == "boolean_operator" {
        if let Ok(text) = node.utf8_text(src) {
            if text.contains(" && ") || text.contains(" || ") || text.contains(" and ") || text.contains(" or ") {
                *count += 1;
            }
        }
    } else if decision_nodes.contains(&kind) {
        *count += 1;
    }

    let mut cursor = node.walk();
    if cursor.goto_first_child() {
        loop {
            count_decisions(cursor.node(), src, ext, count);
            if !cursor.goto_next_sibling() { break; }
        }
    }
}

/// Cognitive complexity: nested control structures cost more.
fn compute_cognitive(node: Node<'_>, depth: u32, src: &[u8], ext: &str) -> u32 {
    let nesting_nodes: &[&str] = match ext {
        "rs" => &["if_expression", "while_expression", "for_expression",
                   "loop_expression", "match_expression"],
        "py" => &["if_statement", "while_statement", "for_statement"],
        "go" => &["if_statement", "for_statement", "switch_statement"],
        _ => &["if_statement", "while_statement", "for_statement", "switch_statement"],
    };

    let mut score = 0u32;
    let kind = node.kind();
    let is_nesting = nesting_nodes.contains(&kind);
    if is_nesting {
        score += 1 + depth;
    }

    let child_depth = if is_nesting { depth + 1 } else { depth };
    let mut cursor = node.walk();
    if cursor.goto_first_child() {
        loop {
            score += compute_cognitive(cursor.node(), child_depth, src, ext);
            if !cursor.goto_next_sibling() { break; }
        }
    }
    score
}

/// Coupling: count distinct call targets.
fn compute_coupling(node: Node<'_>, src: &[u8], ext: &str, language: &Language) -> u32 {
    let query_src = match call_query(ext) {
        Some(q) => q,
        None => return 0,
    };
    let query = match Query::new(language, query_src) {
        Ok(q) => q,
        Err(_) => return 0,
    };
    let callee_idx = query.capture_index_for_name("callee").unwrap_or(0);
    let mut cursor = QueryCursor::new();
    let mut callees = std::collections::HashSet::new();
    let mut matches = cursor.matches(&query, node, src);
    while let Some(m) = matches.next() {
        for cap in m.captures {
            if cap.index == callee_idx {
                if let Ok(name) = cap.node.utf8_text(src) {
                    callees.insert(name.to_string());
                }
            }
        }
    }
    callees.len() as u32
}
