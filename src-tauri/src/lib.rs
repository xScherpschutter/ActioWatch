mod commands;
mod models;
mod monitor;
mod tray;

use commands::network::get_open_ports;
use commands::process::{
    get_process_affinity, get_process_details, get_process_modules, kill_process,
    set_process_affinity, set_process_priority,
};
use commands::startup::{get_startup_apps, toggle_startup_app};
use tauri::Manager;

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
            get_process_modules,
            get_open_ports,
            get_startup_apps,
            toggle_startup_app,
            set_process_priority,
            get_process_affinity,
            set_process_affinity
        ])
        .setup(|app| {
            // Create tray icon
            let _tray = tray::create_tray(app)?;

            // Start system monitoring in background
            monitor::start_monitoring(app.handle().clone());

            // Check for --minimized argument
            #[cfg(desktop)]
            if let Some(window) = app.get_webview_window("main") {
                let args: Vec<String> = std::env::args().collect();
                if args.contains(&"--minimized".to_string()) {
                    window.hide().unwrap();
                }

                // Handle Close Requested Event (Minimize instead of Close)
                let window_clone = window.clone();
                window.on_window_event(move |event| {
                    if let tauri::WindowEvent::CloseRequested { api, .. } = event {
                        window_clone.hide().unwrap();
                        api.prevent_close();
                    }
                });
            }

            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
