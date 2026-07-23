use std::process::{Child, Command, Stdio};
#[cfg(target_os = "windows")]
use std::os::windows::process::CommandExt;

pub struct SingBoxProcess {
    child: Option<Child>,
}

impl SingBoxProcess {
    pub fn new() -> Self {
        Self { child: None }
    }

    pub fn is_running(&self) -> bool {
        if let Some(ref mut child) = self.child {
            match child.try_wait() {
                Ok(Some(_)) => {
                    // Exited, clean up
                    false
                }
                Ok(None) => true,
                Err(_) => false,
            }
        } else {
            false
        }
    }

    pub async fn start(&mut self, cfg: &super::config::AppConfig, _app: &tauri::AppHandle) -> anyhow::Result<()> {
        // Write config to temp
        let temp_dir = std::env::temp_dir().join("dakal");
        std::fs::create_dir_all(&temp_dir)?;
        let config_path = temp_dir.join("config.json");

        // Load user config or use embedded default
        let config_content = match &cfg.config_path {
            Some(path) if std::path::Path::new(path).exists() => {
                std::fs::read_to_string(path)?
            }
            _ => cfg.generate_config(),
        };
        std::fs::write(&config_path, &config_content)?;

        // Locate sing-box.exe (same dir as app)
        let exe_dir = std::env::current_exe()?
            .parent()
            .expect("exe parent")
            .to_path_buf();
        let singbox = exe_dir.join("sing-box.exe");

        if !singbox.exists() {
            return Err(anyhow::anyhow!("sing-box.exe not found next to app. Place it in the same folder."));
        }

        let child = Command::new(&singbox)
            .arg("run")
            .arg("-c")
            .arg(&config_path)
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .creation_flags(0x08000000) // CREATE_NO_WINDOW
            .spawn()
            .map_err(|e| anyhow::anyhow!("Failed to start sing-box: {e}"))?;

        self.child = Some(child);
        Ok(())
    }

    pub async fn stop(&mut self) {
        if let Some(mut child) = self.child.take() {
            let _ = child.kill();
            let _ = child.wait();
        }
    }
}
