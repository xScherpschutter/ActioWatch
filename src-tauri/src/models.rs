use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct StartupApp {
    pub name: String,
    pub path: String,
    pub enabled: bool,
}

#[derive(Serialize, Clone)]
pub struct ProcessInfo {
    pub pid: u32,
    pub name: String,
    pub cpu_usage: f32,
    pub total_cpu_usage: f32,
    pub memory_usage: u64,
    pub total_memory_usage: u64,
    pub disk_read: u64,
    pub disk_write: u64,
    pub total_disk_read: u64,
    pub total_disk_write: u64,
    pub thread_count: u64,
    pub children: Vec<ProcessInfo>,
}

#[derive(Serialize, Clone)]
pub struct ComponentInfo {
    pub label: String,
    pub temperature: f32,
    pub max_temperature: f32,
    pub critical_temperature: Option<f32>,
}

#[derive(Serialize, Clone)]
pub struct SystemStats {
    pub cpu_usage: f32,
    pub memory_used: u64,
    pub memory_total: u64,
    pub network_up: u64,
    pub network_down: u64,
    pub disk_read: u64,
    pub disk_write: u64,
    pub gpu_usage: Option<f32>,         // Keeping option open
    pub components: Vec<ComponentInfo>, // Sensor data
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
    pub environ: Vec<String>,
}

#[derive(Serialize, Clone)]
pub struct PortInfo {
    pub pid: Option<u32>,
    pub process_name: String,
    pub port: u16,
    pub protocol: String, // "TCP" or "UDP"
    pub address: String,
}
