use crate::config::AppConfig;
use crate::tunnel;
use anyhow::Result;
use colored::*;
use std::process::{Child, Command, Stdio};
use std::time::Duration;

#[cfg(target_os = "windows")]
use std::os::windows::process::CommandExt;
#[cfg(target_os = "windows")]
const CREATE_NO_WINDOW: u32 = 0x08000000;

pub struct ServerSupervisor {
    config: AppConfig,
    active_children: Vec<(&'static str, Child)>,
    is_running: bool,
}

impl ServerSupervisor {
    pub fn new(config: AppConfig) -> Self {
        Self {
            config,
            active_children: Vec::new(),
            is_running: false,
        }
    }

    /// Spawns background server services (PHP FastCGI, Caddy, Cloudflared) silently
    /// with CREATE_NO_WINDOW to completely eliminate console flashing or desktop jitter.
    pub fn spawn_services(&mut self) -> Result<()> {
        // Ensure any previous instances are cleanly terminated first
        self.stop_services();

        println!("{}", "=========================================================".cyan());
        println!("{}", "   Nextcloud Server for Windows (All-In-One Supervisor)  ".bold().green());
        println!("{}", "=========================================================".cyan());

        // 0. Ensure SQLite/libSQL WAL & concurrency PRAGMAs are active
        let db_file = self.config.data_dir.join("nextcloud.db");
        if let Err(e) = crate::db::apply_wal_optimizations(&db_file) {
            eprintln!("Warning: failed to apply SQLite WAL optimizations: {}", e);
        } else {
            println!("{} libSQL / SQLite WAL optimizations active on {:?}", "✓".green(), db_file);
        }

        // 1. Start PHP FastCGI
        if self.config.php_path.exists() {
            println!("{} Starting PHP FastCGI on 127.0.0.1:{}...", "==>".blue(), self.config.fastcgi_port);
            let mut php_cmd = Command::new(&self.config.php_path);
            php_cmd.args([
                "-b",
                &format!("127.0.0.1:{}", self.config.fastcgi_port),
                "-c",
                "config/php/php.ini",
            ])
            .stdout(Stdio::null())
            .stderr(Stdio::null());

            #[cfg(target_os = "windows")]
            {
                php_cmd.creation_flags(CREATE_NO_WINDOW);
            }

            match php_cmd.spawn() {
                Ok(child) => {
                    self.active_children.push(("PHP-CGI", child));
                    println!("{} PHP FastCGI engine running silently.", "✓".green());
                }
                Err(e) => {
                    eprintln!("{} Failed to start PHP-CGI: {}", "[!]".red(), e);
                }
            }
        } else {
            println!(
                "{} PHP-CGI binary not found at {:?}. PHP engine on standby.",
                "ℹ".yellow(),
                self.config.php_path
            );
        }

        // 2. Start Caddy Web Server
        if self.config.caddy_path.exists() {
            println!("{} Starting Caddy Web Server on port {}...", "==>".blue(), self.config.server_port);
            let mut caddy_cmd = Command::new(&self.config.caddy_path);
            caddy_cmd.args(["run", "--config", "config/caddy/Caddyfile"])
                .stdout(Stdio::null())
                .stderr(Stdio::null());

            #[cfg(target_os = "windows")]
            {
                caddy_cmd.creation_flags(CREATE_NO_WINDOW);
            }

            match caddy_cmd.spawn() {
                Ok(child) => {
                    self.active_children.push(("Caddy", child));
                    println!("{} Caddy Web Server running silently.", "✓".green());
                }
                Err(e) => {
                    eprintln!("{} Failed to start Caddy: {}", "[!]".red(), e);
                }
            }
        } else {
            println!(
                "{} Caddy binary not found at {:?}. Web server on standby.",
                "ℹ".yellow(),
                self.config.caddy_path
            );
        }

        // 3. Start Cloudflare Tunnel (if token is set)
        if let Some(ref token) = self.config.cloudflare_token {
            if !token.trim().is_empty() && self.config.cloudflared_path.exists() {
                match tunnel::spawn_tunnel(&self.config.cloudflared_path, token) {
                    Ok(tunnel_child) => {
                        self.active_children.push(("Cloudflared", tunnel_child));
                    }
                    Err(e) => {
                        eprintln!("{} Failed to start Cloudflare Tunnel: {}", "[!]".red(), e);
                    }
                }
            }
        }

        self.is_running = true;
        println!("\n{} Nextcloud Server stack initialized. Local UI: {}", "🚀".bold(), format!("http://localhost:{}", self.config.server_port).cyan());
        Ok(())
    }

    /// Stops all running server child processes and ensures zero background tasks remain
    pub fn stop_services(&mut self) {
        if !self.active_children.is_empty() {
            println!("{} Stopping server child processes...", "==>".blue());
            for (name, mut child) in self.active_children.drain(..) {
                println!("  Terminating {}...", name);
                let _ = child.kill();
                let _ = child.wait();
            }
        }

        // On Windows, ensure any lingering worker processes spawned by PHP or Caddy are also swept
        #[cfg(target_os = "windows")]
        {
            let mut cmd = Command::new("cmd");
            cmd.args(["/c", "taskkill /F /IM caddy.exe /IM php-cgi.exe /IM cloudflared.exe >nul 2>&1"])
               .creation_flags(CREATE_NO_WINDOW);
            let _ = cmd.status();
        }

        self.is_running = false;
        println!("{} All server components stopped cleanly.", "✓".green());
    }

    /// Returns whether server services are currently active
    pub fn is_running(&self) -> bool {
        self.is_running
    }

    /// Run standalone supervisor monitoring loop (used by CLI `nextcloud-aio start`)
    #[allow(dead_code)]
    pub fn start(&mut self) -> Result<()> {
        self.spawn_services()?;

        println!("Press Ctrl+C to shut down all services gracefully.\n");

        while self.is_running {
            std::thread::sleep(Duration::from_millis(1000));

            for (name, child) in self.active_children.iter_mut() {
                if let Ok(Some(exit_status)) = child.try_wait() {
                    println!("{} Process '{}' exited with status: {}", "[!]".yellow(), name, exit_status);
                }
            }
        }

        self.stop_services();
        Ok(())
    }
}

impl Drop for ServerSupervisor {
    fn drop(&mut self) {
        self.stop_services();
    }
}
