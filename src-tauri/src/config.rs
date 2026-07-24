use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct AppConfig {
    /// Path to user's config.json. If None, generates from defaults
    pub config_path: Option<String>,
    /// Server IP for ping test
    pub server_ip: String,
    /// Server port for ping test (TCP)
    pub server_port: u16,
    /// Embedded config (your working config)
    pub ss_server: String,
    pub ss_port: u16,
    pub ss_method: String,
    pub ss_password: String,
    pub stls_server: String,
    pub stls_port: u16,
    pub stls_password: String,
    pub stls_sni: String,
}

impl Default for AppConfig {
    fn default() -> Self {
        Self {
            config_path: None,
            server_ip: "187.127.83.147".into(),
            server_port: 8380,
            ss_server: "187.127.83.147".into(),
            ss_port: 8380,
            ss_method: "2022-blake3-chacha20-poly1305".into(),
            ss_password: "tE+3/qlN/orCZRVUutWouysZ8BQs4RWzq46WK6CDGG4=".into(),
            stls_server: "187.127.83.147".into(),
            stls_port: 8553,
            stls_password: "y2lachetore".into(),
            stls_sni: "dl.google.com".into(),
        }
    }
}

impl AppConfig {
    pub fn load_or_default() -> Self {
        // Try loading from disk next to exe
        if let Ok(exe) = std::env::current_exe() {
            if let Some(dir) = exe.parent() {
                let path = dir.join("configs").join("profile.json");
                if path.exists() {
                    if let Ok(content) = std::fs::read_to_string(&path) {
                        if let Ok(cfg) = serde_json::from_str::<AppConfig>(&content) {
                            return cfg;
                        }
                    }
                }
            }
        }
        Self::default()
    }

    pub fn get_ping_target(&self) -> Option<String> {
        Some(format!("{}:{}", self.server_ip, self.server_port))
    }

    /// Generate sing-box config JSON string from embedded params
    pub fn generate_config(&self) -> String {
        serde_json::to_string_pretty(&serde_json::json!({
            "log": {
                "level": "info",
                "timestamp": true
            },
            "dns": {
                "servers": [
                    {
                        "tag": "remote-doh",
                        "type": "https",
                        "server": "1.1.1.1",
                        "detour": "ss-out"
                    },
                    {
                        "tag": "google-doh",
                        "type": "https",
                        "server": "8.8.8.8",
                        "detour": "ss-out"
                    },
                    {
                        "tag": "local-dns",
                        "type": "udp",
                        "server": "192.168.1.1",
                        "detour": "direct"
                    }
                ],
                "final": "remote-doh",
                "strategy": "ipv4_only"
            },
            "inbounds": [
                {
                    "type": "tun",
                    "tag": "tun-in",
                    "address": ["172.19.0.1/30"],
                    "auto_route": true,
                    "strict_route": true,
                    "sniff": true,
                    "stack": "system"
                }
            ],
            "outbounds": [
                {
                    "type": "shadowsocks",
                    "tag": "ss-out",
                    "server": self.ss_server,
                    "server_port": self.ss_port,
                    "method": self.ss_method,
                    "password": self.ss_password,
                    "detour": "shadowtls-out",
                    "udp_over_tcp": { "enabled": true }
                },
                {
                    "type": "shadowtls",
                    "tag": "shadowtls-out",
                    "server": self.stls_server,
                    "server_port": self.stls_port,
                    "password": self.stls_password,
                    "version": 3,
                    "tls": {
                        "enabled": true,
                        "server_name": self.stls_sni,
                        "insecure": false
                    }
                },
                {
                    "type": "direct",
                    "tag": "direct"
                }
            ],
            "route": {
                "rules": [
                    {
                        "protocol": "dns",
                        "outbound": "direct"
                    },
                    {
                        "ip_cidr": ["187.127.83.147/32"],
                        "outbound": "direct"
                    }
                ],
                "final": "ss-out",
                "auto_detect_interface": true
            }
        }))
        .unwrap_or_default()
    }
}
