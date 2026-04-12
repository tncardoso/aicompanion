mod analysis;
mod app;
mod config;
mod git;
mod ui;

use anyhow::{Context, Result};
use crossterm::{
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::{backend::CrosstermBackend, Terminal};
use std::{
    io,
    time::Duration,
};

use app::App;

fn main() -> Result<()> {
    // Detect repo root from the current directory.
    let cwd = std::env::current_dir().context("cannot determine current directory")?;
    let repo_root = git::find_repo_root(&cwd)
        .context("not inside a git repository — run aicompanion from inside a git repo")?;

    // Load config (uses defaults if .aicompanion.toml not present).
    let config = config::load(&repo_root)?;

    // Initial git state + analysis, scoped to cwd.
    let git_state = git::collect(&repo_root, &cwd)?;
    let analysis = analysis::run(&git_state)?;

    // Start filesystem watcher.
    let (_watcher, watcher_rx) = git::watcher::watch(&repo_root)?;

    // Set up terminal.
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;
    terminal.clear()?;

    let mut app = App::new(git_state, analysis, config);

    // Main loop.
    loop {
        terminal.draw(|frame| {
            app.ui.render(frame, &app.git_state, &app.analysis, &app.config);
        })?;

        if app.should_quit {
            break;
        }

        // Poll for keyboard events (50ms timeout to remain responsive to watcher).
        app.poll_events(Duration::from_millis(50))?;

        if app.should_quit {
            break;
        }

        // Check watcher for file-system changes.
        app.check_watcher(&watcher_rx);
    }

    // Restore terminal.
    disable_raw_mode()?;
    execute!(terminal.backend_mut(), LeaveAlternateScreen)?;
    terminal.show_cursor()?;

    Ok(())
}
