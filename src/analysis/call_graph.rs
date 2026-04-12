use std::collections::{HashMap, HashSet, VecDeque};

use streaming_iterator::StreamingIterator;
use tree_sitter::{Parser, Query, QueryCursor};

use super::parser::{call_query, function_def_query, language_for_extension};

/// A unique identifier for a function: (file_path, function_name).
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct FnId {
    pub file: String,
    pub name: String,
}

impl FnId {
    pub fn label(&self) -> String {
        let file_stem = std::path::Path::new(&self.file)
            .file_stem()
            .map(|s| s.to_string_lossy().into_owned())
            .unwrap_or_else(|| self.file.clone());
        format!("{}::{}", file_stem, self.name)
    }
}

/// Directed call graph restricted to functions connected to the changed set.
#[derive(Debug, Default, Clone)]
pub struct CallGraph {
    /// Adjacency list: caller → callees.
    pub edges: HashMap<FnId, Vec<FnId>>,
    /// All nodes (topologically sorted for display).
    pub nodes: Vec<FnId>,
}

/// Build a call graph from all source files in the repo.
///
/// `all_files` — every source file reachable from the repo root.
/// `changed_paths` — relative paths of files that were modified or added.
///
/// Edges are only included when at least one end belongs to the changed set,
/// so the graph stays focused on what actually changed while showing the full
/// context of callers and callees from anywhere in the codebase.
pub fn build(all_files: &[(String, String)], changed_paths: &HashSet<String>) -> CallGraph {
    struct FnEntry {
        id: FnId,
        start_byte: usize,
        end_byte: usize,
        is_changed: bool,
    }

    // ── Pass 1: collect all function definitions ─────────────────────────
    let mut all_entries: Vec<FnEntry> = Vec::new();

    for (path, source) in all_files {
        let is_changed = changed_paths.contains(path.as_str());
        let ext = std::path::Path::new(path)
            .extension()
            .and_then(|e| e.to_str())
            .unwrap_or("")
            .to_string();
        let language = match language_for_extension(&ext) {
            Some(l) => l,
            None => continue,
        };
        let mut parser = Parser::new();
        if parser.set_language(&language).is_err() {
            continue;
        }
        let tree = match parser.parse(source.as_bytes(), None) {
            Some(t) => t,
            None => continue,
        };
        let fn_query_src = match function_def_query(&ext) {
            Some(q) => q,
            None => continue,
        };
        let fn_query = match Query::new(&language, fn_query_src) {
            Ok(q) => q,
            Err(_) => continue,
        };
        let name_idx = fn_query.capture_index_for_name("name").unwrap_or(0);
        let mut cursor = QueryCursor::new();
        let source_bytes = source.as_bytes();
        let mut matches = cursor.matches(&fn_query, tree.root_node(), source_bytes);
        while let Some(m) = matches.next() {
            for cap in m.captures {
                if cap.index == name_idx {
                    if let Ok(name) = cap.node.utf8_text(source_bytes) {
                        let fn_node = cap.node.parent().unwrap_or(tree.root_node());
                        all_entries.push(FnEntry {
                            id: FnId { file: path.clone(), name: name.to_string() },
                            start_byte: fn_node.start_byte(),
                            end_byte: fn_node.end_byte(),
                            is_changed,
                        });
                    }
                }
            }
        }
    }

    // Build lookup structures.
    let all_fn_ids: HashSet<FnId> = all_entries.iter().map(|e| e.id.clone()).collect();
    let changed_fn_ids: HashSet<FnId> = all_entries.iter()
        .filter(|e| e.is_changed)
        .map(|e| e.id.clone())
        .collect();

    // ── Pass 2: scan function bodies for calls ───────────────────────────
    let mut edges: HashMap<FnId, Vec<FnId>> = HashMap::new();

    for entry in &all_entries {
        let path = &entry.id.file;
        let source = match all_files.iter().find(|(p, _)| p == path) {
            Some((_, s)) => s,
            None => continue,
        };
        let ext = std::path::Path::new(path)
            .extension()
            .and_then(|e| e.to_str())
            .unwrap_or("");
        let language = match language_for_extension(ext) {
            Some(l) => l,
            None => continue,
        };
        let call_q_src = match call_query(ext) {
            Some(q) => q,
            None => continue,
        };
        let call_q = match Query::new(&language, call_q_src) {
            Ok(q) => q,
            Err(_) => continue,
        };
        let callee_idx = call_q.capture_index_for_name("callee").unwrap_or(0);
        let source_bytes = source.as_bytes();

        let mut parser = Parser::new();
        if parser.set_language(&language).is_err() {
            continue;
        }
        let tree = match parser.parse(source_bytes, None) {
            Some(t) => t,
            None => continue,
        };
        let fn_node = match tree.root_node().descendant_for_byte_range(entry.start_byte, entry.end_byte) {
            Some(n) => n,
            None => continue,
        };

        let mut call_cursor = QueryCursor::new();
        let mut call_matches = call_cursor.matches(&call_q, fn_node, source_bytes);

        while let Some(cm) = call_matches.next() {
            for cap in cm.captures {
                if cap.index == callee_idx {
                    if let Ok(callee_name) = cap.node.utf8_text(source_bytes) {
                        // Match callee by name against any known function
                        // (prefer same-file match, then first across all files).
                        let callee = find_callee(callee_name, &entry.id, &all_fn_ids);
                        if let Some(callee) = callee {
                            // Only keep edges where at least one end is changed.
                            let relevant = entry.is_changed || changed_fn_ids.contains(&callee);
                            if relevant {
                                let list = edges.entry(entry.id.clone()).or_default();
                                if !list.contains(&callee) {
                                    list.push(callee);
                                }
                            }
                        }
                    }
                }
            }
        }
    }

    // ── Nodes: all changed functions + anything connected to them ─────────
    let mut node_set: HashSet<FnId> = changed_fn_ids;
    for (caller, callees) in &edges {
        node_set.insert(caller.clone());
        for c in callees {
            node_set.insert(c.clone());
        }
    }

    let nodes = topological_sort(&node_set, &edges);
    CallGraph { edges, nodes }
}

