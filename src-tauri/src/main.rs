#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use tauri::Manager;

#[tauri::command]
fn start_vpn() -> Result<(), String> {
    #[cfg(windows)]
    {
        use std::process::Command;
        Command::new("cmd")
            .args(["/C", "start.bat"])
            .spawn()
            .map_err(|e| e.to_string())?;
    }
    Ok(())
}

#[tauri::command]
fn stop_vpn() -> Result<(), String> {
    #[cfg(windows)]
    {
        use std::process::Command;
        Command::new("taskkill")
            .args(["/F", "/IM", "sing-box.exe"])
            .output()
            .map_err(|e| e.to_string())?;
    }
    Ok(())
}

fn main() {
    tauri::Builder::default()
        .invoke_handler(tauri::generate_handler![start_vpn, stop_vpn])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}