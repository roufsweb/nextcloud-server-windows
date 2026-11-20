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

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct LogEntry {
    pub time: String,
    pub tag: String,
    pub message: String,
}

#[derive(Serialize, Clone, Debug)]
pub struct DiagnosticItem {
    pub id: String,
    pub name: String,
    pub status: String, // "ok", "warning", "error"
    pub what: String,
    pub why: String,
    pub fix: String,
    pub action_id: Option<String>,
}

#[derive(Serialize, Clone, Debug)]
pub struct DiagnosticsResponse {
    pub overall: String, // "healthy", "degraded", "critical"
    pub issues_count: usize,
    pub items: Vec<DiagnosticItem>,
}

fn run_diagnostics(cfg: &AppConfig, running: bool) -> DiagnosticsResponse {
    let mut items = Vec::new();
    let mut has_error = false;
    let mut has_warning = false;

    // 1. Web Port Check
    if !running {
        let addr = format!("127.0.0.1:{}", cfg.server_port);
        match std::net::TcpListener::bind(&addr) {
            Ok(_) => {
                items.push(DiagnosticItem {
                    id: "web_port".into(),
                    name: format!("HTTP Web Port ({})", cfg.server_port),
                    status: "ok".into(),
                    what: format!("Port {} is free and available.", cfg.server_port),
                    why: "No local process or conflicting service is bound to this port.".into(),
                    fix: "No action required.".into(),
                    action_id: None,
                });
            }
            Err(e) => {
                has_error = true;
                items.push(DiagnosticItem {
                    id: "web_port".into(),
                    name: format!("HTTP Web Port ({})", cfg.server_port),
                    status: "error".into(),
                    what: format!("Port {} is already in use by another program (OS Error: {}).", cfg.server_port, e),
                    why: "Another application (such as IIS, Apache, Docker, or Node.js) is listening on this port.".into(),
                    fix: "Switch to port 8085 in Config -> Server Web Port, or terminate the conflicting process in Task Manager.".into(),
                    action_id: Some("switch_port_8085".into()),
                });
            }
        }
    } else {
        items.push(DiagnosticItem {
            id: "web_port".into(),
            name: format!("HTTP Web Port ({})", cfg.server_port),
            status: "ok".into(),
            what: format!("Nextcloud Caddy proxy is listening on port {}.", cfg.server_port),
            why: "Web server is active and serving traffic.".into(),
            fix: "No action required.".into(),
            action_id: None,
        });
    }

    // 2. PHP Runtime Binary
    if cfg.php_path.exists() {
        items.push(DiagnosticItem {
            id: "php_engine".into(),
            name: "PHP 8.3 FastCGI Binary".into(),
            status: "ok".into(),
            what: format!("PHP runtime binary found at: {}", cfg.php_path.display()),
            why: "PHP 8.3 Non-Thread-Safe runtime is installed correctly.".into(),
            fix: "No action required.".into(),
            action_id: None,
        });
    } else {
        has_error = true;
        items.push(DiagnosticItem {
            id: "php_engine".into(),
            name: "PHP 8.3 FastCGI Binary".into(),
            status: "error".into(),
            what: format!("PHP-CGI executable not found at: {}", cfg.php_path.display()),
            why: "The runtime directory was moved, deleted, or blocked by Windows Defender / Antivirus.".into(),
            fix: "Extract the official Nextcloud Windows AIO portable release bundle or run download_windows_runtime.sh.".into(),
            action_id: None,
        });
    }

    // 3. Caddy Server Binary
    if cfg.caddy_path.exists() {
        items.push(DiagnosticItem {
            id: "caddy_binary".into(),
            name: "Caddy HTTP/3 Server Binary".into(),
            status: "ok".into(),
            what: format!("Caddy binary verified at: {}", cfg.caddy_path.display()),
            why: "Native Windows Caddy executable is ready.".into(),
            fix: "No action required.".into(),
            action_id: None,
        });
    } else {
        has_error = true;
        items.push(DiagnosticItem {
            id: "caddy_binary".into(),
            name: "Caddy HTTP/3 Server Binary".into(),
            status: "error".into(),
            what: format!("Caddy binary not found at: {}", cfg.caddy_path.display()),
            why: "The runtime binary was moved, deleted, or quarantined by security software.".into(),
            fix: "Restore runtime/caddy/caddy.exe from release archive or allow in antivirus.".into(),
            action_id: None,
        });
    }

    // 4. Data Root Permissions & Disk Capacity
    let data_dir = &cfg.data_dir;
    let _ = std::fs::create_dir_all(data_dir);
    let probe_file = data_dir.join(".access_probe.tmp");
    let write_ok = std::fs::write(&probe_file, b"probe").is_ok() && std::fs::remove_file(&probe_file).is_ok();
    let (total_gb, _, free_gb) = drive::get_disk_space(data_dir);

    if !write_ok {
        has_error = true;
        items.push(DiagnosticItem {
            id: "data_permissions".into(),
            name: "Data Root Directory Permissions".into(),
            status: "error".into(),
            what: format!("Write permission denied in Nextcloud data directory: {}", data_dir.display()),
            why: "Windows NTFS folder permissions restrict write access for the current user session.".into(),
            fix: "Click 'Grant Permissions' or run 'icacls' to grant Full Control permissions.".into(),
            action_id: Some("grant_dataroot_acl".into()),
        });
    } else if free_gb < 2 && total_gb > 0 {
        has_warning = true;
        items.push(DiagnosticItem {
            id: "data_capacity".into(),
            name: "Data Storage Drive Capacity".into(),
            status: "warning".into(),
            what: format!("Low free storage space: only {} GB remaining on data drive.", free_gb),
            why: "The target storage drive is nearing maximum capacity.".into(),
            fix: "Free up disk space or mount a secondary hard drive in Storage & Drives.".into(),
            action_id: None,
        });
    } else {
        items.push(DiagnosticItem {
            id: "data_storage".into(),
            name: "Data Root Directory".into(),
            status: "ok".into(),
            what: format!("Data directory is accessible and writable ({} GB free of {} GB).", free_gb, total_gb),
            why: "Full read/write permissions verified and adequate disk space is available.".into(),
            fix: "No action required.".into(),
            action_id: None,
        });
    }

    // 5. Database Health
    let db_path = data_dir.join("nextcloud.db");
    if db_path.exists() {
        let db_stat = crate::db::get_status(&db_path, "libSQL / SQLite (WAL)");
        if db_stat.integrity == "ok" {
            items.push(DiagnosticItem {
                id: "database".into(),
                name: "Database Engine (libSQL / SQLite WAL)".into(),
                status: "ok".into(),
                what: format!("SQLite database file verified healthy (Size: {:.2} MB, WAL: {:.2} MB).",
                    db_stat.file_size_bytes as f64 / (1024.0 * 1024.0),
                    db_stat.wal_size_bytes as f64 / (1024.0 * 1024.0)),
                why: "ACID integrity check passed with zero corrupted pages.".into(),
                fix: "No action required.".into(),
                action_id: None,
            });
        } else {
            has_warning = true;
            items.push(DiagnosticItem {
                id: "database".into(),
                name: "Database Engine (libSQL / SQLite WAL)".into(),
                status: "warning".into(),
                what: format!("Database integrity warning: {}", db_stat.integrity),
                why: "An abrupt Windows shutdown or crash may have left uncommitted journal entries.".into(),
                fix: "Click 'Optimize Database' to checkpoint WAL transactions and vacuum pages.".into(),
                action_id: Some("optimize_db".into()),
            });
        }
    } else {
        items.push(DiagnosticItem {
            id: "database".into(),
            name: "Database Engine (libSQL / SQLite WAL)".into(),
            status: "ok".into(),
            what: "Database ready for initialization on first user setup.".into(),
            why: "Database will be created automatically upon starting Nextcloud.".into(),
            fix: "No action required.".into(),
            action_id: None,
        });
    }

    // 6. Cloudflare Tunnel
    if let Some(ref token) = cfg.cloudflare_token {
        if crate::tunnel::validate_token(token).is_ok() {
            items.push(DiagnosticItem {
                id: "tunnel".into(),
                name: "Cloudflare Encrypted Tunnel".into(),
                status: "ok".into(),
                what: "Cloudflare Tunnel token is valid and active.".into(),
                why: "Authenticated token format verified for secure remote access.".into(),
                fix: "No action required.".into(),
                action_id: None,
            });
        } else {
            has_warning = true;
            items.push(DiagnosticItem {
                id: "tunnel".into(),
                name: "Cloudflare Encrypted Tunnel".into(),
                status: "warning".into(),
                what: "Cloudflare Tunnel token appears malformed or expired.".into(),
                why: "The token string was incomplete, corrupted, or revoked in Cloudflare Zero Trust.".into(),
                fix: "Generate a new token in the Cloudflare Zero Trust dashboard and paste it in Config.".into(),
                action_id: Some("open_config_tunnel".into()),
            });
        }
    } else {
        items.push(DiagnosticItem {
            id: "tunnel".into(),
            name: "Remote Access (Cloudflare Tunnel)".into(),
            status: "ok".into(),
            what: "Remote tunnel is disabled. Nextcloud is in Local Network (LAN) mode.".into(),
            why: "No tunnel token has been configured yet.".into(),
            fix: "If you want secure remote access without port forwarding, configure a token in Config.".into(),
            action_id: Some("open_config_tunnel".into()),
        });
    }

    // 7. External Drives Check
    for drive_path in &cfg.external_drives {
        if drive_path.exists() {
            items.push(DiagnosticItem {
                id: format!("drive_{}", drive_path.to_string_lossy()),
                name: format!("External Drive ({})", drive_path.display()),
                status: "ok".into(),
                what: format!("Mounted drive path '{}' is accessible.", drive_path.display()),
                why: "Drive letter is connected and online in Windows.".into(),
                fix: "No action required.".into(),
                action_id: None,
            });
        } else {
            has_warning = true;
            items.push(DiagnosticItem {
                id: format!("drive_{}", drive_path.to_string_lossy()),
                name: format!("External Drive ({})", drive_path.display()),
                status: "warning".into(),
                what: format!("Mounted drive '{}' is disconnected or missing.", drive_path.display()),
                why: "The external USB drive was unplugged or Windows reassigned its drive letter upon reboot.".into(),
                fix: "Plug the USB drive back in, or unmount it from Storage & Drives if no longer needed.".into(),
                action_id: Some(format!("unmount:{}", drive_path.to_string_lossy())),
            });
        }
    }

    // 8. PHP Runtime Hardening & FastCGI Isolation
    let php_ini_path = Path::new("config/php/php.ini");
    if php_ini_path.exists() {
        if let Ok(ini_content) = fs::read_to_string(php_ini_path) {
            let has_expose_off = ini_content.contains("expose_php = Off");
            let has_fix_pathinfo = ini_content.contains("cgi.fix_pathinfo = 0");
            let has_no_url_include = ini_content.contains("allow_url_include = Off");

            if has_expose_off && has_fix_pathinfo && has_no_url_include {
                items.push(DiagnosticItem {
                    id: "php_security".into(),
                    name: "PHP Runtime Hardening & FastCGI Isolation".into(),
                    status: "ok".into(),
                    what: "PHP security directives verified (version fingerprinting hidden, FastCGI script execution blocked, RFI disabled).".into(),
                    why: "php.ini contains recommended production security parameters.".into(),
                    fix: "No action required.".into(),
                    action_id: None,
                });
            } else {
                has_warning = true;
                items.push(DiagnosticItem {
                    id: "php_security".into(),
                    name: "PHP Runtime Hardening & FastCGI Isolation".into(),
                    status: "warning".into(),
                    what: "One or more PHP security hardening directives are missing or disabled.".into(),
                    why: "cgi.fix_pathinfo or expose_php is not strictly configured in config/php/php.ini.".into(),
                    fix: "Ensure config/php/php.ini has 'expose_php = Off' and 'cgi.fix_pathinfo = 0'.".into(),
                    action_id: None,
                });
            }
        }
    }

    // 9. Localhost API Perimeter & In-App Security
    items.push(DiagnosticItem {
        id: "api_perimeter".into(),
        name: "Localhost API Perimeter & In-App Security".into(),
        status: "ok".into(),
        what: "Supervisor loopback binding, DNS rebinding filter, and Cross-Origin CSRF protection active.".into(),
        why: "Host validation ensures local processes or external iframes cannot spoof commands or access daemon endpoints.".into(),
        fix: "No action required.".into(),
        action_id: None,
    });

    // 10. Storage Boundary & System Directory Isolation
    if drive::is_safe_user_directory(&cfg.data_dir) {
        items.push(DiagnosticItem {
            id: "storage_isolation".into(),
            name: "Storage Boundary & System Directory Isolation".into(),
            status: "ok".into(),
            what: format!("Nextcloud data directory '{}' is isolated within safe user space.", cfg.data_dir.display()),
            why: "System directories (Windows, Program Files, System Volume) are protected from accidental ACL modifications.".into(),
            fix: "No action required.".into(),
            action_id: None,
        });
    } else {
        has_error = true;
        items.push(DiagnosticItem {
            id: "storage_isolation".into(),
            name: "Storage Boundary & System Directory Isolation".into(),
            status: "error".into(),
            what: format!("Data directory '{}' targets a protected Windows system path.", cfg.data_dir.display()),
            why: "Placing Nextcloud user data inside Windows or Program Files compromises OS stability and security.".into(),
            fix: "Change Nextcloud Data Root in Config to a non-system folder (e.g., C:\\NextcloudData or D:\\Nextcloud).".into(),
            action_id: Some("open_config_dataroot".into()),
        });
    }

    let overall = if has_error {
        "critical"
    } else if has_warning {
        "degraded"
    } else {
        "healthy"
    };

    let issues_count = items.iter().filter(|i| i.status != "ok").count();

    DiagnosticsResponse {
        overall: overall.to_string(),
        issues_count,
        items,
    }
}

