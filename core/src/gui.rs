//! Standalone In-App Desktop GUI Window Launcher for Windows
//! Launches the dashboard as a dedicated, standalone application window
//! (with zero browser URL bars, tabs, or bookmarks) using the native Windows App host.

use std::path::{Path, PathBuf};
use std::process::{Child, Command};
use colored::*;

/// Candidate executable paths for the native Windows app window host
const WINDOWS_APP_HOSTS: &[&str] = &[
    // Microsoft Edge (Standard on 100% of Windows 10 / 11 / Server installations)
    r"C:\Program Files (x86)\Microsoft\Edge\Application\msedge.exe",
    r"C:\Program Files\Microsoft\Edge\Application\msedge.exe",
    // Google Chrome fallback
    r"C:\Program Files\Google\Chrome\Application\chrome.exe",
    r"C:\Program Files (x86)\Google\Chrome\Application\chrome.exe",
    // Brave Browser fallback
    r"C:\Program Files\BraveSoftware\Brave-Browser\Application\brave.exe",
];

/// Resolve the local user-data directory for the standalone GUI window profile
fn get_webview_data_dir() -> PathBuf {
    if let Ok(local_app_data) = std::env::var("LOCALAPPDATA") {
        PathBuf::from(local_app_data).join("NextcloudAIO").join("webview")
    } else {
        PathBuf::from("data").join("webview")
    }
}

/// Locate a suitable app-mode binary on Windows
fn find_app_host() -> Option<PathBuf> {
    for candidate in WINDOWS_APP_HOSTS {
        let p = Path::new(candidate);
        if p.exists() {
            return Some(p.to_path_buf());
        }
    }

    // Try finding via PATH
    for name in &["msedge.exe", "msedge", "chrome.exe", "chrome"] {
        if let Ok(output) = Command::new("where").arg(name).output() {
            if output.status.success() {
                let stdout = String::from_utf8_lossy(&output.stdout);
                if let Some(first_line) = stdout.lines().next() {
                    let p = PathBuf::from(first_line.trim());
                    if p.exists() {
                        return Some(p);
                    }
                }
            }
        }
    }

    None
}

/// Spawns the dedicated, standalone desktop application window
/// Returns the Child handle so the supervisor can track window lifecycle
pub fn spawn_in_app_gui(port: u16) -> anyhow::Result<Child> {
    let url = format!("http://127.0.0.1:{}", port);
    let user_data = get_webview_data_dir();
    let _ = std::fs::create_dir_all(&user_data);

    if let Some(host) = find_app_host() {
        println!(
            "{} Launching standalone desktop in-app GUI window: {}",
            "[GUI]".bold().cyan(),
            host.display()
        );

        let child = Command::new(&host)
            .arg(format!("--app={}", url))
            .arg(format!("--user-data-dir={}", user_data.display()))
            .arg("--window-size=1240,840")
            .arg("--app-id=NextcloudAIO")
            .arg("--disable-features=TranslateUI")
            .arg("--no-first-run")
            .arg("--no-default-browser-check")
            .spawn()?;

        return Ok(child);
    }

    // Secondary fallback: Windows default shell open
    println!(
        "{} App host binary not found at standard paths. Spawning default window...",
        "[WARN]".bold().yellow()
    );

    #[cfg(target_os = "windows")]
    {
        let child = Command::new("cmd")
            .args(["/c", "start", &url])
            .spawn()?;
        Ok(child)
    }

    #[cfg(not(target_os = "windows"))]
    {
        let child = Command::new("xdg-open")
            .arg(&url)
            .spawn()?;
        Ok(child)
    }
}
