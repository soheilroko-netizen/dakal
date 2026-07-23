use serde::{Deserialize, Serialize};
use std::sync::Mutex;
use tauri::{
    menu::{MenuBuilder, MenuItemBuilder},
    tray::TrayIconBuilder,
    Manager,
};

mod config;
mod ping;
mod singbox;

struct AppState {
    singbox: Mutex<Option<singbox::SingBoxProcess>>,
    config_path: Mutex<Option<String>>,
}

#[derive(Serialize, Deserialize, Clone)]
struct StatusInfo {
    running: bool,
    ping_ms: Option<u64>,
    uptime_secs: Option<u64>,
    server_ip: String,
    server_port: u16,
}

#[tauri::command]
async fn start_vpn(
    state: tauri::State<'_, AppState>,
    app: tauri::AppHandle,
) -> Result<String, String> {
    let mut proc = state.singbox.lock().map_err(|e| e.to_string())?;
    if proc.is_some() && proc.as_ref().unwrap().is_running() {
        return Err("already running".into());
    }

    let cfg_path = state
        .config_path
        .lock()
        .map_err(|e| e.to_string())?
        .clone()
        .unwrap_or_else(|| "config.json".into());

    let child = singbox::start(&cfg_path).map_err(|e| format!("sing-box failed: {}", e))?;
    *proc = Some(child);

    app.tray_icon_by_id("main")
        .ok_or("no tray")?
        .set_icon_as_template(false)
        .map_err(|e| e.to_string())?;

    Ok("started".into())
}

#[tauri::command]
async fn stop_vpn(
    state: tauri::State<'_, AppState>,
    app: tauri::AppHandle,
) -> Result<String, String> {
    let mut proc = state.singbox.lock().map_err(|e| e.to_string())?;
    if let Some(p) = proc.take() {
        p.kill().map_err(|e| format!("kill failed: {}", e))?;
    }
    app.tray_icon_by_id("main")
        .ok_or("no tray")?
        .set_icon_as_template(true)
        .map_err(|e| e.to_string())?;
    Ok("stopped".into())
}

#[tauri::command]
async fn ping_server(
    config_path: tauri::State<'_, AppState>,
) -> Result<u64, String> {
    // Read config to get server IP
    let path = config_path
        .config_path
        .lock()
        .map_err(|e| e.to_string())?
        .clone()
        .unwrap_or_else(|| "config.json".into());

    let cfg = config::load(&path)?;
    let ip = cfg
        .outbounds
        .iter()
        .find(|o| o.r#type == "shadowsocks")
        .map(|o| o.server.clone())
        .unwrap_or_default();

    if ip.is_empty() {
        return Err("no server IP in config".into());
    }

    ping::tcp_ping(&ip, 2000)
        .await
        .map_err(|e| format!("ping failed: {}", e))
}

#[tauri::command]
async fn get_status(
    state: tauri::State<'_, AppState>,
) -> Result<StatusInfo, String> {
    let proc = state.singbox.lock().map_err(|e| e.to_string())?;
    let running = proc.as_ref().map(|p| p.is_running()).unwrap_or(false);
    let uptime = proc.as_ref().and_then(|p| p.uptime_secs());

    // Read server IP from config for display
    let path = state
        .config_path
        .lock()
        .map_err(|e| e.to_string())?
        .clone()
        .unwrap_or_else(|| "config.json".into());

    let cfg = config::load(&path).ok();
    let (server_ip, server_port) = cfg
        .as_ref()
        .and_then(|c| {
            c.outbounds
                .iter()
                .find(|o| o.r#type == "shadowtls")
                .map(|o| (o.server.clone(), o.server_port))
        })
        .unwrap_or_default();

    // Try ping
    let ping_ms = if running {
        ping::tcp_ping(&server_ip, 3000).await.ok()
    } else {
        None
    };

    Ok(StatusInfo {
        running,
        ping_ms,
        uptime_secs: uptime,
        server_ip,
        server_port,
    })
}

#[tauri::command]
async fn list_configs() -> Vec<String> {
    config::list_profiles()
}

#[tauri::command]
async fn load_profile(name: String, state: tauri::State<'_, AppState>) -> Result<String, String> {
    let path = config::profile_path(&name);
    config::load(&path)?;
    *state.config_path.lock().map_err(|e| e.to_string())? = Some(path.clone());
    Ok(path)
}

#[tauri::command]
async fn save_profile(name: String, content: String) -> Result<String, String> {
    config::save(&name, &content)
}

#[tauri::command]
async fn get_current_config(state: tauri::State<'_, AppState>) -> Result<config::DakalConfig, String> {
    let path = state
        .config_path
        .lock()
        .map_err(|e| e.to_string())?
        .clone()
        .unwrap_or_else(|| "config.json".into());
    config::load(&path)
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .manage(AppState {
            singbox: Mutex::new(None),
            config_path: Mutex::new(None),
        })
        .plugin(tauri_plugin_process::init())
        .plugin(tauri_plugin_shell::init())
        .setup(|app| {
            if cfg!(debug_assertions) {
                app.handle().plugin(
                    tauri_plugin_log::Builder::default()
                        .level(log::LevelFilter::Info)
                        .build(),
                )?;
            }

            // Build tray menu
            let show =
                MenuItemBuilder::with_id("show", "Show").build(app).unwrap();
            let quit =
                MenuItemBuilder::with_id("quit", "Quit").build(app).unwrap();
            let menu = MenuBuilder::new(app)
                .item(&show)
                .item(&quit)
                .build()
                .unwrap();

            let _tray = TrayIconBuilder::new()
                .icon(app.default_window_icon().unwrap().clone())
                .menu(&menu)
                .on_menu_event(|app, event| {
                    match event.id().as_ref() {
                        "show" => {
                            if let Some(w) = app.get_webview_window("main") {
                                let _ = w.show();
                                let _ = w.set_focus();
                            }
                        }
                        "quit" => {
                            // Kill singbox on quit
                            let state: tauri::State<AppState> = app.state();
                            if let Ok(mut proc) = state.singbox.lock() {
                                if let Some(p) = proc.take() {
                                    let _ = p.kill();
                                }
                            }
                            app.exit(0);
                        }
                        _ => {}
                    }
                })
                .build(app)?;

            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            start_vpn,
            stop_vpn,
            get_status,
            ping_server,
            list_configs,
            load_profile,
            save_profile,
            get_current_config,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
