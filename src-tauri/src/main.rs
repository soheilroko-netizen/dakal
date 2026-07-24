#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use std::path::PathBuf;
use std::process::{Child, Command, Stdio};
use std::sync::Arc;
use std::time::Instant;
#[cfg(target_os = "windows")]
use std::os::windows::process::CommandExt;
use tokio::net::TcpStream;
use tokio::sync::Mutex;
use tauri::{Emitter, Manager, State};

// ── State ──────────────────────────────────────────────

struct AppState {
    child: Arc<Mutex<Option<Child>>>,
    ping_target: String,
}

fn exe_dir() -> PathBuf {
    std::env::current_exe()
        .ok()
        .and_then(|p| p.parent().map(|d| d.to_path_buf()))
        .unwrap_or_else(|| PathBuf::from("."))
}

fn extract_ping_target() -> String {
    let path = exe_dir().join("config.json");
    let content = match std::fs::read_to_string(&path) {
        Ok(c) => c,
        Err(_) => return String::new(),
    };
    let json: serde_json::Value = match serde_json::from_str(&content) {
        Ok(j) => j,
        Err(_) => return String::new(),
    };
    let outbounds = match json.get("outbounds")?.as_array() {
        Some(a) => a,
        None => return String::new(),
    };
    let first = outbounds.first()?;
    let server = first.get("server")?.as_str()?;
    let port = first.get("server_port")?.as_u64()?;
    format!("{}:{}", server, port)
}

// ── Commands ───────────────────────────────────────────

#[tauri::command]
async fn connect(state: State<'_, AppState>) -> Result<(), String> {
    let mut guard = state.child.lock().await;
    if guard.is_some() {
        return Err("Already connected".into());
    }

    let dir = exe_dir();
    let singbox = dir.join("sing-box.exe");
    let config = dir.join("config.json");

    if !singbox.exists() {
        return Err("sing-box.exe not found in app folder".into());
    }
    if !config.exists() {
        return Err("config.json not found in app folder".into());
    }

    let child = Command::new(&singbox)
        .arg("run")
        .arg("-c")
        .arg(&config)
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .creation_flags(0x08000000) // CREATE_NO_WINDOW
        .spawn()
        .map_err(|e| format!("Failed to start sing-box: {e}"))?;

    *guard = Some(child);
    Ok(())
}

#[tauri::command]
async fn disconnect(state: State<'_, AppState>) -> Result<(), String> {
    let mut guard = state.child.lock().await;
    if let Some(mut child) = guard.take() {
        let _ = child.kill();
        let _ = child.wait();
    }
    Ok(())
}

#[tauri::command]
async fn get_status(state: State<'_, AppState>) -> Result<serde_json::Value, String> {
    let mut guard = state.child.lock().await;
    let running = if let Some(ref mut child) = *guard {
        match child.try_wait() {
            Ok(Some(_)) => { *guard = None; false }
            Ok(None) => true,
            Err(_) => { *guard = None; false }
        }
    } else {
        false
    };
    drop(guard);

    let ping = if running && !state.ping_target.is_empty() {
        let addr = state.ping_target.clone();
        let start = Instant::now();
        match tokio::time::timeout(
            std::time::Duration::from_secs(3),
            TcpStream::connect(&addr),
        )
        .await
        {
            Ok(Ok(_)) => start.elapsed().as_millis() as u64,
            _ => 0,
        }
    } else {
        0
    };

    Ok(serde_json::json!({
        "running": running,
        "ping": ping,
    }))
}

// ── Entry ──────────────────────────────────────────────

fn main() {
    let state = AppState {
        child: Arc::new(Mutex::new(None)),
        ping_target: extract_ping_target(),
    };

    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_shell::init())
        .plugin(tauri_plugin_fs::init())
        .manage(state)
        .invoke_handler(tauri::generate_handler![
            connect,
            disconnect,
            get_status,
        ])
        .setup(|_app| {
            // Tray icon
            use tauri::menu::{Menu, MenuItem, PredefinedMenuItem};
            use tauri::tray::{MouseButton, MouseButtonState, TrayIconBuilder, TrayIconEvent};

            let show = MenuItem::with_id(_app, "show", "Show", true, None::<&str>)?;
            let quit = MenuItem::with_id(_app, "quit", "Quit", true, None::<&str>)?;
            let tray_menu = Menu::with_items(
                _app,
                &[&show, &PredefinedMenuItem::separator(_app)?, &quit],
            )?;

            let _tray = TrayIconBuilder::new()
                .menu(&tray_menu)
                .on_menu_event(move |app, event| match event.id.as_ref() {
                    "show" => {
                        if let Some(w) = app.get_webview_window("main") {
                            let _ = w.show();
                            let _ = w.set_focus();
                        }
                    }
                    "quit" => app.exit(0),
                    _ => {}
                })
                .on_tray_icon_event(|tray, event| {
                    if let TrayIconEvent::Click {
                        button: MouseButton::Left,
                        button_state: MouseButtonState::Up,
                        ..
                    } = event
                    {
                        let app = tray.app_handle();
                        if let Some(w) = app.get_webview_window("main") {
                            let _ = w.show();
                            let _ = w.set_focus();
                        }
                    }
                })
                .build(_app)?;

            // Close → hide to tray
            let app_handle = _app.handle().clone();
            if let Some(window) = _app.get_webview_window("main") {
                window.on_window_event(move |event| {
                    if let tauri::WindowEvent::CloseRequested { api, .. } = event {
                        api.prevent_close();
                        if let Some(w) = app_handle.get_webview_window("main") {
                            let _ = w.hide();
                        }
                    }
                });
            }

            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("error while running dakal");
}
