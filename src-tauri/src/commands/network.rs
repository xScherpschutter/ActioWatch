use crate::models::{PortInfo};
use listeners;
use sysinfo::{Pid, System};

#[tauri::command]
pub fn get_open_ports() -> Result<Vec<PortInfo>, String> {
    let mut ports = Vec::new();
    let mut sys = System::new();
    sys.refresh_processes();

    // Use listeners crate to get open ports
    let listeners = listeners::get_all().map_err(|e| e.to_string())?;

    for l in listeners {
        let pid = l.process.pid;
        let process_name = sys
            .process(Pid::from(pid as usize))
            .map(|p| p.name().to_string())
            .unwrap_or_else(|| "Unknown".to_string());

        ports.push(PortInfo {
            pid: Some(pid),
            process_name,
            port: l.socket.port(),
            protocol: "TCP/UDP".to_string(),
            address: l.socket.to_string(),
        });
    }

    ports.sort_by_key(|p| p.port);
    Ok(ports)
}
