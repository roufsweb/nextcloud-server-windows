use crate::config::AppConfig;
use crate::drive;
use crate::tunnel;
use anyhow::Result;
use colored::*;
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::Path;
use std::sync::Arc;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::TcpListener;
use tokio::sync::Mutex;

#[derive(Serialize)]
struct StatusResponse {
    running: bool,
    server_port: u16,
    fastcgi_port: u16,
    data_dir: String,
    tunnel_status: String,
    tunnel_url: String,
    drives: Vec<DriveInfo>,
    total_disk_gb: u64,
    used_disk_gb: u64,
}

#[derive(Serialize)]
struct DriveInfo {
    name: String,
    path: String,
    permission: String,
    status: String,
    total_gb: u64,
    used_gb: u64,
    free_gb: u64,
}

#[derive(Deserialize)]
#[allow(dead_code)]
struct MountDriveRequest {
    path: String,
    name: Option<String>,
}

#[derive(Deserialize)]
struct UnmountDriveRequest {
    path: String,
}

#[derive(Deserialize)]
struct TunnelTokenRequest {
    token: String,
}

#[derive(Deserialize)]
struct DataRootRequest {
    path: String,
}

#[derive(Deserialize)]
struct AdminUpdateRequest {
    username: String,
    password: Option<String>,
}

#[derive(Deserialize)]
struct PortChangeRequest {
    port: u16,
}

#[derive(Deserialize)]
struct DomainRequest {
    domain: String,
}

