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
    pub entries: Vec<FileEntry>,
}

impl Default for RightPanel {
    fn default() -> Self {
        Self {
            list_state: ListState::default(),
            diff_scroll: 0,
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
    }

    pub fn select_prev(&mut self) {
        if self.entries.is_empty() { return; }
        let prev = self.list_state.selected()
            .map(|i| i.saturating_sub(1))
            .unwrap_or(0);
        self.list_state.select(Some(prev));
        self.diff_scroll = 0;
    }

    pub fn scroll_diff_down(&mut self, amount: usize) {
        self.diff_scroll = self.diff_scroll.saturating_add(amount);
    }

    pub fn scroll_diff_up(&mut self, amount: usize) {
        self.diff_scroll = self.diff_scroll.saturating_sub(amount);
    }

    pub fn render(&mut self, frame: &mut Frame, area: Rect, git_state: &GitState, focused: bool) {
        let border_style = if focused {
            Style::default().fg(Color::Cyan)
        } else {
            Style::default().fg(Color::DarkGray)
        };

        // Split: top 35% file list, bottom 65% diff.
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([Constraint::Percentage(35), Constraint::Percentage(65)])
            .split(area);

        self.render_file_list(frame, chunks[0], border_style, focused);
        self.render_diff(frame, chunks[1], git_state, border_style);
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

                let half_width = area.width.saturating_sub(3) / 2;

                let diff_lines = if is_untracked {
                    render_untracked_diff(&path, &git_state.repo_root, half_width)
                } else {
                    let file_diff = git_state.diffs.iter().find(|d| d.path == path);
                    match file_diff {
                        Some(fd) => diff_view::render(fd, half_width),
                        None => vec![Line::from(Span::styled(
                            " No diff available",
                            Style::default().fg(Color::DarkGray),
                        ))],
                    }
                };

                let total = diff_lines.len();
                let max_scroll = total.saturating_sub(area.height.saturating_sub(2) as usize);
                if self.diff_scroll > max_scroll {
                    self.diff_scroll = max_scroll;
                }

                let title = format!(" Diff: {} ", path);
                let paragraph = Paragraph::new(diff_lines)
                    .block(
                        Block::default()
                            .title(Span::styled(
                                title,
                                Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD),
                            ))
                            .borders(Borders::ALL)
                            .border_style(border_style),
                    )
                    .scroll((self.diff_scroll as u16, 0));

                frame.render_widget(paragraph, area);

                if total > area.height as usize {
                    let mut scrollbar_state = ScrollbarState::new(total).position(self.diff_scroll);
                    frame.render_stateful_widget(
                        Scrollbar::new(ScrollbarOrientation::VerticalRight),
                        area,
                        &mut scrollbar_state,
                    );
                }
            }
        }
    }
}

/// Render an untracked file as a simple "new file" diff (all lines added).
fn render_untracked_diff(path: &str, repo_root: &std::path::Path, half_width: u16) -> Vec<Line<'static>> {
    let full_path = repo_root.join(path);
    let source = std::fs::read_to_string(&full_path).unwrap_or_default();
    let hw = half_width as usize;

    let mut lines = vec![
        Line::from(Span::styled(
            format!(" NEW FILE: {}", path),
            Style::default().fg(Color::Green).add_modifier(Modifier::BOLD),
        )),
        Line::from(Span::styled(
            format!("{}┼{}", "─".repeat(hw), "─".repeat(hw)),
            Style::default().fg(Color::DarkGray),
        )),
    ];

    for source_line in source.lines() {
        let content = format!("+ {}", source_line);
        let padded = pad_str(&content, hw);
        lines.push(Line::from(vec![
            Span::styled(padded.clone(), Style::default().fg(Color::Green)),
            Span::styled("│", Style::default().fg(Color::DarkGray)),
            Span::styled(padded, Style::default().fg(Color::Green)),
        ]));
    }
    lines
}

fn pad_str(s: &str, width: usize) -> String {
    let chars: Vec<char> = s.chars().collect();
    if chars.len() >= width {
        chars[..width].iter().collect()
    } else {
        let mut result: String = chars.iter().collect();
        result.extend(std::iter::repeat(' ').take(width - chars.len()));
        result
    }
}
