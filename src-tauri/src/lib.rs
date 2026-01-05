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

#[tauri::command]
fn get_process_details(pid: u32) -> Result<models::ProcessDetails, String> {
    let mut sys = System::new();
    sys.refresh_processes();

    if let Some(process) = sys.process(Pid::from(pid as usize)) {
        Ok(models::ProcessDetails {
            pid: process.pid().as_u32(),
            name: process.name().to_string(),
            cmd: process.cmd().to_vec(),
            exe: process
                .exe()
                .map(|p| p.to_string_lossy().to_string())
                .unwrap_or_default(),
            cwd: process
                .cwd()
                .map(|p| p.to_string_lossy().to_string())
                .unwrap_or_default(),
            root: process
                .root()
                .map(|p| p.to_string_lossy().to_string())
                .unwrap_or_default(),
            status: format!("{:?}", process.status()),
            run_time: process.run_time(),
            memory_usage: process.memory(),
            cpu_usage: process.cpu_usage(),
        })
    } else {
        Err(format!("Process {} not found", pid))
    }
}

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
        .invoke_handler(tauri::generate_handler![kill_process, get_process_details])
        .setup(|app| {
            let quit_i = MenuItem::with_id(app, "quit", "Quit", true, None::<&str>)?;
            let show_i = MenuItem::with_id(app, "show", "Show", true, None::<&str>)?;
            let hide_i = MenuItem::with_id(app, "hide", "Hide", true, None::<&str>)?;
            let widget_i = MenuItem::with_id(app, "widget", "Widget Mode", true, None::<&str>)?;
            let standard_i =
                MenuItem::with_id(app, "standard", "Standard Mode", true, None::<&str>)?;
            let menu = Menu::with_items(app, &[&show_i, &hide_i, &standard_i, &widget_i, &quit_i])?;

            // Build tray icon with explicit icon for Windows
            let tray_builder = TrayIconBuilder::with_id("tray")
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
                });

            // Set icon explicitly for Windows
            #[cfg(target_os = "windows")]
            let tray_builder = tray_builder.icon(
                app.default_window_icon()
                    .expect("Failed to load window icon")
                    .clone(),
            );

            let _tray = tray_builder.build(app)?;

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
                    // Refresh processes including Disk Usage
                    sys.refresh_processes_specifics(
                        sysinfo::ProcessRefreshKind::new()
                            .with_cpu()
                            .with_memory()
                            .with_disk_usage(),
                    );
                    networks.refresh();

                    // Calculate global CPU usage
                    let global_cpu = sys.global_cpu_info().cpu_usage();
                    let memory_used = sys.used_memory();
                    let memory_total = sys.total_memory();

                    // Calculate global Network usage
                    let mut network_up = 0;
                    let mut network_down = 0;
                    for (_name, data) in &networks {
                        network_up += data.transmitted();
                        network_down += data.received();
                        // Optional: verbose logging to debug specific interfaces
                        // println!("Interface {}: Up: {}, Down: {}", name, data.transmitted(), data.received());
                    }

                    // println!(
                    //     "Total Network: Up: {}, Down: {}, Interfaces: {}",
                    //     network_up,
                    //     network_down,
                    //     networks.len()
                    // );

                    // Create a map of PID -> Children PIDs
                    let mut children_map: std::collections::HashMap<u32, Vec<u32>> =
                        std::collections::HashMap::new();
                    let mut all_pids: HashSet<u32> = HashSet::new();

                    for (pid, process) in sys.processes() {
                        let pid_u32 = pid.as_u32();
                        all_pids.insert(pid_u32);
                        if let Some(parent) = process.parent() {
                            children_map
                                .entry(parent.as_u32())
                                .or_default()
                                .push(pid_u32);
                        }
                    }

                    // Helper to build process node with aggregation
                    fn build_process_node(
                        pid: u32,
                        sys: &sysinfo::System,
                        children_map: &std::collections::HashMap<u32, Vec<u32>>,
                    ) -> Option<ProcessInfo> {
                        if let Some(process) = sys.process(Pid::from(pid as usize)) {
                            let disk_usage = process.disk_usage();
                            let mut node = ProcessInfo {
                                pid,
                                name: process.name().to_string(),
                                cpu_usage: process.cpu_usage(),
                                total_cpu_usage: process.cpu_usage(),
                                memory_usage: process.memory(),
                                total_memory_usage: process.memory(),
                                disk_read: disk_usage.read_bytes,
                                disk_write: disk_usage.written_bytes,
                                total_disk_read: disk_usage.read_bytes,
                                total_disk_write: disk_usage.written_bytes,
                                thread_count: process.tasks().map(|t| t.len() as u64).unwrap_or(0),
                                children: Vec::new(),
                            };

                            let mut children_total_cpu = 0.0;
                            let mut children_total_mem = 0;
                            let mut children_total_disk_read = 0;
                            let mut children_total_disk_write = 0;

                            if let Some(children_pids) = children_map.get(&pid) {
                                for &child_pid in children_pids {
                                    if let Some(child_node) =
                                        build_process_node(child_pid, sys, children_map)
                                    {
                                        children_total_cpu += child_node.total_cpu_usage;
                                        children_total_mem += child_node.total_memory_usage;
                                        children_total_disk_read += child_node.total_disk_read;
                                        children_total_disk_write += child_node.total_disk_write;
                                        node.children.push(child_node);
                                    }
                                }
                            }

                            // Update totals
                            node.total_cpu_usage += children_total_cpu;
                            node.total_memory_usage += children_total_mem;
                            node.total_disk_read += children_total_disk_read;
                            node.total_disk_write += children_total_disk_write;

                            // Sort children by Total CPU usage
                            node.children.sort_by(|a, b| {
                                b.total_cpu_usage
                                    .partial_cmp(&a.total_cpu_usage)
                                    .unwrap_or(std::cmp::Ordering::Equal)
                            });

                            Some(node)
                        } else {
                            None
                        }
                    }

                    // Identify roots
                    let roots: Vec<u32> = sys
                        .processes()
                        .iter()
                        .filter_map(|(pid, process)| {
                            let pid_u32 = pid.as_u32();
                            let parent_pid = process.parent().map(|p| p.as_u32());

                            match parent_pid {
                                None => Some(pid_u32),
                                Some(ppid) => {
                                    if !all_pids.contains(&ppid) {
                                        Some(pid_u32)
                                    } else {
                                        None
                                    }
                                }
                            }
                        })
                        .collect();

                    let mut processes: Vec<ProcessInfo> = Vec::new();
                    for root_pid in roots {
                        if let Some(node) = build_process_node(root_pid, &sys, &children_map) {
                            processes.push(node);
                        }
                    }

                    // Sort root processes by CPU usage (descending)
                    processes.sort_by(|a, b| {
                        b.total_cpu_usage
                            .partial_cmp(&a.total_cpu_usage)
                            .unwrap_or(std::cmp::Ordering::Equal)
                    });

                    // Aggregate global disk usage from processes
                    let mut total_disk_read = 0;
                    let mut total_disk_write = 0;
                    for p in &processes {
                        total_disk_read += p.total_disk_read;
                        total_disk_write += p.total_disk_write;
                    }

                    let stats = SystemStats {
                        cpu_usage: global_cpu,
                        memory_used,
                        memory_total,
                        network_up,
                        network_down,
                        disk_read: total_disk_read,
                        disk_write: total_disk_write,
                        gpu_usage: None,
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
                    if memory_used as f64 > memory_total as f64 * 0.90 {
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
                                memory_used as f64 / memory_total as f64 * 100.0
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
