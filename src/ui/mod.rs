pub mod diff_view;
pub mod graph;
pub mod left_panel;
pub mod right_panel;

use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::Paragraph,
    Frame,
};

use crate::analysis::Analysis;
use crate::config::Config;
use crate::git::GitState;

use left_panel::LeftPanel;
use right_panel::RightPanel;

/// Which panel currently has keyboard focus.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Focus {
    Left,       // call graph
    Metrics,    // metrics table
    Right,      // file list
    RightDiff,  // diff pane
}

pub struct Ui {
    pub left: LeftPanel,
    pub right: RightPanel,
    pub focus: Focus,
}

impl Default for Ui {
    fn default() -> Self {
        Self {
            left: LeftPanel::default(),
            right: RightPanel::default(),
            focus: Focus::Right,
        }
    }
}

impl Ui {
    pub fn toggle_focus(&mut self) {
        self.focus = match self.focus {
            Focus::Left => Focus::Metrics,
            Focus::Metrics => Focus::Right,
            Focus::Right => Focus::RightDiff,
            Focus::RightDiff => Focus::Left,
        };
    }

    pub fn toggle_focus_back(&mut self) {
        self.focus = match self.focus {
            Focus::Left => Focus::RightDiff,
            Focus::Metrics => Focus::Left,
            Focus::Right => Focus::Metrics,
            Focus::RightDiff => Focus::Right,
        };
    }

    pub fn update(&mut self, git_state: &GitState) {
        self.right.update_entries(git_state);
    }

    pub fn render(&mut self, frame: &mut Frame, git_state: &GitState, analysis: &Analysis, config: &Config) {
        let area = frame.area();

        // Status bar at the bottom (1 line).
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([Constraint::Min(0), Constraint::Length(1)])
            .split(area);

        let main_area = chunks[0];
        let status_area = chunks[1];

        // Left 40% / Right 60% split.
        let panels = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([Constraint::Percentage(40), Constraint::Percentage(60)])
            .split(main_area);

        self.left.render(
            frame, panels[0], analysis, &config.thresholds,
            self.focus == Focus::Left,
            self.focus == Focus::Metrics,
        );
        self.right.render(
            frame, panels[1], git_state,
            self.focus == Focus::Right,
            self.focus == Focus::RightDiff,
        );
        render_status_bar(frame, status_area, self.focus, &self.left.sort);
    }
}

fn render_status_bar(frame: &mut Frame, area: Rect, focus: Focus, sort: &left_panel::MetricSort) {
    let focus_label = match focus {
        Focus::Left => "LEFT",
        Focus::Metrics => "METRICS",
        Focus::Right => "FILES",
        Focus::RightDiff => "DIFF",
    };
    let nav_hint = match focus {
        Focus::RightDiff => "j/k:scroll  ",
        _ => "j/k:navigate  ",
    };
    let spans = vec![
        Span::styled(" aicompanion ", Style::default().fg(Color::Black).bg(Color::Cyan).add_modifier(Modifier::BOLD)),
        Span::styled(format!("  Focus: {} ", focus_label), Style::default().fg(Color::White)),
        Span::styled("  Tab", Style::default().fg(Color::Yellow)),
        Span::styled(":switch  ", Style::default().fg(Color::DarkGray)),
        Span::styled("j/k", Style::default().fg(Color::Yellow)),
        Span::styled(nav_hint.to_string(), Style::default().fg(Color::DarkGray)),
        Span::styled("q", Style::default().fg(Color::Yellow)),
        Span::styled(":quit  ", Style::default().fg(Color::DarkGray)),
        Span::styled("r", Style::default().fg(Color::Yellow)),
        Span::styled(":reload  ", Style::default().fg(Color::DarkGray)),
        Span::styled("</>", Style::default().fg(Color::Yellow)),
        Span::styled(format!(":sort  Sort: {}", sort.label()), Style::default().fg(Color::DarkGray)),
    ];
    let bar = Paragraph::new(Line::from(spans))
        .style(Style::default().bg(Color::Reset));
    frame.render_widget(bar, area);
}
