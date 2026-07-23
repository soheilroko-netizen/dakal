use std::process::{Child, Command, Stdio};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::time::Instant;

pub struct SingBoxProcess {
    child: Option<Child>,
    started_at: Instant,
    killed: Arc<AtomicBool>,
}

impl SingBoxProcess {
    pub fn is_running(&self) -> bool {
        !self.killed.load(Ordering::SeqCst)
            && self
                .child
                .as_ref()
                .map(|c| c.try_wait().ok().flatten().is_none())
                .unwrap_or(false)
    }

    pub fn uptime_secs(&self) -> Option<u64> {
        if self.is_running() {
            Some(self.started_at.elapsed().as_secs())
        } else {
            None
        }
    }

    pub fn kill(mut self) -> Result<(), String> {
        self.killed.store(true, Ordering::SeqCst);
        if let Some(ref mut child) = self.child {
            child
                .kill()
                .map_err(|e| format!("failed to kill sing-box: {}", e))?;
            child.wait().ok();
        }
        Ok(())
    }
}

/// Start sing-box with the given config path.
/// Looks for sing-box.exe (Windows) or sing-box (Linux/Mac) next to the app.
pub fn start(config_path: &str) -> Result<SingBoxProcess, String> {
    let exe_dir = std::env::current_exe()
        .ok()
        .and_then(|p| p.parent().map(|p| p.to_path_buf()))
        .unwrap_or_else(|| std::path::PathBuf::from("."));

    let singbox_exe = if cfg!(target_os = "windows") {
        exe_dir.join("sing-box.exe")
    } else {
        exe_dir.join("sing-box")
    };

    if !singbox_exe.exists() {
        return Err(format!(
            "sing-box not found at: {}. Place sing-box binary next to the app.",
            singbox_exe.display()
        ));
    }

    let child = Command::new(&singbox_exe)
        .arg("run")
        .arg("-c")
        .arg(config_path)
        .stdout(Stdio::null())
        .stderr(Stdio::piped())
        .spawn()
        .map_err(|e| format!("failed to spawn sing-box: {}", e))?;

    let killed = Arc::new(AtomicBool::new(false));

    Ok(SingBoxProcess {
        child: Some(child),
        started_at: Instant::now(),
        killed,
    })
}
