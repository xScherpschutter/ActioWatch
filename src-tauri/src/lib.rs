mod models;

use models::{ProcessInfo, SystemStats};
use std::collections::HashSet;
use std::time::Duration;
use sysinfo::{Networks, Pid, System};
use tauri::menu::{Menu, MenuItem};
use tauri::tray::TrayIconBuilder;
use tauri::Manager;
use tauri::{Emitter, LogicalSize, Size};
use tauri_plugin_notification::NotificationExt;

#[tauri::command]
fn kill_process(pid: u32) -> Result<bool, String> {
    let mut sys = System::new();

    sys.refresh_processes();

    if let Some(process) = sys.process(Pid::from(pid as usize)) {
        return Ok(process.kill());
    }

    Err(format!("PID process {} not found", pid))
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_notification::init())
        .invoke_handler(tauri::generate_handler![kill_process])
        .setup(|app| {
            let quit_i = MenuItem::with_id(app, "quit", "Quit", true, None::<&str>)?;
            let show_i = MenuItem::with_id(app, "show", "Show", true, None::<&str>)?;
            let hide_i = MenuItem::with_id(app, "hide", "Hide", true, None::<&str>)?;
            let widget_i = MenuItem::with_id(app, "widget", "Widget Mode", true, None::<&str>)?;
            let standard_i =
                MenuItem::with_id(app, "standard", "Standard Mode", true, None::<&str>)?;
            let menu = Menu::with_items(app, &[&show_i, &hide_i, &standard_i, &widget_i, &quit_i])?;

            let _tray = TrayIconBuilder::with_id("tray")
                .menu(&menu)
                .show_menu_on_left_click(true)
                .on_menu_event(|app, event| match event.id.as_ref() {
                    "quit" => {
                        app.exit(0);
                    }
                    "show" => {
                        if let Some(window) = app.get_webview_window("main") {
                            let _ = window.show();
                            let _ = window.set_focus();
                        }
                    }
                    "hide" => {
                        if let Some(window) = app.get_webview_window("main") {
                            let _ = window.hide();
                        }
                    }
                    "widget" => {
                        if let Some(window) = app.get_webview_window("main") {
                            let _ = window.set_size(Size::Logical(LogicalSize {
                                width: 320.0,
                                height: 480.0,
                            }));
                            let _ = window.emit("view-change", "widget");
                            let _ = window.show();
                        }
                    }
                    "standard" => {
                        if let Some(window) = app.get_webview_window("main") {
                            let _ = window.set_size(Size::Logical(LogicalSize {
                                width: 800.0,
                                height: 600.0,
                            }));
                            let _ = window.emit("view-change", "process");
                            let _ = window.show();
                        }
                    }
                    _ => {}
                })
                .build(app)?;

            let app_handle = app.handle().clone();

            tauri::async_runtime::spawn(async move {
                let mut sys = System::new_all();
                let mut networks = Networks::new_with_refreshed_list();
                let mut high_cpu_count = 0;
                let mut high_memory_count = 0;
                let mut notified_pids = HashSet::new();

                loop {
                    // Refresh CPU, Memory, Processes and Networks
                    sys.refresh_cpu();
                    sys.refresh_memory();
                    sys.refresh_processes();
                    networks.refresh();

                    // Calculate global CPU usage
                    let global_cpu = sys.global_cpu_info().cpu_usage();
                    let memory_used = sys.used_memory();
                    let memory_total = sys.total_memory();

                    // Calculate global Network usage
                    let mut network_up = 0;
                    let mut network_down = 0;
                    for (_interface_name, data) in &networks {
                        network_up += data.transmitted();
                        network_down += data.received();
                    }

                    // Get top 5 processes by CPU
                    let mut processes: Vec<ProcessInfo> = sys
                        .processes()
                        .iter()
                        .map(|(pid, process)| ProcessInfo {
                            pid: pid.as_u32(),
                            name: process.name().to_string(),
                            cpu_usage: process.cpu_usage(),
                            memory_usage: process.memory(),
                            // sysinfo 0.30: tasks() returns Option<&HashSet<Pid>>
                            thread_count: process.tasks().map(|t| t.len() as u64).unwrap_or(0),
                        })
                        .collect();

                    // Sort processes by CPU usage (descending)
                    processes.sort_by(|a, b| {
                        b.cpu_usage
                            .partial_cmp(&a.cpu_usage)
                            .unwrap_or(std::cmp::Ordering::Equal)
                    });

                    let stats = SystemStats {
                        cpu_usage: global_cpu,
                        memory_used,
                        memory_total,
                        network_up,
                        network_down,
                        top_processes: processes,
                    };

                    // Emit stats to frontend
                    if let Err(e) = app_handle.emit("stats-update", &stats) {
                        eprintln!("Error emitting stats: {}", e);
                    }

                    // Watchdog logic
                    if global_cpu > 90.0 {
                        high_cpu_count += 1;
                    } else {
                        high_cpu_count = 0;
                    }

                    if high_cpu_count >= 3 {
                        // Show native notification for High CPU
                        let _ = app_handle
                            .notification()
                            .builder()
                            .title("High CPU Alert")
                            .body("System CPU usage is critically high (> 90%)")
                            .show();

                        // Reset counter to avoid spamming immediately
                        high_cpu_count = 0;
                    }

                    // Check for high memory usage (> 25% of total memory)
                    let memory_threshold = sys.total_memory() / 4;
                    for (pid, process) in sys.processes() {
                        let memory_bytes = process.memory();
                        if memory_bytes > memory_threshold {
                            if !notified_pids.contains(pid) {
                                let _ = app_handle
                                    .notification()
                                    .builder()
                                    .title("High Memory Usage")
                                    .body(&format!(
                                        "Process {} is using {:.2} GB RAM",
                                        process.name(),
                                        memory_bytes as f64 / 1_073_741_824.0
                                    ))
                                    .show();

                                notified_pids.insert(*pid);
                            }
                        }
                    }

                    // Alert for high memory usage
                    let memory_used = sys.used_memory() as f64;
                    let total_memory = sys.total_memory() as f64;

                    if memory_used > total_memory * 0.90 {
                        high_memory_count += 1;
                    } else {
                        high_memory_count = 0;
                    }

                    if high_memory_count >= 3 {
                        let _ = app_handle
                            .notification()
                            .builder()
                            .title("Memory Alert")
                            .body(&format!(
                                "System memory usage is critically high ({:.1}%)",
                                memory_used / total_memory * 100.0
                            ))
                            .show();

                        high_memory_count = 0;
                    }

                    // Wait 1 second
                    tokio::time::sleep(Duration::from_secs(1)).await;
                }
            });

            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