pub async fn run_dashboard_server(
    port: u16,
    config: Arc<Mutex<AppConfig>>,
    config_path: &'static str,
) -> Result<()> {
    let listener = TcpListener::bind(format!("127.0.0.1:{}", port)).await?;
    println!("{} Nextcloud AIO Dashboard running on: {}", "🚀".bold(), format!("http://localhost:{}", port).cyan());

    loop {
        let (mut socket, _) = listener.accept().await?;
        let config_clone = Arc::clone(&config);

        tokio::spawn(async move {
            let mut buffer = [0u8; 4096];
            let n = match socket.read(&mut buffer).await {
                Ok(n) if n > 0 => n,
                _ => return,
            };

            let req_str = String::from_utf8_lossy(&buffer[..n]);
            let lines: Vec<&str> = req_str.lines().collect();
            if lines.is_empty() { return; }

            let first_line: Vec<&str> = lines[0].split_whitespace().collect();
            if first_line.len() < 2 { return; }

            let method = first_line[0];
            let uri = first_line[1];

            // Extract body if POST
            let body = req_str.split("\r\n\r\n").nth(1).unwrap_or("");

            let (status_line, content_type, body_response) = if uri == "/api/status" {
                let cfg = config_clone.lock().await;
                let drives_info: Vec<DriveInfo> = cfg.external_drives.iter().map(|p| {
                    let (tot, used, free) = drive::get_disk_space(p);
                    DriveInfo {
                        name: p.file_name().map(|n| n.to_string_lossy().to_string()).unwrap_or_else(|| p.to_string_lossy().to_string()),
                        path: p.to_string_lossy().to_string(),
                        permission: "Full Control (NTFS)".to_string(),
                        status: "Active".to_string(),
                        total_gb: tot,
                        used_gb: used,
                        free_gb: free,
                    }
                }).collect();

                let (data_tot, data_used, _data_free) = drive::get_disk_space(&cfg.data_dir);

                let resp = StatusResponse {
                    running: true,
                    server_port: cfg.server_port,
                    fastcgi_port: cfg.fastcgi_port,
                    data_dir: cfg.data_dir.to_string_lossy().to_string(),
                    tunnel_status: if cfg.cloudflare_token.is_some() { "Connected".into() } else { "Offline".into() },
                    tunnel_url: "https://cloud.yourdomain.com".into(),
                    drives: drives_info,
                    total_disk_gb: data_tot,
                    used_disk_gb: data_used,
                };
                ("HTTP/1.1 200 OK", "application/json", serde_json::to_string(&resp).unwrap_or_default())
            } else if method == "POST" && uri == "/api/drives/mount" {
                if let Ok(req) = serde_json::from_str::<MountDriveRequest>(body) {
                    let drive_path = std::path::PathBuf::from(&req.path);
                    let _ = drive::grant_drive_permissions(&drive_path);
                    let mut cfg = config_clone.lock().await;
                    if !cfg.external_drives.contains(&drive_path) {
                        cfg.external_drives.push(drive_path);
                        let _ = cfg.save(config_path);
                    }
                    ("HTTP/1.1 200 OK", "application/json", "{\"status\":\"success\"}".into())
                } else {
                    ("HTTP/1.1 400 Bad Request", "application/json", "{\"error\":\"invalid request\"}".into())
                }
            } else if method == "POST" && uri == "/api/drives/unmount" {
                if let Ok(req) = serde_json::from_str::<UnmountDriveRequest>(body) {
                    let drive_path = std::path::PathBuf::from(&req.path);
                    let mut cfg = config_clone.lock().await;
                    cfg.external_drives.retain(|d| d != &drive_path);
                    let _ = cfg.save(config_path);
                    ("HTTP/1.1 200 OK", "application/json", "{\"status\":\"unmounted\"}".into())
                } else {
                    ("HTTP/1.1 400 Bad Request", "application/json", "{\"error\":\"invalid request\"}".into())
                }
            } else if method == "POST" && uri == "/api/tunnel/token" {
                if let Ok(req) = serde_json::from_str::<TunnelTokenRequest>(body) {
                    if tunnel::validate_token(&req.token).is_ok() {
                        let mut cfg = config_clone.lock().await;
                        cfg.cloudflare_token = Some(req.token.trim().to_string());
                        let _ = cfg.save(config_path);
                        ("HTTP/1.1 200 OK", "application/json", "{\"status\":\"token_saved\"}".into())
                    } else {
                        ("HTTP/1.1 400 Bad Request", "application/json", "{\"error\":\"invalid token format\"}".into())
                    }
                } else {
                    ("HTTP/1.1 400 Bad Request", "application/json", "{\"error\":\"invalid payload\"}".into())
                }
            } else if method == "POST" && uri == "/api/config/dataroot" {
                if let Ok(req) = serde_json::from_str::<DataRootRequest>(body) {
                    let path = std::path::PathBuf::from(&req.path);
                    let _ = drive::grant_drive_permissions(&path);
                    let mut cfg = config_clone.lock().await;
                    cfg.data_dir = path;
                    let _ = cfg.save(config_path);
                    ("HTTP/1.1 200 OK", "application/json", "{\"status\":\"data_root_updated\"}".into())
                } else {
                    ("HTTP/1.1 400 Bad Request", "application/json", "{\"error\":\"invalid payload\"}".into())
                }
            } else if method == "POST" && uri == "/api/config/port" {
                if let Ok(req) = serde_json::from_str::<PortChangeRequest>(body) {
                    let mut cfg = config_clone.lock().await;
                    cfg.server_port = req.port;
                    let _ = cfg.save(config_path);
                    ("HTTP/1.1 200 OK", "application/json", "{\"status\":\"port_updated\"}".into())
                } else {
                    ("HTTP/1.1 400 Bad Request", "application/json", "{\"error\":\"invalid port\"}".into())
                }
            } else if uri == "/api/config/domains" {
                let cfg = config_clone.lock().await;
                let domains = cfg.trusted_domains.clone();
                ("HTTP/1.1 200 OK", "application/json", serde_json::to_string(&domains).unwrap_or_else(|_| "[]".into()))
            } else if method == "POST" && uri == "/api/config/domains/add" {
                if let Ok(req) = serde_json::from_str::<DomainRequest>(body) {
                    let mut cfg = config_clone.lock().await;
                    let dom = req.domain.trim().to_lowercase();
                    if !dom.is_empty() && !cfg.trusted_domains.contains(&dom) {
                        cfg.trusted_domains.push(dom);
                        let _ = cfg.save(config_path);
                    }
                    ("HTTP/1.1 200 OK", "application/json", "{\"status\":\"domain_added\"}".into())
                } else {
                    ("HTTP/1.1 400 Bad Request", "application/json", "{\"error\":\"invalid domain\"}".into())
                }
            } else if method == "POST" && uri == "/api/config/domains/remove" {
                if let Ok(req) = serde_json::from_str::<DomainRequest>(body) {
                    let mut cfg = config_clone.lock().await;
                    let dom = req.domain.trim().to_lowercase();
                    cfg.trusted_domains.retain(|d| d != &dom);
                    let _ = cfg.save(config_path);
                    ("HTTP/1.1 200 OK", "application/json", "{\"status\":\"domain_removed\"}".into())
                } else {
                    ("HTTP/1.1 400 Bad Request", "application/json", "{\"error\":\"invalid domain\"}".into())
                }
            } else if uri == "/api/users/admin" {
                let cfg = config_clone.lock().await;
                let user = cfg.admin_username.clone().unwrap_or_else(|| "admin".to_string());
                let resp = serde_json::json!({
                    "username": user,
                    "status": "Active (Administrator)",
                    "auth_source": "Local Database"
                });
                ("HTTP/1.1 200 OK", "application/json", resp.to_string())
            } else if method == "POST" && uri == "/api/users/update-admin" {
                if let Ok(req) = serde_json::from_str::<AdminUpdateRequest>(body) {
                    let mut cfg = config_clone.lock().await;
                    let new_user = req.username.trim().to_string();
                    cfg.admin_username = Some(new_user.clone());
                    let _ = cfg.save(config_path);

                    // If a new password was provided, update autoconfig.php and attempt OCC password update
                    if let Some(pwd) = req.password {
                        if !pwd.trim().is_empty() {
                            let auto_cfg_path = "config/nextcloud/autoconfig.php";
                            if let Ok(content) = fs::read_to_string(auto_cfg_path) {
                                let updated = content
                                    .replace("'adminlogin'    => 'admin'", &format!("'adminlogin'    => '{}'", new_user))
                                    .replace("'adminpass'     => 'NextcloudAdmin2026!'", &format!("'adminpass'     => '{}'", pwd.trim()));
                                let _ = fs::write(auto_cfg_path, updated);
                            }

                            // Trigger occ password reset in background if installed
                            let php_path = cfg.php_path.clone();
                            let target_user = new_user.clone();
                            let pass_val = pwd.trim().to_string();
                            std::thread::spawn(move || {
                                let mut cmd = std::process::Command::new(php_path);
                                cmd.arg("nextcloud/occ").args(["user:resetpassword", "--password-from-env", &target_user]);
                                cmd.env("OC_PASS", pass_val);
                                let _ = cmd.output();
                            });
                        }
                    }

                    ("HTTP/1.1 200 OK", "application/json", "{\"status\":\"success\",\"message\":\"Administrator credentials updated\"}".into())
                } else {
                    ("HTTP/1.1 400 Bad Request", "application/json", "{\"error\":\"invalid payload\"}".into())
                }
            } else {
                // Static file serving from ui/
                let clean_uri = if uri == "/" { "/index.html" } else { uri.split('?').next().unwrap_or(uri) };
                let file_path = Path::new("ui").join(clean_uri.trim_start_matches('/'));

                if file_path.exists() && file_path.is_file() {
                    let mime = match file_path.extension().and_then(|s| s.to_str()).unwrap_or("") {
                        "html" => "text/html; charset=utf-8",
                        "css" => "text/css; charset=utf-8",
                        "js" => "application/javascript; charset=utf-8",
                        "svg" => "image/svg+xml",
                        "json" => "application/json",
                        _ => "text/plain",
                    };
                    let content = fs::read_to_string(&file_path).unwrap_or_default();
                    ("HTTP/1.1 200 OK", mime, content)
                } else {
                    ("HTTP/1.1 404 Not Found", "text/plain", "Not Found".to_string())
                }
            };

            let response = format!(
                "{}\r\nContent-Type: {}\r\nContent-Length: {}\r\nAccess-Control-Allow-Origin: *\r\nConnection: close\r\n\r\n{}",
                status_line, content_type, body_response.len(), body_response
            );

            let _ = socket.write_all(response.as_bytes()).await;
        });
    }
}
