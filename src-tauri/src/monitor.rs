use crate::models::{ComponentInfo, ProcessInfo, SystemStats};
use std::collections::HashSet;
use std::time::Duration;
use sysinfo::{Components, Networks, Pid, System};
use tauri::{path::BaseDirectory, AppHandle, Emitter, Manager, Runtime};
use tauri_plugin_notification::NotificationExt;

pub fn start_monitoring<R: Runtime>(app_handle: AppHandle<R>) {
    tauri::async_runtime::spawn(async move {
        let mut sys = System::new_all();
        let mut networks = Networks::new_with_refreshed_list();
        let mut components = Components::new_with_refreshed_list();
        let mut high_cpu_count = 0;
        let mut high_memory_count = 0;
        let mut notified_pids = HashSet::new();
        let mut last_high_cpu_notification = std::time::Instant::now() - Duration::from_secs(60);
        let mut last_high_memory_notification = std::time::Instant::now() - Duration::from_secs(60);

        // Network refresh ticker
        let mut refresh_tick = 0;

        let icon_path_str = app_handle
            .path()
            .resolve("icons/icon.png", BaseDirectory::Resource)
            .ok()
            .map(|p| p.to_string_lossy().to_string());

        loop {
            refresh_tick += 1;

            // Refresh CPU, Memory, Processes
            sys.refresh_cpu();
            sys.refresh_memory();
            // Refresh processes including Disk Usage
            sys.refresh_processes_specifics(
                sysinfo::ProcessRefreshKind::new()
                    .with_cpu()
                    .with_memory()
                    .with_disk_usage(),
            );

            // Re-scan network interfaces periodically to catch new connections (e.g., VPN, WiFi switch)
            if refresh_tick % 10 == 0 {
                networks.refresh_list();
            }
            networks.refresh();

            components.refresh();

            // Calculate global CPU usage
            let global_cpu = sys.global_cpu_info().cpu_usage();
            let memory_used = sys.used_memory();
            let memory_total = sys.total_memory();

            // Calculate global Network usage
            // sysinfo::Networks::refresh() updates the data to show bytes transmitted/received
            // SINCE THE LAST REFRESH. Since we loop every 1s, this value IS the rate (Bytes/s).
            let mut network_up = 0;
            let mut network_down = 0;

            for (_name, data) in &networks {
                network_up += data.transmitted();
                network_down += data.received();
            }

            // Create a map of PID -> Children PIDs
            let mut children_map: std::collections::HashMap<u32, Vec<u32>> =
                std::collections::HashMap::new();
            let mut all_pids: HashSet<u32> = HashSet::new();

            for (pid, process) in sys.processes() {
                let pid_u32 = pid.as_u32();
                all_pids.insert(pid_u32);

                if let Some(parent) = process.parent() {
                    // General validation: Child cannot be older than parent.
                    // We check start_time() which is Unix timestamp in seconds.

                    let is_valid = if let Some(parent_proc) = sys.process(parent) {
                        let p_start = parent_proc.start_time();
                        let c_start = process.start_time();

                        if c_start < p_start {
                            // Child started BEFORE Parent -> Impossible -> Parent PID Reused.
                            false
                        } else if c_start == 0 && p_start == 0 {
                            // Both have 0 start time (likely Access Denied / System Processes).
                            // We cannot distinguish valid parentage from PID reuse.
                            // Defaulting to FALSE (Treat as Root) prevents the "Giant Tree" memory execution bug.
                            // Side Effect: System process tree might be flatter (services separate from wininit), but memory is correct.
                            false
                        } else {
                            // Child started After (or same second as) Parent -> Valid.
                            true
                        }
                    } else {
                        // Parent process not in list?
                        // If we can't find parent, we can't aggregate anyway.
                        true
                    };

                    if is_valid {
                        children_map
                            .entry(parent.as_u32())
                            .or_default()
                            .push(pid_u32);
                    }
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
                    let cpu_count = sys.cpus().len() as f32;

                    // Normalize CPU usage (0-100%) on all platforms
                    let cpu_usage = process.cpu_usage() / cpu_count;

                    let mut node = ProcessInfo {
                        pid,
                        name: process.name().to_string(),
                        cpu_usage,
                        total_cpu_usage: cpu_usage,
                        memory_usage: process.memory(),
                        total_memory_usage: process.memory(),
                        disk_read: disk_usage.read_bytes,
                        disk_write: disk_usage.written_bytes,
                        total_disk_read: disk_usage.read_bytes,
                        total_disk_write: disk_usage.written_bytes,
                        thread_count: process.tasks().map(|t| t.len() as u64).unwrap_or(0),
                        children: Vec::new(),
                    };

                    if let Some(children_pids) = children_map.get(&pid) {
                        for &child_pid in children_pids {
                            if let Some(child_node) =
                                build_process_node(child_pid, sys, children_map)
                            {
                                node.children.push(child_node);
                            }
                        }
                    }

                    // Update totals
                    // Aggregation disabled: Each process shows only its own stats.
                    // #[cfg(target_os = "windows")]
                    // for child in &node.children {
                    //    node.total_cpu_usage += child.total_cpu_usage;
                    //    node.total_memory_usage += child.total_memory_usage;
                    //    node.total_disk_read += child.total_disk_read;
                    //    node.total_disk_write += child.total_disk_write;
                    // }

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
                                return Some(pid_u32);
                            }

                            // Check link validity - mirror logic from children_map construction
                            // If link is invalid, it MUST be a root
                            if let Some(parent_proc) = sys.process(Pid::from(ppid as usize)) {
                                let p_start = parent_proc.start_time();
                                let c_start = process.start_time();

                                if c_start < p_start {
                                    // Invalid: Child older than parent -> Root
                                    Some(pid_u32)
                                } else if c_start == 0 && p_start == 0 {
                                    // Invalid: Ambiguous 0 start times -> Root
                                    Some(pid_u32)
                                } else {
                                    // Valid Child -> Not a root
                                    None
                                }
                            } else {
                                // Parent not found in sys (should be caught by all_pids check but safe fallback)
                                Some(pid_u32)
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

            // Debugging: Count total nodes in the tree to verify no data loss
            /*
            fn count_tree_nodes(nodes: &[ProcessInfo]) -> usize {
                let mut count = 0;
                for node in nodes {
                    count += 1;
                    count += count_tree_nodes(&node.children);
                }
                count
            }
            let tree_count = count_tree_nodes(&processes);
            let sys_count = sys.processes().len();
            println!(
                "DEBUG: SysInfo Total: {}, Frontend Tree Total: {}",
                sys_count, tree_count
            );
            */

            let stats = SystemStats {
                cpu_usage: global_cpu,
                process_count: sys.processes().len(),
                memory_used,
                memory_total,
                network_up,
                network_down,
                disk_read: total_disk_read,
                disk_write: total_disk_write,
                gpu_usage: None,
                components: components
                    .iter()
                    .map(|c| ComponentInfo {
                        label: c.label().to_string(),
                        temperature: c.temperature(),
                        max_temperature: c.max(),
                        critical_temperature: c.critical(),
                    })
                    .collect(),
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

            if high_cpu_count >= 5 {
                if last_high_cpu_notification.elapsed() > Duration::from_secs(60) {
                    if let Some(app_state) = app_handle.try_state::<crate::models::AppLifecycle>() {
                        if app_state
                            .notifications_enabled
                            .load(std::sync::atomic::Ordering::Relaxed)
                        {
                            let mut notification = app_handle
                                .notification()
                                .builder()
                                .title("High CPU Alert")
                                .body("System CPU usage is critically high (> 90%)");

                            if let Some(icon) = &icon_path_str {
                                notification = notification.icon(icon);
                            }

                            let _ = notification.show();
                        }
                    }

                    last_high_cpu_notification = std::time::Instant::now();
                }

                // Reset counter to avoid spamming immediately
                high_cpu_count = 0;
            }

            // Check for high memory usage (> 25% of total memory)
            let memory_threshold = sys.total_memory() / 4;
            for (pid, process) in sys.processes() {
                let memory_bytes = process.memory();
                if memory_bytes > memory_threshold {
                    if !notified_pids.contains(pid) {
                        if let Some(app_state) =
                            app_handle.try_state::<crate::models::AppLifecycle>()
                        {
                            if app_state
                                .notifications_enabled
                                .load(std::sync::atomic::Ordering::Relaxed)
                            {
                                let mut notification = app_handle
                                    .notification()
                                    .builder()
                                    .title("High Memory Usage")
                                    .body(&format!(
                                        "Process {} is using {:.2} GB RAM",
                                        process.name(),
                                        memory_bytes as f64 / 1_073_741_824.0
                                    ));

                                if let Some(icon) = &icon_path_str {
                                    notification = notification.icon(icon);
                                }

                                let _ = notification.show();
                            }
                        }

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
                if last_high_memory_notification.elapsed() > Duration::from_secs(60) {
                    if let Some(app_state) = app_handle.try_state::<crate::models::AppLifecycle>() {
                        if app_state
                            .notifications_enabled
                            .load(std::sync::atomic::Ordering::Relaxed)
                        {
                            let mut notification = app_handle
                                .notification()
                                .builder()
                                .title("Memory Alert")
                                .body(&format!(
                                    "System memory usage is critically high ({:.1}%)",
                                    memory_used as f64 / memory_total as f64 * 100.0
                                ));

                            if let Some(icon) = &icon_path_str {
                                notification = notification.icon(icon);
                            }

                            let _ = notification.show();
                        }
                    }
                    last_high_memory_notification = std::time::Instant::now();
                }

                high_memory_count = 0;
            }

            // Wait 1 second
            tokio::time::sleep(Duration::from_secs(1)).await;
        }
    });
}
