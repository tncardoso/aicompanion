use ratatui::{
    style::{Color, Style},
    text::{Line, Span},
};

use crate::analysis::call_graph::{CallGraph, FnId};

/// Render the call graph as a list of styled lines for display in a panel.
/// Uses box-drawing characters and arrows.
pub fn render(graph: &CallGraph, width: u16) -> Vec<Line<'static>> {
    if graph.nodes.is_empty() {
        return vec![Line::from(Span::styled(
            " (no changed functions detected)",
            Style::default().fg(Color::DarkGray),
        ))];
    }

    let mut lines: Vec<Line<'static>> = Vec::new();

    // Render each node as a box, with arrows to its callees below.
    for (i, node) in graph.nodes.iter().enumerate() {
        let label = node.label();
        // Truncate label to available width (leave room for box borders + padding).
        let max_label = (width as usize).saturating_sub(4).max(8);
        let label = if label.len() > max_label {
            format!("{}…", &label[..max_label - 1])
        } else {
            label
        };

        let box_inner = format!(" {} ", label);
        let box_top = format!("┌{}┐", "─".repeat(box_inner.len()));
        let box_mid = format!("│{}│", box_inner);
        let box_bot = format!("└{}┘", "─".repeat(box_inner.len()));

        lines.push(Line::from(Span::raw(format!(" {}", box_top))));
        lines.push(Line::from(Span::styled(
            format!(" {}", box_mid),
            Style::default().fg(Color::Cyan),
        )));
        lines.push(Line::from(Span::raw(format!(" {}", box_bot))));

        // Show edges.
        if let Some(targets) = graph.edges.get(node) {
            if !targets.is_empty() {
                lines.push(Line::from(Span::styled(
                    "  │".to_string(),
                    Style::default().fg(Color::DarkGray),
                )));
                for (ti, target) in targets.iter().enumerate() {
                    let arrow = if ti == targets.len() - 1 { "  └─▶" } else { "  ├─▶" };
                    let tgt_label = target.label();
                    let tgt_label = if tgt_label.len() > max_label.saturating_sub(5) {
                        format!("{}…", &tgt_label[..max_label.saturating_sub(6)])
                    } else {
                        tgt_label
                    };
                    lines.push(Line::from(vec![
                        Span::styled(arrow.to_string(), Style::default().fg(Color::Yellow)),
                        Span::styled(
                            format!(" {}", tgt_label),
                            Style::default().fg(Color::Green),
                        ),
                    ]));
                }
                lines.push(Line::from(""));
            }
        }

        // Spacing between unconnected nodes.
        if i + 1 < graph.nodes.len() {
            if graph.edges.get(node).map(|e| e.is_empty()).unwrap_or(true) {
                lines.push(Line::from(""));
            }
        }
    }
    lines
}

/// Shorten a FnId label to fit within `max_width` characters.
#[allow(dead_code)]
pub fn short_label(id: &FnId, max_width: usize) -> String {
    let label = id.label();
    if label.len() <= max_width {
        label
    } else {
        format!("{}…", &label[..max_width.saturating_sub(1)])
    }
}
