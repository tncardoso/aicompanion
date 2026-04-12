use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph, Scrollbar, ScrollbarOrientation, ScrollbarState},
    Frame,
};

use crate::analysis::Analysis;
use crate::analysis::metrics::FunctionMetricsDelta;
use crate::config::Thresholds;

use super::graph;

/// Sort order for the metrics table.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum MetricSort {
    #[default]
    CyclomaticValue,
    CyclomaticDelta,
    CognitiveValue,
    CognitiveDelta,
    CouplingValue,
    CouplingDelta,
}

impl MetricSort {
    const ALL: [MetricSort; 6] = [
        MetricSort::CyclomaticValue,
        MetricSort::CyclomaticDelta,
        MetricSort::CognitiveValue,
        MetricSort::CognitiveDelta,
        MetricSort::CouplingValue,
        MetricSort::CouplingDelta,
    ];

    pub fn next(self) -> Self {
        let idx = Self::ALL.iter().position(|s| *s == self).unwrap_or(0);
        Self::ALL[(idx + 1) % Self::ALL.len()]
    }

    pub fn prev(self) -> Self {
        let idx = Self::ALL.iter().position(|s| *s == self).unwrap_or(0);
        Self::ALL[(idx + Self::ALL.len() - 1) % Self::ALL.len()]
    }

    pub fn label(self) -> &'static str {
        match self {
            MetricSort::CyclomaticValue => "Cyclomatic Value",
            MetricSort::CyclomaticDelta => "Cyclomatic Delta",
            MetricSort::CognitiveValue  => "Cognitive Value",
            MetricSort::CognitiveDelta  => "Cognitive Delta",
            MetricSort::CouplingValue   => "Coupling Value",
            MetricSort::CouplingDelta   => "Coupling Delta",
        }
    }
}

pub struct LeftPanel {
    pub graph_scroll: usize,
    pub metrics_scroll: usize,
    pub sort: MetricSort,
}

impl Default for LeftPanel {
    fn default() -> Self {
        Self { graph_scroll: 0, metrics_scroll: 0, sort: MetricSort::default() }
    }
}

impl LeftPanel {
    pub fn scroll_graph_down(&mut self, amount: usize) {
        self.graph_scroll = self.graph_scroll.saturating_add(amount);
    }
    pub fn scroll_graph_up(&mut self, amount: usize) {
        self.graph_scroll = self.graph_scroll.saturating_sub(amount);
    }
    pub fn scroll_metrics_down(&mut self, amount: usize) {
        self.metrics_scroll = self.metrics_scroll.saturating_add(amount);
    }
    pub fn scroll_metrics_up(&mut self, amount: usize) {
        self.metrics_scroll = self.metrics_scroll.saturating_sub(amount);
    }

    pub fn render(&mut self, frame: &mut Frame, area: Rect, analysis: &Analysis, thresholds: &Thresholds, graph_focused: bool, metrics_focused: bool) {
        let graph_border = if graph_focused {
            Style::default().fg(Color::Cyan)
        } else {
            Style::default().fg(Color::DarkGray)
        };
        let metrics_border = if metrics_focused {
            Style::default().fg(Color::Cyan)
        } else {
            Style::default().fg(Color::DarkGray)
        };

        // Split vertically: call graph (top) + metrics table (bottom).
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([Constraint::Percentage(45), Constraint::Percentage(55)])
            .split(area);

        self.render_graph(frame, chunks[0], analysis, graph_border);
        self.render_metrics(frame, chunks[1], analysis, thresholds, metrics_border);
    }

    fn render_graph(&mut self, frame: &mut Frame, area: Rect, analysis: &Analysis, border_style: Style) {
        let inner_width = area.width.saturating_sub(2);
        let graph_lines = graph::render(&analysis.call_graph, inner_width);
        let total = graph_lines.len();

        // Clamp scroll.
        let max_scroll = total.saturating_sub(area.height as usize);
        if self.graph_scroll > max_scroll {
            self.graph_scroll = max_scroll;
        }

        let paragraph = Paragraph::new(graph_lines)
            .block(
                Block::default()
                    .title(Span::styled(" Call Graph ", Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD)))
                    .borders(Borders::ALL)
                    .border_style(border_style),
            )
            .scroll((self.graph_scroll as u16, 0));

        frame.render_widget(paragraph, area);

