use std::sync::Mutex;
use tauri::{AppHandle, Emitter, State};

use crate::{analysis, config, git};

pub struct WatcherState(pub Mutex<Option<notify::RecommendedWatcher>>);

#[tauri::command]
pub async fn get_cwd() -> String {
    std::env::current_dir()
        .unwrap_or_default()
        .to_string_lossy()
        .into_owned()
}

#[tauri::command]
pub async fn get_git_state(repo_path: String) -> Result<git::GitState, String> {
    let path = std::path::Path::new(&repo_path);
    let root = git::find_repo_root(path).map_err(|e| e.to_string())?;
    // Always collect from repo root so the diff is never scoped to a subdir
    git::collect(&root, &root).map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn run_analysis(repo_path: String) -> Result<analysis::Analysis, String> {
    let path = std::path::Path::new(&repo_path);
    let root = git::find_repo_root(path).map_err(|e| e.to_string())?;
    let state = git::collect(&root, &root).map_err(|e| e.to_string())?;
    analysis::run(&state).map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn get_config(repo_path: String) -> Result<config::Config, String> {
    let path = std::path::Path::new(&repo_path);
    let root = git::find_repo_root(path).map_err(|e| e.to_string())?;
    config::load(&root).map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn watch_repo(
    repo_path: String,
    app: AppHandle,
    watcher_state: State<'_, WatcherState>,
) -> Result<(), String> {
    use notify::{Event, RecursiveMode, Watcher};

    let root = git::find_repo_root(std::path::Path::new(&repo_path))
        .map_err(|e| e.to_string())?;
    let app_handle = app.clone();

    let mut watcher = notify::recommended_watcher(move |res: Result<Event, notify::Error>| {
        if let Ok(event) = res {
            let is_git = event.paths.iter().all(|p| {
                p.components().any(|c| c.as_os_str() == ".git")
            });
            if !is_git {
                let _ = app_handle.emit("repo-changed", ());
            }
        }
    })
    .map_err(|e| e.to_string())?;

    watcher
        .watch(&root, RecursiveMode::Recursive)
        .map_err(|e| e.to_string())?;

    *watcher_state.0.lock().unwrap() = Some(watcher);
    Ok(())
}
