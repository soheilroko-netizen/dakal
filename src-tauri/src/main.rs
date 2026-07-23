#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod config;
mod ping;
mod process;

use std::sync::Arc;
use tokio::sync::Mutex;
use tauri::{Emitter, Manager, State};

use config::AppConfig;
use process::SingBoxProcess;

struct AppState {
    proc: Arc<Mutex<SingBoxProcess>>,
    config: Arc<Mutex<AppConfig>>,
}

// ── Tauri Commands ──────────────────────────────────────

#[tauri::command]
async fn start_vpn(state: State<'_, AppState>, app: tauri::AppHandle) -> Result<(), String> {
    let mut proc = state.proc.lock().await;
    if proc.is_running() {
        return Err("Already running".into());
    }

    let cfg = state.config.lock().await.clone();
    proc.start(&cfg, &app).await.map_err(|e| e.to_string())?;

    // Emit connected status
    let _ = app.emit("vpn-status", serde_json::json!({"running": true}));
    Ok(())
}

#[tauri::command]
async fn stop_vpn(state: State<'_, AppState>, app: tauri::AppHandle) -> Result<(), String> {
    let mut proc = state.proc.lock().await;
    proc.stop().await;
    let _ = app.emit("vpn-status", serde_json::json!({"running": false}));
    Ok(())
}

#[tauri::command]
async fn get_status(state: State<'_, AppState>) -> Result<serde_json::Value, String> {
    let proc = state.proc.lock().await;
    let running = proc.is_running();
    let ping = if running {
        let now = std::time::Instant::now();
        // Quick TCP check to server
        match state.config.lock().await.get_ping_target() {
            Some(addr) => {
                let r = tokio::time::timeout(
                    std::time::Duration::from_secs(3),
                    tokio::net::TcpStream::connect(&addr),
                )
                .await;
                match r {
                    Ok(Ok(_)) => now.elapsed().as_millis() as u64,
                    _ => 0,
                }
            }
            None => 0,
        }
    } else {
        0
    };

    Ok(serde_json::json!({
        "running": running,
        "ping": ping,
    }))
}

// ── App Entry ───────────────────────────────────────────

fn main() {
    // Install panic logger for debug
    std::panic::set_hook(Box::new(|info| {
        let msg = format!("PANIC: {info}");
        let _ = std::fs::write("dakal_crash.log", &msg);
        eprintln!("{msg}");
    }));

    let state = AppState {
        proc: Arc::new(Mutex::new(SingBoxProcess::new())),
        config: Arc::new(Mutex::new(AppConfig::load_or_default())),
    };

    tauri::Builder::default()
        .plugin(tauri_plugin_store::Builder::default().build())
        .plugin(tauri_plugin_log::Builder::new().build())
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_shell::init())
        .plugin(tauri_plugin_fs::init())
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_notification::init())
        .plugin(tauri_plugin_os::init())
        .manage(state)
        .invoke_handler(tauri::generate_handler![
            start_vpn,
            stop_vpn,
            get_status,
        ])
        .setup(move |_app| {
            // Create tray
            use tauri::menu::{Menu, MenuItem, PredefinedMenuItem};
            use tauri::tray::{MouseButton, MouseButtonState, TrayIconBuilder, TrayIconEvent};

            let show = MenuItem::with_id(_app, "show", "Show", true, None::<&str>)?;
            let quit = MenuItem::with_id(_app, "quit", "Quit", true, None::<&str>)?;
            let tray_menu = Menu::with_items(_app, &[&show, &PredefinedMenuItem::separator(_app)?, &quit])?;

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
            if let Some(window) = _app.get_webview_window("main") {
                window.on_window_event(move |event| {
                    if let tauri::WindowEvent::CloseRequested { api, .. } = event {
                        api.prevent_close();
                        let _ = window.hide();
                    }
                });
            }

            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("error while running dakal");
}