/// Find a callee by name: prefer same file, then first match across all files.
fn find_callee<'a>(name: &str, caller: &FnId, all_fns: &'a HashSet<FnId>) -> Option<FnId> {
    // Same-file match first.
    if let Some(f) = all_fns.iter().find(|f| f.name == name && f.file == caller.file && *f != caller) {
        return Some(f.clone());
    }
    // Cross-file match (first found).
    all_fns.iter().find(|f| f.name == name && *f != caller).cloned()
}

/// Topological sort (Kahn's algorithm) for display ordering.
fn topological_sort(nodes: &HashSet<FnId>, edges: &HashMap<FnId, Vec<FnId>>) -> Vec<FnId> {
    let mut in_degree: HashMap<&FnId, usize> = nodes.iter().map(|n| (n, 0)).collect();
    for targets in edges.values() {
        for t in targets {
            if let Some(d) = in_degree.get_mut(t) {
                *d += 1;
            }
        }
    }
    let mut queue: VecDeque<&FnId> = in_degree.iter()
        .filter(|(_, d)| **d == 0)
        .map(|(n, _)| *n)
        .collect();
    let mut queue_vec: Vec<&FnId> = queue.drain(..).collect();
    queue_vec.sort_by(|a, b| a.label().cmp(&b.label()));
    queue = queue_vec.into_iter().collect();

    let mut result = Vec::new();
    while let Some(node) = queue.pop_front() {
        result.push(node.clone());
        if let Some(targets) = edges.get(node) {
            let mut next: Vec<&FnId> = targets.iter()
                .filter_map(|t| {
                    let d = in_degree.get_mut(t)?;
                    *d -= 1;
                    if *d == 0 { Some(t) } else { None }
                })
                .collect();
            next.sort_by(|a, b| a.label().cmp(&b.label()));
            queue.extend(next);
        }
    }
    // Cycles: append remaining sorted.
    let mut remaining: Vec<FnId> = nodes.iter()
        .filter(|n| !result.contains(n))
        .cloned()
        .collect();
    remaining.sort_by(|a, b| a.label().cmp(&b.label()));
    result.extend(remaining);
    result
}
