use serde::{Deserialize, Serialize};
use std::path::PathBuf;

#[derive(Debug, Serialize, Deserialize)]
pub struct AppConfig {
    /// Ping target extracted from sing-box config
    pub ping_target: Option<String>,
    /// Paths
    config_dir: PathBuf,
    config_path: PathBuf,
}

impl AppConfig {
    pub fn load() -> Self {
        let exe_dir = std::env::current_exe()
            .ok()
            .and_then(|p| p.parent().map(|d| d.to_path_buf()))
            .unwrap_or_else(|| PathBuf::from("."));

        let config_path = exe_dir.join("config.json");

        let ping_target = Self::extract_ping_target(&config_path);

        Self {
            ping_target,
            config_dir: exe_dir,
            config_path,
        }
    }

    /// Read config.json and extract first outbound's server:port for ping
    fn extract_ping_target(path: &PathBuf) -> Option<String> {
        if !path.exists() {
            return None;
        }
        let content = std::fs::read_to_string(path).ok()?;
        let json: serde_json::Value = serde_json::from_str(&content).ok()?;
        let outbounds = json.get("outbounds")?.as_array()?;
        let ss = outbounds.first()?;
        let server = ss.get("server")?.as_str()?;
        let port = ss.get("server_port")?.as_u64()?;
        Some(format!("{}:{}", server, port))
    }

    pub fn config_path(&self) -> &std::path::Path {
        &self.config_path
    }

    pub fn config_exists(&self) -> bool {
        self.config_path.exists()
    }
}
