//! Standalone Desktop GUI Application Entry Point
//! Windows Subsystem: "windows" (No console / CMD window will appear!)

#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod config;
mod drive;
mod gui;
mod server;
mod supervisor;
mod tunnel;

use config::AppConfig;
use std::sync::Arc;
use tokio::sync::Mutex;

const CONFIG_PATH: &str = "config/app_config.json";
const DEFAULT_PORT: u16 = 8081;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Load application configuration
    let config = AppConfig::load_or_default(CONFIG_PATH);
    let shared_config = Arc::new(Mutex::new(config.clone()));

    // 1. Spawn internal dashboard HTTP server
    let srv_cfg = Arc::clone(&shared_config);
    tokio::spawn(async move {
        if let Err(e) = server::run_dashboard_server(DEFAULT_PORT, srv_cfg, CONFIG_PATH).await {
            eprintln!("Dashboard server error: {}", e);
        }
    });

    // 2. Start supervisor services (Caddy, PHP-CGI, Cloudflared) in background thread
    let sup_config = config.clone();
    std::thread::spawn(move || {
        let sup = supervisor::ServerSupervisor::new(sup_config);
        let _ = sup.start();
    });

    // 3. Allow server 600ms to bind sockets
    tokio::time::sleep(tokio::time::Duration::from_millis(600)).await;

    // 4. Launch standalone in-app desktop GUI application window
    if let Ok(mut gui_process) = gui::spawn_in_app_gui(DEFAULT_PORT) {
        // Wait for the desktop application window to exit
        let _ = gui_process.wait();
    }

    Ok(())
}
