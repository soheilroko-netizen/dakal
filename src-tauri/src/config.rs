use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;

#[derive(Deserialize, Serialize, Clone, Debug)]
pub struct DakalConfig {
    pub log: Option<LogConfig>,
    pub dns: Option<DnsConfig>,
    pub inbounds: Vec<Inbound>,
    pub outbounds: Vec<Outbound>,
    pub route: Option<RouteConfig>,
}

#[derive(Deserialize, Serialize, Clone, Debug)]
pub struct LogConfig {
    pub level: String,
    pub timestamp: bool,
}

#[derive(Deserialize, Serialize, Clone, Debug)]
pub struct DnsConfig {
    pub servers: Vec<DnsServer>,
    pub final: String,
}

#[derive(Deserialize, Serialize, Clone, Debug)]
pub struct DnsServer {
    pub tag: String,
    pub r#type: String,
    pub server: String,
    pub detour: Option<String>,
}

#[derive(Deserialize, Serialize, Clone, Debug)]
pub struct Inbound {
    pub r#type: String,
    pub tag: String,
    pub address: Vec<String>,
    pub auto_route: Option<bool>,
    pub strict_route: Option<bool>,
    pub stack: Option<String>,
}

#[derive(Deserialize, Serialize, Clone, Debug)]
pub struct Outbound {
    pub r#type: String,
    pub tag: String,
    pub server: String,
    pub server_port: u16,
    pub method: Option<String>,
    pub password: Option<String>,
    pub detour: Option<String>,
    pub version: Option<u8>,
    pub tls: Option<TlsConfig>,
    pub udp_over_tcp: Option<UdpOverTcp>,
}

#[derive(Deserialize, Serialize, Clone, Debug)]
pub struct TlsConfig {
    pub enabled: bool,
    pub server_name: Option<String>,
    pub insecure: Option<bool>,
}

#[derive(Deserialize, Serialize, Clone, Debug)]
pub struct UdpOverTcp {
    pub enabled: bool,
}

#[derive(Deserialize, Serialize, Clone, Debug)]
pub struct RouteConfig {
    pub rules: Vec<RouteRule>,
    pub final: String,
    pub auto_detect_interface: Option<bool>,
    pub default_domain_resolver: Option<String>,
}

#[derive(Deserialize, Serialize, Clone, Debug)]
pub struct RouteRule {
    pub action: Option<String>,
    pub protocol: Option<String>,
    pub ip_cidr: Option<Vec<String>>,
    pub outbound: Option<String>,
}

fn config_dir() -> PathBuf {
    let base = std::env::current_exe()
        .ok()
        .and_then(|p| p.parent().map(|p| p.to_path_buf()))
        .unwrap_or_else(|| PathBuf::from("."));
    base.join("profiles")
}

pub fn list_profiles() -> Vec<String> {
    let dir = config_dir();
    if !dir.exists() {
        return vec!["default".into()];
    }
    let mut names: Vec<String> = fs::read_dir(&dir)
        .ok()
        .into_iter()
        .flatten()
        .filter_map(|e| e.ok())
        .filter(|e| e.path().extension().map(|ext| ext == "json").unwrap_or(false))
        .filter_map(|e| e.path().file_stem().map(|s| s.to_string_lossy().into_owned()))
        .collect();
    names.sort();
    if !names.contains(&"default".into()) {
        names.insert(0, "default".into());
    }
    names
}

pub fn profile_path(name: &str) -> String {
    let dir = config_dir();
    if name == "default" {
        std::env::current_exe()
            .ok()
            .and_then(|p| p.parent().map(|p| p.join("config.json")))
            .unwrap_or_else(|| PathBuf::from("config.json"))
            .to_string_lossy()
            .into_owned()
    } else {
        config_dir()
            .join(format!("{}.json", name))
            .to_string_lossy()
            .into_owned()
    }
}

pub fn load(path: &str) -> Result<DakalConfig, String> {
    let text = fs::read_to_string(path).map_err(|e| format!("can't read {}: {}", path, e))?;
    serde_json::from_str(&text).map_err(|e| format!("invalid JSON: {}", e))
}

pub fn save(name: &str, content: &str) -> Result<String, String> {
    let dir = config_dir();
    fs::create_dir_all(&dir).map_err(|e| format!("can't create profiles dir: {}", e))?;
    let path = dir.join(format!("{}.json", name));
    fs::write(&path, content).map_err(|e| format!("can't write: {}", e))?;
    Ok(path.to_string_lossy().into_owned())
}
