use ratatui::{
    style::{Color, Modifier, Style},
    text::{Line, Span},
};

use crate::git::diff::{DiffLineKind, FileDiff};

/// Build side-by-side diff lines from a FileDiff.
/// Each returned Line has left and right halves separated by a │.
/// `half_width` is the width of each side in characters.
pub fn render(file_diff: &FileDiff, half_width: u16) -> Vec<Line<'static>> {
    let hw = half_width as usize;
    let mut lines: Vec<Line<'static>> = Vec::new();

    // Header
    lines.push(Line::from(vec![
        Span::styled(
            format!(" DIFF: {}", file_diff.path),
            Style::default().fg(Color::White).add_modifier(Modifier::BOLD),
        ),
    ]));
    lines.push(separator(hw));

    // Column headers
    lines.push(Line::from(vec![
        Span::styled(pad("  OLD", hw), Style::default().fg(Color::DarkGray).add_modifier(Modifier::BOLD)),
        Span::styled("│", Style::default().fg(Color::DarkGray)),
        Span::styled(pad("  NEW", hw), Style::default().fg(Color::DarkGray).add_modifier(Modifier::BOLD)),
    ]));
    lines.push(separator(hw));

    for hunk in &file_diff.hunks {
        // Hunk header line
        lines.push(Line::from(Span::styled(
            format!(" {}", truncate(&hunk.header, hw * 2 + 1)),
            Style::default().fg(Color::Blue).add_modifier(Modifier::ITALIC),
        )));

        // Pair up lines: removed on left, added on right, context on both.
        let mut old_buf: Vec<String> = Vec::new();
        let mut new_buf: Vec<String> = Vec::new();

        for dl in &hunk.lines {
            match dl.kind {
                DiffLineKind::Removed => old_buf.push(dl.content.clone()),
                DiffLineKind::Added => new_buf.push(dl.content.clone()),
                DiffLineKind::Context => {
                    // Flush any accumulated old/new pairs first.
                    flush_pairs(&mut lines, &mut old_buf, &mut new_buf, hw);
                    // Context appears on both sides.
                    lines.push(context_line(&dl.content, hw));
                }
            }
        }
        flush_pairs(&mut lines, &mut old_buf, &mut new_buf, hw);
        lines.push(Line::from(""));
    }

    lines
}

/// Flush accumulated removed/added lines as paired side-by-side rows.
fn flush_pairs(lines: &mut Vec<Line<'static>>, old: &mut Vec<String>, new: &mut Vec<String>, hw: usize) {
    let max_rows = old.len().max(new.len());
    for i in 0..max_rows {
        let left = old.get(i).cloned().unwrap_or_default();
        let right = new.get(i).cloned().unwrap_or_default();
        lines.push(diff_line(&left, &right, hw, old.get(i).is_some(), new.get(i).is_some()));
    }
    old.clear();
    new.clear();
}

fn diff_line(left: &str, right: &str, hw: usize, has_left: bool, has_right: bool) -> Line<'static> {
    let left_prefix = if has_left { "- " } else { "  " };
    let right_prefix = if has_right { "+ " } else { "  " };
    let left_content = format!("{}{}", left_prefix, left);
    let right_content = format!("{}{}", right_prefix, right);

    let left_style = if has_left {
        Style::default().fg(Color::Red)
    } else {
        Style::default()
    };
    let right_style = if has_right {
        Style::default().fg(Color::Green)
    } else {
        Style::default()
    };

    Line::from(vec![
        Span::styled(pad(&left_content, hw), left_style),
        Span::styled("│", Style::default().fg(Color::DarkGray)),
        Span::styled(pad(&right_content, hw), right_style),
    ])
}

fn context_line(content: &str, hw: usize) -> Line<'static> {
    let padded = pad(&format!("  {}", content), hw);
    Line::from(vec![
        Span::raw(padded.clone()),
        Span::styled("│", Style::default().fg(Color::DarkGray)),
        Span::raw(padded),
    ])
}

fn separator(hw: usize) -> Line<'static> {
    Line::from(Span::styled(
        format!("{}┼{}", "─".repeat(hw), "─".repeat(hw)),
        Style::default().fg(Color::DarkGray),
    ))
}

/// Pad or truncate `s` to exactly `width` characters.
fn pad(s: &str, width: usize) -> String {
    let chars: Vec<char> = s.chars().collect();
    if chars.len() >= width {
        chars[..width].iter().collect()
    } else {
        let mut result: String = chars.iter().collect();
        for _ in 0..(width - chars.len()) {
            result.push(' ');
        }
        result
    }
}

fn truncate(s: &str, max: usize) -> String {
    let chars: Vec<char> = s.chars().collect();
    if chars.len() <= max {
        s.to_string()
    } else {
        format!("{}…", chars[..max - 1].iter().collect::<String>())
    }
}
