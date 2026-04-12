use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, List, ListItem, ListState, Paragraph, Scrollbar, ScrollbarOrientation, ScrollbarState},
    Frame,
};

use crate::git::GitState;

use super::diff_view;

/// An entry in the file list — either a changed file or an untracked file.
#[derive(Debug, Clone)]
pub struct FileEntry {
    pub path: String,
    pub added: usize,
    pub removed: usize,
    pub is_untracked: bool,
}

pub struct RightPanel {
    pub list_state: ListState,
    pub diff_scroll: usize,
    pub diff_hscroll: usize,
    pub entries: Vec<FileEntry>,
}

impl Default for RightPanel {
    fn default() -> Self {
        Self {
            list_state: ListState::default(),
            diff_scroll: 0,
            diff_hscroll: 0,
            entries: Vec::new(),
        }
    }
}

impl RightPanel {
    /// Rebuild the file entry list from the current git state.
    pub fn update_entries(&mut self, state: &GitState) {
        let prev_selected = self.list_state.selected();
        self.entries.clear();

        for diff in &state.diffs {
            self.entries.push(FileEntry {
                path: diff.path.clone(),
                added: diff.added,
                removed: diff.removed,
                is_untracked: false,
            });
        }
        for untracked in &state.untracked {
            self.entries.push(FileEntry {
                path: untracked.path.clone(),
                added: 0,
                removed: 0,
                is_untracked: true,
            });
        }

        // Keep selection within bounds.
        if self.entries.is_empty() {
            self.list_state.select(None);
        } else {
            let sel = prev_selected.unwrap_or(0).min(self.entries.len() - 1);
            self.list_state.select(Some(sel));
        }
    }

    pub fn select_next(&mut self) {
        if self.entries.is_empty() { return; }
        let next = self.list_state.selected()
            .map(|i| (i + 1).min(self.entries.len() - 1))
            .unwrap_or(0);
        self.list_state.select(Some(next));
        self.diff_scroll = 0;
        self.diff_hscroll = 0;
    }

    pub fn select_prev(&mut self) {
        if self.entries.is_empty() { return; }
        let prev = self.list_state.selected()
            .map(|i| i.saturating_sub(1))
            .unwrap_or(0);
        self.list_state.select(Some(prev));
        self.diff_scroll = 0;
        self.diff_hscroll = 0;
    }

    pub fn scroll_diff_down(&mut self, amount: usize) {
        self.diff_scroll = self.diff_scroll.saturating_add(amount);
    }

    pub fn scroll_diff_up(&mut self, amount: usize) {
        self.diff_scroll = self.diff_scroll.saturating_sub(amount);
    }

    pub fn scroll_diff_right(&mut self, amount: usize) {
        self.diff_hscroll = self.diff_hscroll.saturating_add(amount);
    }

    pub fn scroll_diff_left(&mut self, amount: usize) {
        self.diff_hscroll = self.diff_hscroll.saturating_sub(amount);
    }

    pub fn render(&mut self, frame: &mut Frame, area: Rect, git_state: &GitState, list_focused: bool, diff_focused: bool) {
        let list_border_style = if list_focused {
            Style::default().fg(Color::Cyan)
        } else {
            Style::default().fg(Color::DarkGray)
        };
        let diff_border_style = if diff_focused {
            Style::default().fg(Color::Cyan)
        } else {
            Style::default().fg(Color::DarkGray)
        };

        // Split: top 35% file list, bottom 65% diff.
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([Constraint::Percentage(35), Constraint::Percentage(65)])
            .split(area);

        self.render_file_list(frame, chunks[0], list_border_style, list_focused);
        self.render_diff(frame, chunks[1], git_state, diff_border_style);
    }

    fn render_file_list(&mut self, frame: &mut Frame, area: Rect, border_style: Style, focused: bool) {
        let items: Vec<ListItem> = self.entries.iter().map(|entry| {
            let icon = if entry.is_untracked { "?" } else { "M" };
            let stats = if entry.is_untracked {
                " [new]".to_string()
            } else {
                format!(" [+{}/−{}]", entry.added, entry.removed)
            };
            let path_max = area.width.saturating_sub(14) as usize;
            let path_display = if entry.path.len() > path_max {
                format!("…{}", &entry.path[entry.path.len().saturating_sub(path_max - 1)..])
            } else {
                entry.path.clone()
            };
            let style = if entry.is_untracked {
                Style::default().fg(Color::Blue)
            } else {
                Style::default().fg(Color::White)
            };
            let stats_style = Style::default().fg(Color::DarkGray);

            ListItem::new(Line::from(vec![
                Span::styled(format!(" {} ", icon), Style::default().fg(Color::Yellow)),
                Span::styled(path_display, style),
                Span::styled(stats, stats_style),
            ]))
        }).collect();

        let highlight_style = if focused {
            Style::default().fg(Color::Black).bg(Color::Cyan).add_modifier(Modifier::BOLD)
        } else {
            Style::default().fg(Color::Black).bg(Color::DarkGray)
        };

        let list = List::new(items)
            .block(
                Block::default()
                    .title(Span::styled(
                        " Changed Files ",
                        Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD),
                    ))
                    .borders(Borders::ALL)
                    .border_style(border_style),
            )
            .highlight_style(highlight_style)
            .highlight_symbol("▶ ");

