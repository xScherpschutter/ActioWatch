use crate::models::ProcessDetails;
use sysinfo::{Pid, System};

#[tauri::command]
pub fn kill_process(pid: u32) -> Result<bool, String> {
    let mut sys = System::new();
    sys.refresh_processes();

    if let Some(process) = sys.process(Pid::from(pid as usize)) {
        return Ok(process.kill());
    }

    Err(format!("PID process {} not found", pid))
}

#[tauri::command]
pub fn get_process_details(pid: u32) -> Result<ProcessDetails, String> {
    let mut sys = System::new();
    sys.refresh_processes();

    if let Some(process) = sys.process(Pid::from(pid as usize)) {
        Ok(ProcessDetails {
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
            environ: process.environ().to_vec(),
        })
    } else {
        Err(format!("Process {} not found", pid))
    }
}

#[tauri::command]
pub fn get_process_modules(pid: u32) -> Result<Vec<crate::models::ModuleInfo>, String> {
    let mut modules = Vec::new();

    #[cfg(target_os = "windows")]
    {
        use std::mem::size_of;
        use windows::Win32::Foundation::{CloseHandle, HMODULE};
        use windows::Win32::System::ProcessStatus::{EnumProcessModules, GetModuleFileNameExW};
        use windows::Win32::System::Threading::{
            OpenProcess, PROCESS_QUERY_INFORMATION, PROCESS_VM_READ,
        };

        unsafe {
            let handle = OpenProcess(PROCESS_QUERY_INFORMATION | PROCESS_VM_READ, false, pid)
                .map_err(|e| format!("Failed to open process: {}", e))?;

            if !handle.is_invalid() {
                let mut h_modules = [HMODULE::default(); 1024];
                let mut cb_needed = 0;

                if EnumProcessModules(
                    handle,
                    h_modules.as_mut_ptr(),
                    (h_modules.len() * size_of::<HMODULE>()) as u32,
                    &mut cb_needed,
                )
                .is_ok()
                {
                    let module_count = cb_needed as usize / size_of::<HMODULE>();
                    for i in 0..std::cmp::min(module_count, h_modules.len()) {
                        let mut buffer = [0u16; 1024];
                        let len =
                            GetModuleFileNameExW(Some(handle), Some(h_modules[i]), &mut buffer);
                        if len > 0 {
                            let path = String::from_utf16_lossy(&buffer[..len as usize]);
                            let name = std::path::Path::new(&path)
                                .file_name()
                                .map(|n| n.to_string_lossy().to_string())
                                .unwrap_or_else(|| "Unknown".to_string());

                            modules.push(crate::models::ModuleInfo { name, path });
                        }
                    }
                }
                let _ = CloseHandle(handle);
            }
        }
    }

    #[cfg(target_os = "linux")]
    {
        use std::io::BufRead;
        let maps_path = format!("/proc/{}/maps", pid);
        if let Ok(file) = std::fs::File::open(maps_path) {
            let reader = std::io::BufReader::new(file);
            let mut seen_paths = std::collections::HashSet::new();

            for line in reader.lines() {
                if let Ok(l) = line {
                    // Line format: address perms offset dev inode pathname
                    let parts: Vec<&str> = l.split_whitespace().collect();
                    if parts.len() >= 6 {
                        let path = parts[5];
                        if (path.ends_with(".so") || path.contains(".so."))
                            && !seen_paths.contains(path)
                        {
                            seen_paths.insert(path.to_string());
                            let name = std::path::Path::new(path)
                                .file_name()
                                .map(|n| n.to_string_lossy().to_string())
                                .unwrap_or_else(|| "Unknown".to_string());

                            modules.push(crate::models::ModuleInfo {
                                name,
                                path: path.to_string(),
                            });
                        }
                    }
                }
            }
        }
    }

    Ok(modules)
}
