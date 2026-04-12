use ratatui::{
    style::{Color, Modifier, Style},
    text::{Line, Span},
};

use crate::git::diff::{DiffLineKind, FileDiff};

/// Build left (OLD) and right (NEW) line vectors for a two-pane diff.
/// Both vectors always have the same length (shorter side is padded with empty lines).
pub fn render(file_diff: &FileDiff) -> (Vec<Line<'static>>, Vec<Line<'static>>) {
    let mut left: Vec<Line<'static>> = Vec::new();
    let mut right: Vec<Line<'static>> = Vec::new();

    left.push(header_line("  OLD"));
    right.push(header_line("  NEW"));

    for hunk in &file_diff.hunks {
        let hunk_line = hunk_header_line(&hunk.header);
        left.push(hunk_line.clone());
        right.push(hunk_line);

        let mut old_buf: Vec<String> = Vec::new();
        let mut new_buf: Vec<String> = Vec::new();

        for dl in &hunk.lines {
            match dl.kind {
                DiffLineKind::Removed => old_buf.push(dl.content.clone()),
                DiffLineKind::Added   => new_buf.push(dl.content.clone()),
                DiffLineKind::Context => {
                    flush_pairs(&mut left, &mut right, &mut old_buf, &mut new_buf);
                    let ctx = context_line(&dl.content);
                    left.push(ctx.clone());
                    right.push(ctx);
                }
            }
        }
        flush_pairs(&mut left, &mut right, &mut old_buf, &mut new_buf);
        left.push(Line::from(""));
        right.push(Line::from(""));
    }

    (left, right)
}

fn flush_pairs(
    left: &mut Vec<Line<'static>>,
    right: &mut Vec<Line<'static>>,
    old: &mut Vec<String>,
    new: &mut Vec<String>,
) {
    let rows = old.len().max(new.len());
    for i in 0..rows {
        left.push(removed_line(old.get(i).map(String::as_str)));
        right.push(added_line(new.get(i).map(String::as_str)));
    }
    old.clear();
    new.clear();
}

fn removed_line(content: Option<&str>) -> Line<'static> {
    match content {
        Some(s) => Line::from(Span::styled(
            format!("- {s}"),
            Style::default().fg(Color::Red),
        )),
        None => Line::from(""),
    }
}

fn added_line(content: Option<&str>) -> Line<'static> {
    match content {
        Some(s) => Line::from(Span::styled(
            format!("+ {s}"),
            Style::default().fg(Color::Green),
        )),
        None => Line::from(""),
    }
}

fn context_line(content: &str) -> Line<'static> {
    Line::from(format!("  {content}"))
}

fn header_line(label: &'static str) -> Line<'static> {
    Line::from(Span::styled(
        label,
        Style::default().fg(Color::DarkGray).add_modifier(Modifier::BOLD),
    ))
}

fn hunk_header_line(header: &str) -> Line<'static> {
    Line::from(Span::styled(
        format!(" {header}"),
        Style::default().fg(Color::Blue).add_modifier(Modifier::ITALIC),
    ))
}
