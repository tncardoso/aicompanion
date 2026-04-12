use anyhow::Result;
use crossterm::event::{self, Event, KeyCode};
use std::sync::mpsc::Receiver;
use std::time::Duration;

use crate::analysis::Analysis;
use crate::config::Config;
use crate::git::GitState;
use crate::ui::{Focus, Ui};

pub struct App {
    pub ui: Ui,
    pub git_state: GitState,
    pub analysis: Analysis,
    pub config: Config,
    pub should_quit: bool,
    pub status_msg: Option<String>,
}

impl App {
    pub fn new(git_state: GitState, analysis: Analysis, config: Config) -> Self {
        let mut ui = Ui::default();
        ui.update(&git_state);
        Self {
            ui,
            git_state,
            analysis,
            config,
            should_quit: false,
            status_msg: None,
        }
    }

    /// Reload git state and analysis (called on file-system change or `r`).
    pub fn reload(&mut self) {
        let start_dir = self.git_state.start_dir.clone();
        match crate::git::collect(&self.git_state.repo_root, &start_dir) {
            Ok(new_state) => {
                match crate::analysis::run(&new_state) {
                    Ok(new_analysis) => {
                        self.git_state = new_state;
                        self.analysis = new_analysis;
                        self.ui.update(&self.git_state);
                        self.status_msg = Some("Reloaded.".to_string());
                    }
                    Err(e) => self.status_msg = Some(format!("Analysis error: {}", e)),
                }
            }
            Err(e) => self.status_msg = Some(format!("Git error: {}", e)),
        }
        // Reload config too (thresholds may have changed).
        if let Ok(cfg) = crate::config::load(&self.git_state.repo_root) {
            self.config = cfg;
        }
    }

    /// Handle a single crossterm event. Returns `true` if the UI needs redrawing.
    pub fn handle_event(&mut self, event: Event) -> bool {
        self.status_msg = None;
        match event {
            Event::Key(key) => {
                // Global shortcuts (work regardless of focus).
                match key.code {
                    KeyCode::Char('q') => {
                        self.should_quit = true;
                        return false;
                    }
                    KeyCode::Tab => {
                        self.ui.toggle_focus();
                        return true;
                    }
                    KeyCode::Char('r') => {
                        self.reload();
                        return true;
                    }
                    _ => {}
                }

                // Focus-specific shortcuts.
                match self.ui.focus {
                    Focus::Right => self.handle_right_keys(key.code),
                    Focus::Left => self.handle_left_keys(key.code),
                }
                true
            }
            Event::Resize(_, _) => true,
            _ => false,
        }
    }

    fn handle_right_keys(&mut self, code: KeyCode) {
        match code {
            KeyCode::Char('j') | KeyCode::Down => {
                self.ui.right.select_next();
            }
            KeyCode::Char('k') | KeyCode::Up => {
                self.ui.right.select_prev();
            }
            // Scroll diff pane with Ctrl+j / Ctrl+k or d/u.
            KeyCode::Char('d') => {
                self.ui.right.scroll_diff_down(5);
            }
            KeyCode::Char('u') => {
                self.ui.right.scroll_diff_up(5);
            }
            _ => {}
        }
    }

    fn handle_left_keys(&mut self, code: KeyCode) {
        match code {
            KeyCode::Char('j') | KeyCode::Down => {
                self.ui.left.scroll_graph_down(1);
            }
            KeyCode::Char('k') | KeyCode::Up => {
                self.ui.left.scroll_graph_up(1);
            }
            KeyCode::Char('d') => {
                self.ui.left.scroll_metrics_down(3);
            }
            KeyCode::Char('u') => {
                self.ui.left.scroll_metrics_up(3);
            }
            _ => {}
        }
    }

    /// Poll for crossterm events with the given timeout. Returns whether an event was handled.
    pub fn poll_events(&mut self, timeout: Duration) -> Result<bool> {
        if event::poll(timeout)? {
            let ev = event::read()?;
            Ok(self.handle_event(ev))
        } else {
            Ok(false)
        }
    }

    /// Check the watcher channel for a reload signal.
    pub fn check_watcher(&mut self, watcher_rx: &Receiver<()>) -> bool {
        if watcher_rx.try_recv().is_ok() {
            // Drain burst.
            while watcher_rx.try_recv().is_ok() {}
            self.reload();
            true
        } else {
            false
        }
    }
}
