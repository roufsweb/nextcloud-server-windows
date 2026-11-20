//! Standalone Desktop GUI Application Entry Point
//! Windows Subsystem: "windows" (No console / CMD window will appear!)

#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod autostart;
mod config;
mod db;
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

    // Shared supervisor instance for process life-cycle management
    let supervisor = Arc::new(std::sync::Mutex::new(supervisor::ServerSupervisor::new(config.clone())));

    // 1. Pre-spawn background server services (PHP-CGI, Caddy, Cloudflared) silently
    {
        if let Ok(mut sup) = supervisor.lock() {
            let _ = sup.spawn_services();
        }
    }

    // 2. Spawn internal dashboard HTTP server with supervisor reference
    let srv_cfg = Arc::clone(&shared_config);
    let srv_sup = Arc::clone(&supervisor);
    tokio::spawn(async move {
        if let Err(e) = server::run_dashboard_server(DEFAULT_PORT, srv_cfg, srv_sup, CONFIG_PATH).await {
            eprintln!("Dashboard server error: {}", e);
        }
    });

    // 3. Allow server 500ms to bind sockets
    tokio::time::sleep(tokio::time::Duration::from_millis(500)).await;

    // 4. Launch standalone in-app desktop GUI application window
    if let Ok(mut gui_process) = gui::spawn_in_app_gui(DEFAULT_PORT) {
        // Wait for the desktop application window to exit (user closes window)
        let _ = gui_process.wait();
    }

    // 5. Desktop window closed: cleanly terminate all background child processes
    if let Ok(mut sup) = supervisor.lock() {
        sup.stop_services();
    };

    Ok(())
}