        frame.render_stateful_widget(list, area, &mut self.list_state);
    }

    fn render_diff(&mut self, frame: &mut Frame, area: Rect, git_state: &GitState, border_style: Style) {
        let selected = self.list_state.selected();
        let entry = selected.and_then(|i| self.entries.get(i));

        match entry {
            None => {
                let paragraph = Paragraph::new(Span::styled(
                    " Select a file to view its diff",
                    Style::default().fg(Color::DarkGray),
                ))
                .block(
                    Block::default()
                        .title(" Diff ")
                        .borders(Borders::ALL)
                        .border_style(border_style),
                );
                frame.render_widget(paragraph, area);
            }
            Some(entry) => {
                let path = entry.path.clone();
                let is_untracked = entry.is_untracked;

                let (left_lines, right_lines) = if is_untracked {
                    untracked_diff_lines(&path, &git_state.repo_root)
                } else {
                    match git_state.diffs.iter().find(|d| d.path == path) {
                        Some(fd) => diff_view::render(fd),
                        None => (
                            vec![Line::from(Span::styled(" No diff available", Style::default().fg(Color::DarkGray)))],
                            vec![Line::from("")],
                        ),
                    }
                };

                // Render the outer block and obtain the inner area.
                let title = format!(" Diff: {} ", path);
                let outer_block = Block::default()
                    .title(Span::styled(title, Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD)))
                    .borders(Borders::ALL)
                    .border_style(border_style);
                let inner = outer_block.inner(area);
                frame.render_widget(outer_block, area);

                // Split inner area: left pane │ right pane.
                let panes = Layout::default()
                    .direction(Direction::Horizontal)
                    .constraints([Constraint::Fill(1), Constraint::Length(1), Constraint::Fill(1)])
                    .split(inner);
                let left_area  = panes[0];
                let sep_area   = panes[1];
                let right_area = panes[2];

                // Compute scroll bounds.
                let total_lines = left_lines.len().max(right_lines.len());
                let viewport_h  = inner.height as usize;
                let viewport_w  = left_area.width as usize;

                let max_left_w  = line_width_max(&left_lines);
                let max_right_w = line_width_max(&right_lines);
                let max_content_w = max_left_w.max(max_right_w);

                let max_vscroll = total_lines.saturating_sub(viewport_h);
                let max_hscroll = max_content_w.saturating_sub(viewport_w);
                if self.diff_scroll  > max_vscroll { self.diff_scroll  = max_vscroll; }
                if self.diff_hscroll > max_hscroll { self.diff_hscroll = max_hscroll; }

                let scroll = (self.diff_scroll as u16, self.diff_hscroll as u16);

                // Left pane (OLD).
                frame.render_widget(Paragraph::new(left_lines).scroll(scroll), left_area);

                // Separator column.
                let sep_lines: Vec<Line<'static>> = (0..sep_area.height as usize)
                    .map(|_| Line::from(Span::styled("│", Style::default().fg(Color::DarkGray))))
                    .collect();
                frame.render_widget(Paragraph::new(sep_lines), sep_area);

                // Right pane (NEW).
                frame.render_widget(Paragraph::new(right_lines).scroll(scroll), right_area);

                // Vertical scrollbar on the outer right edge.
                if total_lines > viewport_h {
                    let mut state = ScrollbarState::new(total_lines).position(self.diff_scroll);
                    frame.render_stateful_widget(
                        Scrollbar::new(ScrollbarOrientation::VerticalRight), area, &mut state,
                    );
                }
                // Horizontal scrollbar on the outer bottom edge.
                if max_content_w > viewport_w {
                    let mut state = ScrollbarState::new(max_content_w).position(self.diff_hscroll);
                    frame.render_stateful_widget(
                        Scrollbar::new(ScrollbarOrientation::HorizontalBottom), area, &mut state,
                    );
                }
            }
        }
    }
}

/// Build left (OLD) and right (NEW) line vectors for an untracked (all-added) file.
fn untracked_diff_lines(path: &str, repo_root: &std::path::Path) -> (Vec<Line<'static>>, Vec<Line<'static>>) {
    let source = std::fs::read_to_string(repo_root.join(path)).unwrap_or_default();

    let mut left: Vec<Line<'static>>  = Vec::new();
    let mut right: Vec<Line<'static>> = Vec::new();

    left.push(Line::from(Span::styled("  OLD", Style::default().fg(Color::DarkGray).add_modifier(Modifier::BOLD))));
    right.push(Line::from(Span::styled(
        format!("  NEW FILE: {path}"),
        Style::default().fg(Color::Green).add_modifier(Modifier::BOLD),
    )));

    for line in source.lines() {
        left.push(Line::from(""));
        right.push(Line::from(Span::styled(format!("+ {line}"), Style::default().fg(Color::Green))));
    }

    (left, right)
}

/// Return the maximum char-width across all lines.
fn line_width_max(lines: &[Line<'static>]) -> usize {
    lines.iter()
        .map(|l| l.spans.iter().map(|s| s.content.chars().count()).sum::<usize>())
        .max()
        .unwrap_or(0)
}