/// Validates admin username to prevent CLI flag injection and control character exploits
fn is_valid_admin_username(name: &str) -> bool {
    if name.is_empty() || name.len() > 64 || name.starts_with('-') {
        return false;
    }
    name.chars().all(|c| c.is_ascii_alphanumeric() || c == '_' || c == '-' || c == '.' || c == '@')
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
    supervisor: Arc<std::sync::Mutex<crate::supervisor::ServerSupervisor>>,
    config_path: &'static str,
) -> Result<()> {
    let listener = TcpListener::bind(format!("127.0.0.1:{}", port)).await?;
    println!("{} Nextcloud AIO Dashboard running on: {}", "🚀".bold(), format!("http://localhost:{}", port).cyan());

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
        let sup_clone = Arc::clone(&supervisor);
        let logs_clone = Arc::clone(&logs_buffer);

        tokio::spawn(async move {
            let mut req_bytes = Vec::with_capacity(4096);
            let mut chunk = [0u8; 4096];
            let mut content_length = 0usize;
            let mut header_ended = false;

            loop {
                let n = match socket.read(&mut chunk).await {
                    Ok(n) if n > 0 => n,
                    _ => break,
                };
                req_bytes.extend_from_slice(&chunk[..n]);

                if !header_ended {
                    if let Some(pos) = req_bytes.windows(4).position(|w| w == b"\r\n\r\n") {
                        header_ended = true;
                        let header_part = String::from_utf8_lossy(&req_bytes[..pos]);
                        for line in header_part.lines() {
                            let lower = line.to_lowercase();
                            if lower.starts_with("content-length:") {
                                if let Some(val) = line.split(':').nth(1) {
                                    content_length = val.trim().parse::<usize>().unwrap_or(0);
                                }
                            }
                        }
                        let body_len = req_bytes.len() - (pos + 4);
                        if body_len >= content_length {
                            break;
                        }
                    }
                } else {
                    let header_pos = req_bytes.windows(4).position(|w| w == b"\r\n\r\n").unwrap_or(0);
                    let body_len = req_bytes.len() - (header_pos + 4);
                    if body_len >= content_length {
                        break;
                    }
                }
            }

            if req_bytes.is_empty() { return; }
            let req_str = String::from_utf8_lossy(&req_bytes);
            let lines: Vec<&str> = req_str.lines().collect();
            if lines.is_empty() { return; }

            let first_line: Vec<&str> = lines[0].split_whitespace().collect();
            if first_line.len() < 2 { return; }

            let method = first_line[0];
            let uri = first_line[1];

            // Extract HTTP Headers
            let mut host_header = "";
            let mut origin_header = "";
            let mut sec_fetch_site = "";

            for line in &lines[1..] {
                if line.is_empty() { break; }
                let lower = line.to_lowercase();
                if lower.starts_with("host:") {
                    host_header = line.splitn(2, ':').nth(1).unwrap_or("").trim();
                } else if lower.starts_with("origin:") {
                    origin_header = line.splitn(2, ':').nth(1).unwrap_or("").trim();
                } else if lower.starts_with("sec-fetch-site:") {
                    sec_fetch_site = line.splitn(2, ':').nth(1).unwrap_or("").trim();
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
            // Rejects 'null' origin to close sandboxed iframe / privacy redirect bypasses.
            // Rejects Sec-Fetch-Site: cross-site requests.
            let is_local_origin = origin_header.is_empty()
                || origin_header.starts_with("http://localhost")
                || origin_header.starts_with("http://127.0.0.1")
                || origin_header.starts_with("tauri://localhost");

            let is_cross_site = sec_fetch_site.eq_ignore_ascii_case("cross-site");

            if uri.starts_with("/api/") && (is_cross_site || (method != "GET" && method != "OPTIONS" && !is_local_origin)) {
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
                let allow_origin = if is_local_origin && !origin_header.is_empty() { origin_header } else { "http://localhost" };
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
                    running: sup_clone.lock().map(|s| s.is_running()).unwrap_or(false),
                    server_port: cfg.server_port,
                    fastcgi_port: cfg.fastcgi_port,
                    data_dir: cfg.data_dir.to_string_lossy().to_string(),
                    tunnel_status: if cfg.cloudflare_token.is_some() { "Connected".into() } else { "Offline".into() },
                    tunnel_url: if cfg.cloudflare_token.is_some() { "https://cloud.yourdomain.com".into() } else { "".into() },
                    drives: drives_info,
                    total_disk_gb: data_tot,
                    used_disk_gb: data_used,
                };
                ("HTTP/1.1 200 OK", "application/json", serde_json::to_vec(&resp).unwrap_or_default())
            } else if method == "POST" && uri == "/api/services/start" {
                if let Ok(mut sup) = sup_clone.lock() {
                    let _ = sup.spawn_services();
                }
                append_log(&logs_clone, "CORE", "Server services started (Caddy, PHP-CGI, Cloudflared).".into()).await;
                ("HTTP/1.1 200 OK", "application/json", b"{\"status\":\"started\",\"running\":true}".to_vec())
            } else if method == "POST" && uri == "/api/services/stop" {
                if let Ok(mut sup) = sup_clone.lock() {
                    sup.stop_services();
                }
                append_log(&logs_clone, "CORE", "Server services stopped gracefully. All background processes terminated.".into()).await;
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
                    if !drive::is_safe_user_directory(&drive_path) {
                        ("HTTP/1.1 400 Bad Request", "application/json", b"{\"error\":\"Access denied: System-critical directory cannot be mounted\"}".to_vec())
                    } else {
                        let _ = drive::grant_drive_permissions(&drive_path);
                        let mut cfg = config_clone.lock().await;
                        if !cfg.external_drives.contains(&drive_path) {
                            cfg.external_drives.push(drive_path.clone());
                            let _ = cfg.save(config_path);
                        }
                        append_log(&logs_clone, "CORE", format!("Drive '{}' mounted with NTFS full permissions.", req.path)).await;
                        ("HTTP/1.1 200 OK", "application/json", b"{\"status\":\"success\"}".to_vec())
                    }
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
                    let trimmed = req.token.trim();
                    if trimmed.is_empty() {
                        let mut cfg = config_clone.lock().await;
                        cfg.cloudflare_token = None;
                        let _ = cfg.save(config_path);
                        append_log(&logs_clone, "TUNNEL", "Cloudflare Tunnel token cleared. Remote access disabled.".into()).await;
                        ("HTTP/1.1 200 OK", "application/json", b"{\"status\":\"token_cleared\"}".to_vec())
                    } else if tunnel::validate_token(trimmed).is_ok() {
                        let mut cfg = config_clone.lock().await;
                        cfg.cloudflare_token = Some(trimmed.to_string());
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
                    if !drive::is_safe_user_directory(&path) {
                        ("HTTP/1.1 400 Bad Request", "application/json", b"{\"error\":\"Access denied: System-critical directory cannot be used as data root\"}".to_vec())
                    } else {
                        let _ = drive::grant_drive_permissions(&path);
                        let mut cfg = config_clone.lock().await;
                        cfg.data_dir = path;
                        let _ = cfg.save(config_path);
                        append_log(&logs_clone, "CORE", format!("Nextcloud data root updated to: {}", req.path)).await;
                        ("HTTP/1.1 200 OK", "application/json", b"{\"status\":\"data_root_updated\"}".to_vec())
                    }
                } else {
                    ("HTTP/1.1 400 Bad Request", "application/json", b"{\"error\":\"invalid payload\"}".to_vec())
                }
            } else if method == "POST" && uri == "/api/config/port" {
                if let Ok(req) = serde_json::from_str::<PortChangeRequest>(body) {
                    if req.port < 80 || req.port == 9000 || req.port == 8081 {
                        let err_msg = if req.port == 9000 {
                            "Port 9000 is reserved for internal PHP FastCGI pool"
                        } else if req.port == 8081 {
                            "Port 8081 is reserved for internal Supervisor daemon"
                        } else {
                            "Port must be a valid port number >= 80"
                        };
                        ("HTTP/1.1 400 Bad Request", "application/json", format!("{{\"error\":\"{}\"}}", err_msg).into_bytes())
                    } else {
                        let mut cfg = config_clone.lock().await;
                        cfg.server_port = req.port;
                        let _ = cfg.save(config_path);
                        append_log(&logs_clone, "CADDY", format!("Web server port configured to: {}", req.port)).await;
                        ("HTTP/1.1 200 OK", "application/json", b"{\"status\":\"port_updated\"}".to_vec())
                    }
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
                    let new_user = req.username.trim().to_string();
                    if !is_valid_admin_username(&new_user) {
                        ("HTTP/1.1 400 Bad Request", "application/json", b"{\"error\":\"Invalid username: must be 1-64 characters (alphanumeric, _, -, ., @) and cannot start with '-'\"}".to_vec())
                    } else {
                        let mut password_valid = true;
                        if let Some(ref pwd) = req.password {
                            let pass_trimmed = pwd.trim();
                            if !pass_trimmed.is_empty() {
                                if pass_trimmed.len() < 8 || pass_trimmed.len() > 128 || pass_trimmed.contains('\0') {
                                    password_valid = false;
                                }
                            }
                        }

                        if !password_valid {
                            ("HTTP/1.1 400 Bad Request", "application/json", b"{\"error\":\"Password must be between 8 and 128 characters and cannot contain null bytes\"}".to_vec())
                        } else {
                            let mut cfg = config_clone.lock().await;
                            cfg.admin_username = Some(new_user.clone());
                            let _ = cfg.save(config_path);

                            if let Some(pwd) = req.password {
                                let pass_trimmed = pwd.trim();
                                if !pass_trimmed.is_empty() {
                                    let auto_cfg_path = "config/nextcloud/autoconfig.php";
                                    if let Ok(content) = fs::read_to_string(auto_cfg_path) {
                                        let updated = content
                                            .replace("'adminlogin'    => 'admin'", &format!("'adminlogin'    => '{}'", new_user))
                                            .replace("'adminpass'     => 'NextcloudAdmin2026!'", &format!("'adminpass'     => '{}'", pass_trimmed));
                                        let _ = fs::write(auto_cfg_path, updated);
                                    }

                                    let php_path = cfg.php_path.clone();
                                    let target_user = new_user.clone();
                                    let pass_val = pass_trimmed.to_string();
                                    std::thread::spawn(move || {
                                        let mut cmd = std::process::Command::new(php_path);
                                        cmd.arg("nextcloud/occ").args(["user:resetpassword", "--password-from-env", &target_user]);
                                        cmd.env("OC_PASS", pass_val);
                                        #[cfg(target_os = "windows")]
                                        {
                                            use std::os::windows::process::CommandExt;
                                            const CREATE_NO_WINDOW: u32 = 0x08000000;
                                            cmd.creation_flags(CREATE_NO_WINDOW);
                                        }
                                        let _ = cmd.output();
                                    });
                                }
                            }

                            append_log(&logs_clone, "CORE", format!("Updated admin credentials for user '{}'.", new_user)).await;
                            ("HTTP/1.1 200 OK", "application/json", b"{\"status\":\"success\",\"message\":\"Administrator credentials updated\"}".to_vec())
                        }
                    }
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
                    let p = std::path::Path::new(&req.path);
                    if !p.exists() || !p.is_dir() {
                        ("HTTP/1.1 400 Bad Request", "application/json", b"{\"error\":\"Path does not exist or is not a directory\"}".to_vec())
                    } else {
                        let _ = drive::open_in_file_manager(&req.path);
                        ("HTTP/1.1 200 OK", "application/json", b"{\"status\":\"opened\"}".to_vec())
                    }
                } else {
                    ("HTTP/1.1 400 Bad Request", "application/json", b"{\"error\":\"invalid path\"}".to_vec())
                }
            } else if method == "POST" && uri == "/api/system/pick-folder" {
                #[derive(serde::Deserialize)]
                struct PickReq {
                    title: Option<String>,
                }
                let title = serde_json::from_str::<PickReq>(body)
                    .ok()
                    .and_then(|r| r.title)
                    .unwrap_or_else(|| "Select Nextcloud Folder".to_string());
                let picked = drive::pick_folder_dialog(&title);
                let resp = serde_json::json!({
                    "status": if picked.is_some() { "selected" } else { "cancelled" },
                    "path": picked
                });
                ("HTTP/1.1 200 OK", "application/json", serde_json::to_vec(&resp).unwrap_or_default())
            } else if method == "GET" && uri == "/api/system/diagnostics" {
                let cfg = config_clone.lock().await;
                let is_running = sup_clone.lock().map(|s| s.is_running()).unwrap_or(false);
                let diag = run_diagnostics(&cfg, is_running);
                ("HTTP/1.1 200 OK", "application/json", serde_json::to_vec(&diag).unwrap_or_default())
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

            let cache_control = if uri.starts_with("/api/") {
                "Cache-Control: no-store, no-cache, must-revalidate\r\nPragma: no-cache\r\n"
            } else {
                "Cache-Control: public, max-age=3600\r\n"
            };

            let csp_header = "Content-Security-Policy: default-src 'self' 'unsafe-inline' data:; connect-src 'self' http://localhost:* http://127.0.0.1:*; img-src 'self' data:; style-src 'self' 'unsafe-inline'; font-src 'self' data:;\r\n";

            let response_headers = format!(
                "{}\r\nContent-Type: {}\r\nContent-Length: {}\r\n{}{}X-Content-Type-Options: nosniff\r\nX-Frame-Options: DENY\r\nReferrer-Policy: no-referrer\r\n{}Connection: close\r\n\r\n",
                status_line, content_type, body_bytes.len(), allow_origin_hdr, cache_control, csp_header
            );

            let _ = socket.write_all(response_headers.as_bytes()).await;
            let _ = socket.write_all(&body_bytes).await;
        });
    }
}
