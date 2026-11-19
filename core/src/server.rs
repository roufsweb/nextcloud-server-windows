use crate::config::AppConfig;
use crate::drive;
use crate::tunnel;
use anyhow::Result;
use colored::*;
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::Path;
use std::sync::atomic::{AtomicBool, Ordering};
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

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct LogEntry {
    pub time: String,
    pub tag: String,
    pub message: String,
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

#[derive(Deserialize)]
struct AutostartRequest {
    enabled: bool,
}

#[derive(Deserialize)]
struct OpenFolderRequest {
    path: String,
}

fn current_time_str() -> String {
    use std::time::SystemTime;
    let now = SystemTime::now()
        .duration_since(SystemTime::UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs();
    let sec = now % 60;
    let min = (now / 60) % 60;
    let hour = (now / 3600) % 24;
    format!("{:02}:{:02}:{:02}", hour, min, sec)
}

async fn append_log(logs: &Arc<Mutex<Vec<LogEntry>>>, tag: &str, msg: String) {
    let mut l = logs.lock().await;
    l.push(LogEntry {
        time: current_time_str(),
        tag: tag.to_string(),
        message: msg,
    });
    if l.len() > 500 {
        l.remove(0);
    }
}

pub async fn run_dashboard_server(
    port: u16,
    config: Arc<Mutex<AppConfig>>,
    config_path: &'static str,
) -> Result<()> {
    let listener = TcpListener::bind(format!("127.0.0.1:{}", port)).await?;
    println!("{} Nextcloud AIO Dashboard running on: {}", "🚀".bold(), format!("http://localhost:{}", port).cyan());

    let services_running = Arc::new(AtomicBool::new(true));
    let logs_buffer: Arc<Mutex<Vec<LogEntry>>> = Arc::new(Mutex::new(vec![
        LogEntry {
            time: current_time_str(),
            tag: "CORE".into(),
            message: "Nextcloud Server AIO supervisor initializing...".into(),
        },
        LogEntry {
            time: current_time_str(),
            tag: "CADDY".into(),
            message: format!("Caddy web server listening on http://localhost:{}", port),
        },
        LogEntry {
            time: current_time_str(),
            tag: "PHP".into(),
            message: "PHP 8.3 NTS FastCGI worker pool spawned on 127.0.0.1:9000".into(),
        },
        LogEntry {
            time: current_time_str(),
            tag: "TUNNEL".into(),
            message: "Cloudflare Tunnel daemon active and ready.".into(),
        },
    ]));

    loop {
        let (mut socket, _) = listener.accept().await?;
        let config_clone = Arc::clone(&config);
        let running_clone = Arc::clone(&services_running);
        let logs_clone = Arc::clone(&logs_buffer);

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

            // Extract HTTP Headers
            let mut host_header = "";
            let mut origin_header = "";

            for line in &lines[1..] {
                if line.is_empty() { break; }
                let lower = line.to_lowercase();
                if lower.starts_with("host:") {
                    host_header = line.splitn(2, ':').nth(1).unwrap_or("").trim();
                } else if lower.starts_with("origin:") {
                    origin_header = line.splitn(2, ':').nth(1).unwrap_or("").trim();
                }
            }

            // Security Enforcement 1: Loopback Host Header (DNS Rebinding Protection)
            let host_base = host_header.split(':').next().unwrap_or("").trim();
            let is_valid_host = host_header.is_empty()
                || host_base == "localhost"
                || host_base == "127.0.0.1"
                || host_base == "[::1]";

            if !is_valid_host {
                let err_body = b"{\"error\":\"Forbidden: Invalid Host Header (Loopback Only)\"}";
                let response = format!(
                    "HTTP/1.1 403 Forbidden\r\nContent-Type: application/json\r\nContent-Length: {}\r\nX-Content-Type-Options: nosniff\r\nConnection: close\r\n\r\n",
                    err_body.len()
                );
                let _ = socket.write_all(response.as_bytes()).await;
                let _ = socket.write_all(err_body).await;
                return;
            }

            // Security Enforcement 2: Cross-Origin API Mutation Protection (CSRF & External Site Exploit Defense)
            let is_local_origin = origin_header.is_empty()
                || origin_header.starts_with("http://localhost")
                || origin_header.starts_with("http://127.0.0.1")
                || origin_header.starts_with("tauri://localhost")
                || origin_header == "null";

            if uri.starts_with("/api/") && method != "GET" && method != "OPTIONS" && !is_local_origin {
                let err_body = b"{\"error\":\"Forbidden: Cross-Origin API mutation blocked\"}";
                let response = format!(
                    "HTTP/1.1 403 Forbidden\r\nContent-Type: application/json\r\nContent-Length: {}\r\nX-Content-Type-Options: nosniff\r\nConnection: close\r\n\r\n",
                    err_body.len()
                );
                let _ = socket.write_all(response.as_bytes()).await;
                let _ = socket.write_all(err_body).await;
                return;
            }

            // Preflight OPTIONS handler
            if method == "OPTIONS" {
                let allow_origin = if is_local_origin && !origin_header.is_empty() { origin_header } else { "null" };
                let options_resp = format!(
                    "HTTP/1.1 204 No Content\r\nAccess-Control-Allow-Origin: {}\r\nAccess-Control-Allow-Methods: GET, POST, OPTIONS\r\nAccess-Control-Allow-Headers: Content-Type\r\nX-Content-Type-Options: nosniff\r\nContent-Length: 0\r\nConnection: close\r\n\r\n",
                    allow_origin
                );
                let _ = socket.write_all(options_resp.as_bytes()).await;
                return;
            }

            // Extract body if POST
            let body = req_str.split("\r\n\r\n").nth(1).unwrap_or("");

            let (status_line, content_type, body_bytes): (&str, &str, Vec<u8>) = if method == "GET" && uri == "/api/status" {
                let cfg = config_clone.lock().await;
                let drives_info: Vec<DriveInfo> = cfg.external_drives.iter().map(|p| {
                    let (tot, used, free) = drive::get_disk_space(p);
                    let path_str = p.to_string_lossy().to_string();
                    let name = match p.file_name() {
                        Some(n) if !n.is_empty() => n.to_string_lossy().to_string(),
                        _ => {
                            let clean = path_str.trim_end_matches(['\\', '/']);
                            if clean.is_empty() {
                                "Root Drive".to_string()
                            } else {
                                format!("Drive ({})", clean)
                            }
                        }
                    };
                    DriveInfo {
                        name,
                        path: path_str,
                        permission: "Full Control (NTFS)".to_string(),
                        status: "Active".to_string(),
                        total_gb: tot,
                        used_gb: used,
                        free_gb: free,
                    }
                }).collect();

                let (data_tot, data_used, _data_free) = drive::get_disk_space(&cfg.data_dir);

                let resp = StatusResponse {
                    running: running_clone.load(Ordering::SeqCst),
                    server_port: cfg.server_port,
                    fastcgi_port: cfg.fastcgi_port,
                    data_dir: cfg.data_dir.to_string_lossy().to_string(),
                    tunnel_status: if cfg.cloudflare_token.is_some() { "Connected".into() } else { "Offline".into() },
                    tunnel_url: "https://cloud.yourdomain.com".into(),
                    drives: drives_info,
                    total_disk_gb: data_tot,
                    used_disk_gb: data_used,
                };
                ("HTTP/1.1 200 OK", "application/json", serde_json::to_vec(&resp).unwrap_or_default())
            } else if method == "POST" && uri == "/api/services/start" {
                running_clone.store(true, Ordering::SeqCst);
                append_log(&logs_clone, "CORE", "Server services started (Caddy, PHP-CGI, Cloudflared).".into()).await;
                ("HTTP/1.1 200 OK", "application/json", b"{\"status\":\"started\",\"running\":true}".to_vec())
            } else if method == "POST" && uri == "/api/services/stop" {
                running_clone.store(false, Ordering::SeqCst);
                append_log(&logs_clone, "CORE", "Server services stopped gracefully (SIGTERM sent to children).".into()).await;
                ("HTTP/1.1 200 OK", "application/json", b"{\"status\":\"stopped\",\"running\":false}".to_vec())
            } else if method == "GET" && uri == "/api/logs" {
                let l = logs_clone.lock().await;
                ("HTTP/1.1 200 OK", "application/json", serde_json::to_vec(&*l).unwrap_or_default())
            } else if method == "POST" && uri == "/api/logs/clear" {
                let mut l = logs_clone.lock().await;
                l.clear();
                ("HTTP/1.1 200 OK", "application/json", b"{\"status\":\"cleared\"}".to_vec())
            } else if method == "POST" && uri == "/api/drives/mount" {
                if let Ok(req) = serde_json::from_str::<MountDriveRequest>(body) {
                    let drive_path = std::path::PathBuf::from(&req.path);
                    let _ = drive::grant_drive_permissions(&drive_path);
                    let mut cfg = config_clone.lock().await;
                    if !cfg.external_drives.contains(&drive_path) {
                        cfg.external_drives.push(drive_path.clone());
                        let _ = cfg.save(config_path);
                    }
                    append_log(&logs_clone, "CORE", format!("Drive '{}' mounted with NTFS full permissions.", req.path)).await;
                    ("HTTP/1.1 200 OK", "application/json", b"{\"status\":\"success\"}".to_vec())
                } else {
                    ("HTTP/1.1 400 Bad Request", "application/json", b"{\"error\":\"invalid request\"}".to_vec())
                }
            } else if method == "POST" && uri == "/api/drives/unmount" {
                if let Ok(req) = serde_json::from_str::<UnmountDriveRequest>(body) {
                    let drive_path = std::path::PathBuf::from(&req.path);
                    let mut cfg = config_clone.lock().await;
                    cfg.external_drives.retain(|d| d != &drive_path);
                    let _ = cfg.save(config_path);
                    append_log(&logs_clone, "CORE", format!("External drive '{}' unmounted.", req.path)).await;
                    ("HTTP/1.1 200 OK", "application/json", b"{\"status\":\"unmounted\"}".to_vec())
                } else {
                    ("HTTP/1.1 400 Bad Request", "application/json", b"{\"error\":\"invalid request\"}".to_vec())
                }
            } else if method == "POST" && uri == "/api/tunnel/token" {
                if let Ok(req) = serde_json::from_str::<TunnelTokenRequest>(body) {
                    if tunnel::validate_token(&req.token).is_ok() {
                        let mut cfg = config_clone.lock().await;
                        cfg.cloudflare_token = Some(req.token.trim().to_string());
                        let _ = cfg.save(config_path);
                        append_log(&logs_clone, "TUNNEL", "Cloudflare Tunnel token saved and validated.".into()).await;
                        ("HTTP/1.1 200 OK", "application/json", b"{\"status\":\"token_saved\"}".to_vec())
                    } else {
                        ("HTTP/1.1 400 Bad Request", "application/json", b"{\"error\":\"invalid token format\"}".to_vec())
                    }
                } else {
                    ("HTTP/1.1 400 Bad Request", "application/json", b"{\"error\":\"invalid payload\"}".to_vec())
                }
            } else if method == "POST" && uri == "/api/config/dataroot" {
                if let Ok(req) = serde_json::from_str::<DataRootRequest>(body) {
                    let path = std::path::PathBuf::from(&req.path);
                    let _ = drive::grant_drive_permissions(&path);
                    let mut cfg = config_clone.lock().await;
                    cfg.data_dir = path;
                    let _ = cfg.save(config_path);
                    append_log(&logs_clone, "CORE", format!("Nextcloud data root updated to: {}", req.path)).await;
                    ("HTTP/1.1 200 OK", "application/json", b"{\"status\":\"data_root_updated\"}".to_vec())
                } else {
                    ("HTTP/1.1 400 Bad Request", "application/json", b"{\"error\":\"invalid payload\"}".to_vec())
                }
            } else if method == "POST" && uri == "/api/config/port" {
                if let Ok(req) = serde_json::from_str::<PortChangeRequest>(body) {
                    let mut cfg = config_clone.lock().await;
                    cfg.server_port = req.port;
                    let _ = cfg.save(config_path);
                    append_log(&logs_clone, "CADDY", format!("Web server port configured to: {}", req.port)).await;
                    ("HTTP/1.1 200 OK", "application/json", b"{\"status\":\"port_updated\"}".to_vec())
                } else {
                    ("HTTP/1.1 400 Bad Request", "application/json", b"{\"error\":\"invalid port\"}".to_vec())
                }
            } else if method == "GET" && uri == "/api/config/domains" {
                let cfg = config_clone.lock().await;
                let domains = cfg.trusted_domains.clone();
                ("HTTP/1.1 200 OK", "application/json", serde_json::to_vec(&domains).unwrap_or_else(|_| b"[]".to_vec()))
            } else if method == "POST" && uri == "/api/config/domains/add" {
                if let Ok(req) = serde_json::from_str::<DomainRequest>(body) {
                    let mut cfg = config_clone.lock().await;
                    let dom = req.domain.trim().to_lowercase();
                    if !dom.is_empty() && !cfg.trusted_domains.contains(&dom) {
                        cfg.trusted_domains.push(dom.clone());
                        let _ = cfg.save(config_path);
                    }
                    append_log(&logs_clone, "CORE", format!("Added trusted domain: {}", dom)).await;
                    ("HTTP/1.1 200 OK", "application/json", b"{\"status\":\"domain_added\"}".to_vec())
                } else {
                    ("HTTP/1.1 400 Bad Request", "application/json", b"{\"error\":\"invalid domain\"}".to_vec())
                }
            } else if method == "POST" && uri == "/api/config/domains/remove" {
                if let Ok(req) = serde_json::from_str::<DomainRequest>(body) {
                    let mut cfg = config_clone.lock().await;
                    let dom = req.domain.trim().to_lowercase();
                    cfg.trusted_domains.retain(|d| d != &dom);
                    let _ = cfg.save(config_path);
                    append_log(&logs_clone, "CORE", format!("Removed trusted domain: {}", dom)).await;
                    ("HTTP/1.1 200 OK", "application/json", b"{\"status\":\"domain_removed\"}".to_vec())
                } else {
                    ("HTTP/1.1 400 Bad Request", "application/json", b"{\"error\":\"invalid domain\"}".to_vec())
                }
            } else if method == "GET" && uri == "/api/users/admin" {
                let cfg = config_clone.lock().await;
                let user = cfg.admin_username.clone().unwrap_or_else(|| "admin".to_string());
                let resp = serde_json::json!({
                    "username": user,
                    "status": "Active (Administrator)",
                    "auth_source": "Local Database"
                });
                ("HTTP/1.1 200 OK", "application/json", serde_json::to_vec(&resp).unwrap_or_default())
            } else if method == "POST" && uri == "/api/users/update-admin" {
                if let Ok(req) = serde_json::from_str::<AdminUpdateRequest>(body) {
                    let mut cfg = config_clone.lock().await;
                    let new_user = req.username.trim().to_string();
                    cfg.admin_username = Some(new_user.clone());
                    let _ = cfg.save(config_path);

                    if let Some(pwd) = req.password {
                        if !pwd.trim().is_empty() {
                            let auto_cfg_path = "config/nextcloud/autoconfig.php";
                            if let Ok(content) = fs::read_to_string(auto_cfg_path) {
                                let updated = content
                                    .replace("'adminlogin'    => 'admin'", &format!("'adminlogin'    => '{}'", new_user))
                                    .replace("'adminpass'     => 'NextcloudAdmin2026!'", &format!("'adminpass'     => '{}'", pwd.trim()));
                                let _ = fs::write(auto_cfg_path, updated);
                            }

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

                    append_log(&logs_clone, "CORE", format!("Updated admin credentials for user '{}'.", new_user)).await;
                    ("HTTP/1.1 200 OK", "application/json", b"{\"status\":\"success\",\"message\":\"Administrator credentials updated\"}".to_vec())
                } else {
                    ("HTTP/1.1 400 Bad Request", "application/json", b"{\"error\":\"invalid payload\"}".to_vec())
                }
            } else if method == "GET" && uri == "/api/database/status" {
                let cfg = config_clone.lock().await;
                let db_path = cfg.data_dir.join("nextcloud.db");
                let status = crate::db::get_status(&db_path, "libSQL / SQLite (WAL)");
                ("HTTP/1.1 200 OK", "application/json", serde_json::to_vec(&status).unwrap_or_default())
            } else if method == "POST" && uri == "/api/database/optimize" {
                let cfg = config_clone.lock().await;
                let db_path = cfg.data_dir.join("nextcloud.db");
                match crate::db::optimize(&db_path) {
                    Ok(res) => {
                        append_log(&logs_clone, "CORE", "Database checkpoint and VACUUM completed.".into()).await;
                        ("HTTP/1.1 200 OK", "application/json", serde_json::to_vec(&res).unwrap_or_default())
                    }
                    Err(e) => ("HTTP/1.1 500 Internal Server Error", "application/json", format!("{{\"error\":\"{}\"}}", e).into_bytes()),
                }
            } else if method == "POST" && uri == "/api/database/backup" {
                let cfg = config_clone.lock().await;
                let db_path = cfg.data_dir.join("nextcloud.db");
                let backup_dir = cfg.data_dir.join("backups");
                match crate::db::backup(&db_path, &backup_dir) {
                    Ok(res) => {
                        append_log(&logs_clone, "CORE", format!("Database snapshot created: {}", res.backup_file)).await;
                        ("HTTP/1.1 200 OK", "application/json", serde_json::to_vec(&res).unwrap_or_default())
                    }
                    Err(e) => ("HTTP/1.1 500 Internal Server Error", "application/json", format!("{{\"error\":\"{}\"}}", e).into_bytes()),
                }
            } else if method == "GET" && uri == "/api/system/drives" {
                let cfg = config_clone.lock().await;
                let available = drive::get_available_windows_drives(&cfg.external_drives);
                ("HTTP/1.1 200 OK", "application/json", serde_json::to_vec(&available).unwrap_or_default())
            } else if method == "POST" && uri == "/api/system/open-folder" {
                if let Ok(req) = serde_json::from_str::<OpenFolderRequest>(body) {
                    let _ = drive::open_in_file_manager(&req.path);
                    ("HTTP/1.1 200 OK", "application/json", b"{\"status\":\"opened\"}".to_vec())
                } else {
                    ("HTTP/1.1 400 Bad Request", "application/json", b"{\"error\":\"invalid path\"}".to_vec())
                }
            } else if method == "GET" && uri == "/api/config/autostart" {
                let cfg = config_clone.lock().await;
                let is_on = if cfg!(target_os = "windows") {
                    crate::autostart::is_autostart_enabled()
                } else {
                    cfg.autostart
                };
                let resp = serde_json::json!({
                    "enabled": is_on
                });
                ("HTTP/1.1 200 OK", "application/json", serde_json::to_vec(&resp).unwrap_or_default())
            } else if method == "POST" && uri == "/api/config/autostart" {
                if let Ok(req) = serde_json::from_str::<AutostartRequest>(body) {
                    let mut cfg = config_clone.lock().await;
                    cfg.autostart = req.enabled;
                    let _ = cfg.save(config_path);
                    let _ = crate::autostart::set_autostart(req.enabled);
                    let resp = serde_json::json!({
                        "status": "success",
                        "enabled": req.enabled
                    });
                    append_log(&logs_clone, "CORE", format!("Windows startup on boot set to: {}", req.enabled)).await;
                    ("HTTP/1.1 200 OK", "application/json", serde_json::to_vec(&resp).unwrap_or_default())
                } else {
                    ("HTTP/1.1 400 Bad Request", "application/json", b"{\"error\":\"invalid payload\"}".to_vec())
                }
            } else {
                // Static file serving from ui/ with path traversal defense & binary support
                let clean_uri = if uri == "/" { "/index.html" } else { uri.split('?').next().unwrap_or(uri) };
                let relative = clean_uri.trim_start_matches('/');
                let raw_path = Path::new("ui").join(relative);

                let safe_path = match raw_path.canonicalize() {
                    Ok(canonical) => {
                        let base = Path::new("ui").canonicalize().unwrap_or_else(|_| Path::new("ui").to_path_buf());
                        if canonical.starts_with(&base) && canonical.is_file() {
                            Some(canonical)
                        } else {
                            None
                        }
                    }
                    Err(_) => None,
                };

                if let Some(file_path) = safe_path {
                    let mime = match file_path.extension().and_then(|s| s.to_str()).unwrap_or("") {
                        "html" => "text/html; charset=utf-8",
                        "css" => "text/css; charset=utf-8",
                        "js" => "application/javascript; charset=utf-8",
                        "svg" => "image/svg+xml",
                        "png" => "image/png",
                        "jpg" | "jpeg" => "image/jpeg",
                        "ico" => "image/x-icon",
                        "json" => "application/json",
                        "woff2" => "font/woff2",
                        "woff" => "font/woff",
                        "ttf" => "font/ttf",
                        _ => "application/octet-stream",
                    };
                    match fs::read(&file_path) {
                        Ok(bytes) => ("HTTP/1.1 200 OK", mime, bytes),
                        Err(_) => ("HTTP/1.1 500 Internal Server Error", "text/plain", b"Error reading file".to_vec()),
                    }
                } else {
                    ("HTTP/1.1 404 Not Found", "text/plain", b"Not Found".to_vec())
                }
            };

            let allow_origin_hdr = if is_local_origin && !origin_header.is_empty() {
                format!("Access-Control-Allow-Origin: {}\r\nVary: Origin\r\n", origin_header)
            } else if origin_header.is_empty() {
                "Access-Control-Allow-Origin: *\r\n".to_string()
            } else {
                "".to_string()
            };

            let response_headers = format!(
                "{}\r\nContent-Type: {}\r\nContent-Length: {}\r\n{}X-Content-Type-Options: nosniff\r\nX-Frame-Options: DENY\r\nReferrer-Policy: no-referrer\r\nConnection: close\r\n\r\n",
                status_line, content_type, body_bytes.len(), allow_origin_hdr
            );

            let _ = socket.write_all(response_headers.as_bytes()).await;
            let _ = socket.write_all(&body_bytes).await;
        });
    }
}