        if total > area.height as usize {
            let mut scrollbar_state = ScrollbarState::new(total).position(self.graph_scroll);
            frame.render_stateful_widget(
                Scrollbar::new(ScrollbarOrientation::VerticalRight),
                area,
                &mut scrollbar_state,
            );
        }
    }

    fn render_metrics(&mut self, frame: &mut Frame, area: Rect, analysis: &Analysis, thresholds: &Thresholds, border_style: Style) {
        // Sort a local copy of the metrics according to the current sort order.
        let mut metrics = analysis.metrics.clone();
        metrics.sort_by(|a, b| {
            let cmp = match self.sort {
                MetricSort::CyclomaticValue => b.cyclomatic.cmp(&a.cyclomatic),
                MetricSort::CyclomaticDelta => cmp_delta(a.cyclomatic_delta, a.cyclomatic, b.cyclomatic_delta, b.cyclomatic),
                MetricSort::CognitiveValue  => b.cognitive.cmp(&a.cognitive),
                MetricSort::CognitiveDelta  => cmp_delta(a.cognitive_delta, a.cognitive, b.cognitive_delta, b.cognitive),
                MetricSort::CouplingValue   => b.coupling.cmp(&a.coupling),
                MetricSort::CouplingDelta   => cmp_delta(a.coupling_delta, a.coupling, b.coupling_delta, b.coupling),
            };
            cmp
        });

        let header = Line::from(vec![
            Span::styled(
                format!(" {:<22}{:<20}{:>10}{:>10}{:>10}  ",
                    "File", "Function", "Cycl", "Cogn", "Cpl"),
                Style::default().fg(Color::White).add_modifier(Modifier::BOLD | Modifier::UNDERLINED),
            ),
        ]);

        let sep = Line::from(Span::styled(
            "─".repeat(area.width.saturating_sub(2) as usize),
            Style::default().fg(Color::DarkGray),
        ));

        let mut lines: Vec<Line<'static>> = vec![header, sep];

        if metrics.is_empty() {
            lines.push(Line::from(Span::styled(
                " (no changed functions detected)",
                Style::default().fg(Color::DarkGray),
            )));
        }

        for m in &metrics {
            lines.push(self.metric_row(m, thresholds));
        }

        let total = lines.len();
        let max_scroll = total.saturating_sub(area.height as usize);
        if self.metrics_scroll > max_scroll {
            self.metrics_scroll = max_scroll;
        }

        let paragraph = Paragraph::new(lines)
            .block(
                Block::default()
                    .title(Span::styled(" Metrics ", Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD)))
                    .borders(Borders::ALL)
                    .border_style(border_style),
            )
            .scroll((self.metrics_scroll as u16, 0));

        frame.render_widget(paragraph, area);
    }

    fn metric_row<'a>(&self, m: &FunctionMetricsDelta, thresholds: &Thresholds) -> Line<'a> {
        let warn = crate::analysis::has_warning(m, thresholds);
        let row_style = if warn {
            Style::default().fg(Color::Yellow)
        } else {
            Style::default().fg(Color::Gray)
        };

        let file_col = truncate_left(&m.file, 21);
        let fn_col = truncate_col(&m.name, 20);

        let cycl_val_style = if crate::analysis::is_warning(m.cyclomatic, thresholds.cyclomatic) {
            Style::default().fg(Color::Red).add_modifier(Modifier::BOLD)
        } else {
            row_style
        };
        let cogn_val_style = if crate::analysis::is_warning(m.cognitive, thresholds.cognitive) {
            Style::default().fg(Color::Red).add_modifier(Modifier::BOLD)
        } else {
            row_style
        };
        let cpl_val_style = if crate::analysis::is_warning(m.coupling, thresholds.coupling) {
            Style::default().fg(Color::Red).add_modifier(Modifier::BOLD)
        } else {
            row_style
        };

        let warn_indicator = if warn { " ⚠" } else { "  " };

        let mut spans: Vec<Span<'a>> = vec![
            Span::styled(format!(" {:<22}", file_col), row_style),
            Span::styled(format!("{:<20}", fn_col), row_style),
        ];

        metric_cell(&mut spans, m.cyclomatic, m.cyclomatic_delta, cycl_val_style, row_style);
        metric_cell(&mut spans, m.cognitive, m.cognitive_delta, cogn_val_style, row_style);
        metric_cell(&mut spans, m.coupling, m.coupling_delta, cpl_val_style, row_style);

        spans.push(Span::styled(warn_indicator.to_string(), Style::default().fg(Color::Yellow)));

        Line::from(spans)
    }
}

/// Append spans for one metric cell (10 chars wide) with an optional delta.
fn metric_cell<'a>(
    spans: &mut Vec<Span<'a>>,
    value: u32,
    delta: Option<i64>,
    val_style: Style,
    neutral_style: Style,
) {
    const WIDTH: usize = 10;
    match delta {
        Some(d) => {
            let val_str = format!("{}", value);
            let delta_str = format!(" ({:+})", d);
            let total = val_str.len() + delta_str.len();
            let pad = WIDTH.saturating_sub(total);
            let padded_val = format!("{}{}", " ".repeat(pad), val_str);

            let delta_style = if d > 0 {
                Style::default().fg(Color::Red)
            } else if d < 0 {
                Style::default().fg(Color::Green)
            } else {
                neutral_style
            };

            spans.push(Span::styled(padded_val, val_style));
            spans.push(Span::styled(delta_str, delta_style));
        }
        None => {
            spans.push(Span::styled(format!("{:>WIDTH$}", value), val_style));
        }
    }
}

/// Compare two entries by delta DESC, falling back to the current value when delta is `None`.
fn cmp_delta(a_delta: Option<i64>, a_val: u32, b_delta: Option<i64>, b_val: u32) -> std::cmp::Ordering {
    let a = a_delta.unwrap_or(a_val as i64);
    let b = b_delta.unwrap_or(b_val as i64);
    b.cmp(&a)
}

/// Truncate from the right, keeping the start: `"longname…"`
fn truncate_col(s: &str, max: usize) -> String {
    let chars: Vec<char> = s.chars().collect();
    if chars.len() <= max {
        s.to_string()
    } else {
        format!("{}…", chars[..max.saturating_sub(1)].iter().collect::<String>())
    }
}

/// Truncate from the left, keeping the suffix: `"…ng/file.rs"`
fn truncate_left(s: &str, max: usize) -> String {
    let chars: Vec<char> = s.chars().collect();
    if chars.len() <= max {
        s.to_string()
    } else {
        let keep = max.saturating_sub(1);
        format!("…{}", chars[chars.len() - keep..].iter().collect::<String>())
    }
}
