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

    pub fn is_running(&mut self) -> bool {
        if let Some(ref mut child) = self.child {
            match child.try_wait() {
                Ok(Some(_)) => false,
                Ok(None) => true,
                Err(_) => false,
            }
        } else {
            false
        }
    }

    pub async fn start(&mut self, cfg: &super::config::AppConfig) -> anyhow::Result<()> {
        if !cfg.config_exists() {
            return Err(anyhow::anyhow!("config.json not found next to dakal.exe. Place your sing-box config in the same folder."));
        }

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
            .arg(cfg.config_path())
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
