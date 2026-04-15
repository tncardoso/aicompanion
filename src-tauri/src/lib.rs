mod analysis;
mod commands;
mod config;
mod git;

use commands::WatcherState;
use std::sync::Mutex;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    println!("dd");
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .manage(WatcherState(Mutex::new(None)))
        .invoke_handler(tauri::generate_handler![
            commands::get_cwd,
            commands::get_git_state,
            commands::run_analysis,
            commands::get_config,
            commands::watch_repo,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
