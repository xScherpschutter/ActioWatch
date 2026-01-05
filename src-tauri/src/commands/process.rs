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
