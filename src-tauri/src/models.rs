use serde::Serialize;

#[derive(Serialize, Clone)]
pub struct ProcessInfo {
    pub pid: u32,
    pub name: String,
    pub cpu_usage: f32,
    pub memory_usage: u64,
    pub thread_count: u64,
    pub children: Vec<ProcessInfo>,
}

#[derive(Serialize, Clone)]
pub struct SystemStats {
    pub cpu_usage: f32,
    pub memory_used: u64,
    pub memory_total: u64,
    pub network_up: u64,
    pub network_down: u64,
    pub top_processes: Vec<ProcessInfo>,
}

#[derive(Serialize, Clone)]
pub struct ProcessDetails {
    pub pid: u32,
    pub name: String,
    pub cmd: Vec<String>,
    pub exe: String,
    pub cwd: String,
    pub root: String,
    pub status: String,
    pub run_time: u64,
    pub memory_usage: u64,
    pub cpu_usage: f32,
}
