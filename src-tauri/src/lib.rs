mod commands;
mod models;
mod monitor;
mod tray;

use commands::network::get_open_ports;
use commands::process::{get_process_details, kill_process};
use commands::startup::get_startup_apps;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_os::init())
        .plugin(tauri_plugin_notification::init())
        .plugin(tauri_plugin_autostart::init(
            tauri_plugin_autostart::MacosLauncher::LaunchAgent,
            Some(vec!["--minimized"]),
        ))
        .invoke_handler(tauri::generate_handler![
            kill_process,
            get_process_details,
            get_open_ports,
            get_startup_apps
        ])
        .setup(|app| {
            // Create tray icon
            let _tray = tray::create_tray(app)?;

            // Start system monitoring in background
            monitor::start_monitoring(app.handle().clone());

            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
