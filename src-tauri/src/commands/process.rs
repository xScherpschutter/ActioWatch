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

#[tauri::command]
pub fn set_process_priority(pid: u32, priority: String) -> Result<bool, String> {
    #[cfg(target_os = "windows")]
    {
        use windows::Win32::Foundation::CloseHandle;
        use windows::Win32::System::Threading::{
            OpenProcess, SetPriorityClass, ABOVE_NORMAL_PRIORITY_CLASS,
            BELOW_NORMAL_PRIORITY_CLASS, HIGH_PRIORITY_CLASS, IDLE_PRIORITY_CLASS,
            NORMAL_PRIORITY_CLASS, PROCESS_SET_INFORMATION, REALTIME_PRIORITY_CLASS,
        };

        let priority_class = match priority.as_str() {
            "Realtime" => REALTIME_PRIORITY_CLASS,
            "High" => HIGH_PRIORITY_CLASS,
            "Above Normal" => ABOVE_NORMAL_PRIORITY_CLASS,
            "Normal" => NORMAL_PRIORITY_CLASS,
            "Below Normal" => BELOW_NORMAL_PRIORITY_CLASS,
            "Low" => IDLE_PRIORITY_CLASS,
            _ => return Err("Invalid priority level".to_string()),
        };

        unsafe {
            let handle = OpenProcess(PROCESS_SET_INFORMATION, false, pid)
                .map_err(|e| format!("Failed to open process: {}", e))?;

            if handle.is_invalid() {
                return Err("Invalid process handle".to_string());
            }

            let result = SetPriorityClass(handle, priority_class);
            let _ = CloseHandle(handle);

            match result {
                Ok(_) => Ok(true),
                Err(e) => Err(format!("Failed to set priority: {}", e)),
            }
        }
    }

    #[cfg(target_os = "linux")]
    {
        // Map abstract priority levels to nice values (-20 to 19)
        // Lower is higher priority
        let nice_value = match priority.as_str() {
            "Realtime" => -20, // Requires root
            "High" => -10,
            "Above Normal" => -5,
            "Normal" => 0,
            "Below Normal" => 5,
            "Low" => 19,
            _ => return Err("Invalid priority level".to_string()),
        };

        unsafe {
            // setpriority(which, who, prio)
            // PRIO_PROCESS is 0
            let ret = libc::setpriority(0, pid, nice_value);
            if ret == 0 {
                Ok(true)
            } else {
                let err = std::io::Error::last_os_error();
                // If permission denied, try pkexec
                if err.kind() == std::io::ErrorKind::PermissionDenied {
                    use std::process::Command;
                    let status = Command::new("pkexec")
                        .arg("renice")
                        .arg("-n")
                        .arg(nice_value.to_string())
                        .arg("-p")
                        .arg(pid.to_string())
                        .status();

                    match status {
                        Ok(exit_status) => {
                            if exit_status.success() {
                                Ok(true)
                            } else {
                                Err(format!(
                                    "Failed to set priority via pkexec: {}",
                                    exit_status
                                ))
                            }
                        }
                        Err(e) => Err(format!("Failed to execute pkexec: {}", e)),
                    }
                } else {
                    Err(format!("Failed to set priority: {}", err))
                }
            }
        }
    }

    #[cfg(not(any(target_os = "windows", target_os = "linux")))]
    {
        Err("Not supported on this OS".to_string())
    }
}

#[tauri::command]
pub fn get_process_affinity(pid: u32) -> Result<Vec<u32>, String> {
    #[cfg(target_os = "windows")]
    {
        use windows::Win32::Foundation::CloseHandle;
        use windows::Win32::System::Threading::{
            GetProcessAffinityMask, OpenProcess, PROCESS_QUERY_LIMITED_INFORMATION,
        };

        unsafe {
            let handle = OpenProcess(PROCESS_QUERY_LIMITED_INFORMATION, false, pid)
                .map_err(|e| format!("Failed to open process: {}", e))?;

            if handle.is_invalid() {
                return Err("Invalid process handle".to_string());
            }

            let mut process_mask = 0;
            let mut system_mask = 0;

            let result = GetProcessAffinityMask(handle, &mut process_mask, &mut system_mask);
            let _ = CloseHandle(handle);

            if result.is_err() {
                return Err("Failed to get affinity mask".to_string());
            }

            let mut cpus = Vec::new();
            for i in 0..64 {
                if (process_mask >> i) & 1 == 1 {
                    cpus.push(i);
                }
            }
            Ok(cpus)
        }
    }

    #[cfg(target_os = "linux")]
    {
        use nix::sched::sched_getaffinity;
        use nix::unistd::Pid;
        use sysinfo::{CpuRefreshKind, RefreshKind, System};

        match sched_getaffinity(Pid::from_raw(pid as i32)) {
            Ok(cpuset) => {
                let mut cpus = Vec::new();
                let s = System::new_with_specifics(
                    RefreshKind::new().with_cpu(CpuRefreshKind::everything()),
                );
                for i in 0..s.cpus().len() {
                    if cpuset.is_set(i).unwrap_or(false) {
                        cpus.push(i as u32);
                    }
                }
                Ok(cpus)
            }
            Err(e) => Err(format!("Failed to get affinity: {}", e)),
        }
    }

    #[cfg(not(any(target_os = "windows", target_os = "linux")))]
    {
        Err("Not supported on this OS".to_string())
    }
}

#[tauri::command]
pub fn set_process_affinity(pid: u32, cpus: Vec<u32>) -> Result<bool, String> {
    if cpus.is_empty() {
        return Err("At least one CPU must be selected".to_string());
    }

    #[cfg(target_os = "windows")]
    {
        use windows::Win32::Foundation::CloseHandle;
        use windows::Win32::System::Threading::{
            OpenProcess, SetProcessAffinityMask, PROCESS_SET_INFORMATION,
        };

        unsafe {
            let handle = OpenProcess(PROCESS_SET_INFORMATION, false, pid)
                .map_err(|e| format!("Failed to open process: {}", e))?;

            if handle.is_invalid() {
                return Err("Invalid process handle".to_string());
            }

            let mut mask: usize = 0;
            for cpu in cpus {
                if cpu < 64 {
                    mask |= 1 << cpu;
                }
            }

            let result = SetProcessAffinityMask(handle, mask);
            let _ = CloseHandle(handle);

            match result {
                Ok(_) => Ok(true),
                Err(e) => Err(format!("Failed to set affinity: {}", e)),
            }
        }
    }

    #[cfg(target_os = "linux")]
    {
        use nix::sched::{sched_setaffinity, CpuSet};
        use nix::unistd::Pid;

        let mut cpuset = CpuSet::new();
        for cpu in cpus {
            // CpuSet size is limited, typically 1024
            if let Err(_) = cpuset.set(cpu as usize) {
                return Err(format!("Invalid CPU index: {}", cpu));
            }
        }

        match sched_setaffinity(Pid::from_raw(pid as i32), &cpuset) {
            Ok(_) => Ok(true),
            Err(e) => Err(format!("Failed to set affinity: {}", e)),
        }
    }

    #[cfg(not(any(target_os = "windows", target_os = "linux")))]
    {
        Err("Not supported on this OS".to_string())
    }
}
