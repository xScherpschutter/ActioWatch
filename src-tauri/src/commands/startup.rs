use crate::models::{StartupApp};
use std::path::Path;

#[cfg(target_os = "windows")]
use winreg::enums::*;
#[cfg(target_os = "windows")]
use winreg::RegKey;

#[tauri::command]
pub fn get_startup_apps() -> Result<Vec<StartupApp>, String> {
    let mut apps = Vec::new();

    #[cfg(target_os = "windows")]
    {
        let hkcu = RegKey::predef(HKEY_CURRENT_USER);
        let path = Path::new("Software")
            .join("Microsoft")
            .join("Windows")
            .join("CurrentVersion")
            .join("Run");

        let path_str = path
            .to_str()
            .unwrap_or("Software\\Microsoft\\Windows\\CurrentVersion\\Run");

        if let Ok(run_key) = hkcu.open_subkey(path_str) {
            for (name, value) in run_key.enum_values().map(|x| x.unwrap()) {
                apps.push(StartupApp {
                    name,
                    path: value.to_string(),
                    enabled: true,
                });
            }
        }
    }

    #[cfg(target_os = "linux")]
    {
        let config_dir = std::env::var("XDG_CONFIG_HOME")
            .map(std::path::PathBuf::from)
            .or_else(|_| {
                std::env::var("HOME").map(|home| std::path::Path::new(&home).join(".config"))
            })
            .ok();

        if let Some(autostart_dir) = config_dir
            .map(|p| p.join("autostart"))
            .filter(|p| p.exists())
        {
            let entries = std::fs::read_dir(autostart_dir)
                .ok()
                .into_iter()
                .flatten()
                .flatten();

            for entry in entries {
                let path = entry.path();
                if path.extension().and_then(|s| s.to_str()) != Some("desktop") {
                    continue;
                }

                if let Ok(content) = std::fs::read_to_string(&path) {
                    let mut name = None;
                    let mut exec = None;
                    let mut hidden = false;

                    for line in content.lines() {
                        if let Some(val) = line.strip_prefix("Name=") {
                            name = Some(val.to_string());
                        } else if let Some(val) = line.strip_prefix("Exec=") {
                            exec = Some(val.to_string());
                        } else if let Some(val) = line.strip_prefix("Hidden=") {
                            hidden = val == "true";
                        }
                    }

                    if !hidden {
                        if let (Some(n), Some(e)) = (name, exec) {
                            apps.push(StartupApp {
                                name: n,
                                path: e,
                                enabled: true,
                            });
                        }
                    }
                }
            }
        }
    }

    Ok(apps)
}
