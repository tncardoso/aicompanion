use anyhow::Result;
use notify::{Event, RecommendedWatcher, RecursiveMode, Watcher};
use std::path::Path;
use std::sync::mpsc::{self, Receiver};
use std::time::Duration;

/// Creates a filesystem watcher on `repo_root` and returns a receiver that
/// yields `()` whenever a debounced change is detected.
pub fn watch(repo_root: &Path) -> Result<(RecommendedWatcher, Receiver<()>)> {
    let (tx, rx) = mpsc::channel::<()>();

    let mut watcher = notify::recommended_watcher(move |res: Result<Event, _>| {
        if let Ok(event) = res {
            // Ignore .git internal churn (lock files, FETCH_HEAD, etc.)
            let is_git_internal = event.paths.iter().all(|p| {
                p.components().any(|c| c.as_os_str() == ".git")
            });
            if !is_git_internal {
                let _ = tx.send(());
            }
        }
    })?;

    watcher.watch(repo_root, RecursiveMode::Recursive)?;
    Ok((watcher, rx))
}

/// Drain all pending events from the channel, returning `true` if at least one
/// arrived. This lets the caller batch rapid bursts of saves into a single reload.
#[allow(dead_code)]
pub fn drain(rx: &Receiver<()>, timeout: Duration) -> bool {
    match rx.recv_timeout(timeout) {
        Ok(()) => {
            // Drain any additional events that arrived during debounce window.
            while rx.try_recv().is_ok() {}
            true
        }
        Err(_) => false,
    }
}
