use crate::models::StartupApp;

#[cfg(target_os = "windows")]
use winreg::{enums::*, RegKey};

#[tauri::command]
pub fn get_startup_apps() -> Result<Vec<StartupApp>, String> {
    let mut apps = Vec::new();

    #[cfg(target_os = "windows")]
    {
        let hkcu = RegKey::predef(HKEY_CURRENT_USER);
        let run_path = "Software\\Microsoft\\Windows\\CurrentVersion\\Run";
        let disabled_path = "Software\\ActioWatch\\DisabledStartup";

        // Read Enabled Apps
        if let Ok(run_key) = hkcu.open_subkey(run_path) {
            for (name, value) in run_key.enum_values().map(|x| x.unwrap()) {
                apps.push(StartupApp {
                    name,
                    path: value.to_string(),
                    enabled: true,
                });
            }
        }

        // Read Disabled Apps
        if let Ok(disabled_key) = hkcu.open_subkey(disabled_path) {
            for (name, value) in disabled_key.enum_values().map(|x| x.unwrap()) {
                // Check if it's already in the list (moved from disabled to enabled manually?)
                if !apps.iter().any(|a| a.name == name) {
                    apps.push(StartupApp {
                        name,
                        path: value.to_string(),
                        enabled: false,
                    });
                }
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

                    if let (Some(n), Some(e)) = (name, exec) {
                        apps.push(StartupApp {
                            name: n,
                            path: e,
                            enabled: !hidden,
                        });
                    }
                }
            }
        }
    }

    Ok(apps)
}

#[tauri::command]
pub fn toggle_startup_app(name: String, path: String, enable: bool) -> Result<(), String> {
    #[cfg(target_os = "windows")]
    {
        let hkcu = RegKey::predef(HKEY_CURRENT_USER);
        let run_path = "Software\\Microsoft\\Windows\\CurrentVersion\\Run";
        let disabled_path = "Software\\ActioWatch\\DisabledStartup";

        if enable {
            // Enable: Add to Run, Remove from Disabled
            let run_key = hkcu
                .open_subkey_with_flags(run_path, KEY_WRITE)
                .map_err(|e| e.to_string())?;
            run_key.set_value(&name, &path).map_err(|e| e.to_string())?;

            if let Ok(disabled_key) = hkcu.open_subkey_with_flags(disabled_path, KEY_WRITE) {
                let _ = disabled_key.delete_value(&name);
            }
        } else {
            // Disable: Add to Disabled, Remove from Run
            let (disabled_key, _) = hkcu
                .create_subkey(disabled_path)
                .map_err(|e| e.to_string())?;
            disabled_key
                .set_value(&name, &path)
                .map_err(|e| e.to_string())?;

            if let Ok(run_key) = hkcu.open_subkey_with_flags(run_path, KEY_WRITE) {
                let _ = run_key.delete_value(&name);
            }
        }
        return Ok(());
    }

    #[cfg(target_os = "linux")]
    {
        let config_dir = std::env::var("XDG_CONFIG_HOME")
            .map(std::path::PathBuf::from)
            .or_else(|_| {
                std::env::var("HOME").map(|home| std::path::Path::new(&home).join(".config"))
            })
            .map_err(|_| "Could not find config directory")?;

        let autostart_dir = config_dir.join("autostart");

        let entries = std::fs::read_dir(&autostart_dir).map_err(|e| e.to_string())?;

        for entry in entries.flatten() {
            let file_path = entry.path();
            if file_path.extension().and_then(|s| s.to_str()) != Some("desktop") {
                continue;
            }

            if let Ok(content) = std::fs::read_to_string(&file_path) {
                let mut is_match = false;
                if content.contains(&format!("Name={}", name))
                    && content.contains(&format!("Exec={}", path))
                {
                    is_match = true;
                }

                if is_match {
                    let new_lines: Vec<String> = content
                        .lines()
                        .filter_map(|line| {
                            if line.starts_with("Hidden=") {
                                None // Remove existing Hidden
                            } else {
                                Some(line.to_string())
                            }
                        })
                        .collect();

                    let mut new_content = new_lines.join("\n");
                    if !enable {
                        new_content.push_str("\nHidden=true\n");
                    } else {
                        new_content.push_str("\nHidden=false\n");
                    }

                    std::fs::write(&file_path, new_content).map_err(|e| e.to_string())?;
                    return Ok(());
                }
            }
        }
        return Err("Application entry not found".to_string());
    }

    #[cfg(not(any(target_os = "windows", target_os = "linux")))]
    {
        return Err("Unsupported OS".to_string());
    }
}
