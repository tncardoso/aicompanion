use std::collections::HashMap;

use ratatui::{
    style::{Color, Style},
    text::{Line, Span},
};

use crate::analysis::call_graph::{CallGraph, FnId};

/// Render the call graph as a list of styled lines for display in a panel.
pub fn render(graph: &CallGraph, width: u16) -> Vec<Line<'static>> {
    if graph.nodes.is_empty() {
        return vec![Line::from(Span::styled(
            " (no changed functions detected)",
            Style::default().fg(Color::DarkGray),
        ))];
    }

    // Build reverse map: callee → [callers]
    let mut callers_map: HashMap<&FnId, Vec<&FnId>> = HashMap::new();
    for (caller, callees) in &graph.edges {
        for callee in callees {
            callers_map.entry(callee).or_default().push(caller);
        }
    }
    // Sort callers for deterministic display.
    for callers in callers_map.values_mut() {
        callers.sort_by(|a, b| a.label().cmp(&b.label()));
    }

    let max_label = (width as usize).saturating_sub(4).max(8);
    let mut lines: Vec<Line<'static>> = Vec::new();

    for node in &graph.nodes {
        let label = truncate_label(&node.label(), max_label);
        let callers = callers_map.get(node).map(|v| v.as_slice()).unwrap_or(&[]);
        let callees = graph.edges.get(node).map(|v| v.as_slice()).unwrap_or(&[]);

        // ── Callers (incoming) ──────────────────────────────────────────
        if !callers.is_empty() {
            for (i, caller) in callers.iter().enumerate() {
                let connector = if i == callers.len() - 1 { "  └─▷" } else { "  ├─▷" };
                let clabel = truncate_label(&caller.label(), max_label.saturating_sub(6));
                lines.push(Line::from(vec![
                    Span::styled(connector.to_string(), Style::default().fg(Color::DarkGray)),
                    Span::styled(format!(" {clabel}"), Style::default().fg(Color::Magenta)),
                ]));
            }
            lines.push(Line::from(Span::styled("  │", Style::default().fg(Color::DarkGray))));
        }

        // ── Node box ────────────────────────────────────────────────────
        let box_inner = format!(" {label} ");
        let box_top = format!("┌{}┐", "─".repeat(box_inner.len()));
        let box_mid = format!("│{box_inner}│");
        let box_bot = format!("└{}┘", "─".repeat(box_inner.len()));

        lines.push(Line::from(Span::raw(format!(" {box_top}"))));
        lines.push(Line::from(Span::styled(
            format!(" {box_mid}"),
            Style::default().fg(Color::Cyan),
        )));
        lines.push(Line::from(Span::raw(format!(" {box_bot}"))));

        // ── Callees (outgoing) ──────────────────────────────────────────
        if !callees.is_empty() {
            lines.push(Line::from(Span::styled("  │", Style::default().fg(Color::DarkGray))));
            for (i, callee) in callees.iter().enumerate() {
                let arrow = if i == callees.len() - 1 { "  └─▶" } else { "  ├─▶" };
                let clabel = truncate_label(&callee.label(), max_label.saturating_sub(6));
                lines.push(Line::from(vec![
                    Span::styled(arrow.to_string(), Style::default().fg(Color::Yellow)),
                    Span::styled(format!(" {clabel}"), Style::default().fg(Color::Green)),
                ]));
            }
        }

        lines.push(Line::from(""));
    }

    lines
}

fn truncate_label(label: &str, max: usize) -> String {
    let chars: Vec<char> = label.chars().collect();
    if chars.len() <= max {
        label.to_string()
    } else {
        format!("{}…", chars[..max.saturating_sub(1)].iter().collect::<String>())
    }
}

/// Shorten a FnId label to fit within `max_width` characters.
#[allow(dead_code)]
pub fn short_label(id: &FnId, max_width: usize) -> String {
    truncate_label(&id.label(), max_width)
}
