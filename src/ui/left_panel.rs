use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph, Scrollbar, ScrollbarOrientation, ScrollbarState},
    Frame,
};

use crate::analysis::Analysis;
use crate::config::Thresholds;

use super::graph;

pub struct LeftPanel {
    pub graph_scroll: usize,
    pub metrics_scroll: usize,
}

impl Default for LeftPanel {
    fn default() -> Self {
        Self { graph_scroll: 0, metrics_scroll: 0 }
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

    pub fn render(&mut self, frame: &mut Frame, area: Rect, analysis: &Analysis, thresholds: &Thresholds, focused: bool) {
        let border_style = if focused {
            Style::default().fg(Color::Cyan)
        } else {
            Style::default().fg(Color::DarkGray)
        };

        // Split vertically: call graph (top) + metrics table (bottom).
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([Constraint::Percentage(45), Constraint::Percentage(55)])
            .split(area);

        self.render_graph(frame, chunks[0], analysis, border_style);
        self.render_metrics(frame, chunks[1], analysis, thresholds, border_style);
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
        let metrics = &analysis.metrics;

        let header = Line::from(vec![
            Span::styled(
                format!("{:<22} {:<20} {:>5} {:>5} {:>5}  ",
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

        for m in metrics {
            let warn = crate::analysis::has_warning(m, thresholds);
            let row_style = if warn {
                Style::default().fg(Color::Yellow)
            } else {
                Style::default().fg(Color::Gray)
            };

            let file_col = truncate_col(&m.file, 22);
            let fn_col = truncate_col(&m.name, 20);

            let cycl_style = if crate::analysis::is_warning(m.cyclomatic, thresholds.cyclomatic) {
                Style::default().fg(Color::Red).add_modifier(Modifier::BOLD)
            } else {
                row_style
            };
            let cogn_style = if crate::analysis::is_warning(m.cognitive, thresholds.cognitive) {
                Style::default().fg(Color::Red).add_modifier(Modifier::BOLD)
            } else {
                row_style
            };
            let cpl_style = if crate::analysis::is_warning(m.coupling, thresholds.coupling) {
                Style::default().fg(Color::Red).add_modifier(Modifier::BOLD)
            } else {
                row_style
            };

            let warn_indicator = if warn { " ⚠" } else { "  " };

            lines.push(Line::from(vec![
                Span::styled(format!(" {:<22}", file_col), row_style),
                Span::styled(format!("{:<20}", fn_col), row_style),
                Span::styled(format!("{:>5}", m.cyclomatic), cycl_style),
                Span::styled(format!("{:>5}", m.cognitive), cogn_style),
                Span::styled(format!("{:>5}", m.coupling), cpl_style),
                Span::styled(warn_indicator.to_string(), Style::default().fg(Color::Yellow)),
            ]));
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
}

fn truncate_col(s: &str, max: usize) -> String {
    if s.len() <= max {
        s.to_string()
    } else {
        format!("{}…", &s[..max.saturating_sub(1)])
    }
}
