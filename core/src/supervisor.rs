use crate::config::AppConfig;
use crate::tunnel;
use anyhow::{Context, Result};
use colored::*;
use std::process::{Child, Command, Stdio};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::time::Duration;

pub struct ServerSupervisor {
    config: AppConfig,
    running: Arc<AtomicBool>,
}

impl ServerSupervisor {
    pub fn new(config: AppConfig) -> Self {
        Self {
            config,
            running: Arc::new(AtomicBool::new(true)),
        }
    }

    pub fn start(&self) -> Result<()> {
        println!("{}", "=========================================================".cyan());
        println!("{}", "   Nextcloud Server for Windows (All-In-One Supervisor)  ".bold().green());
        println!("{}", "=========================================================".cyan());

        let mut active_children: Vec<(&'static str, Child)> = Vec::new();

        // 0. Ensure SQLite/libSQL WAL & concurrency PRAGMAs are active
        let db_file = self.config.data_dir.join("nextcloud.db");
        if let Err(e) = crate::db::apply_wal_optimizations(&db_file) {
            eprintln!("Warning: failed to apply SQLite WAL optimizations: {}", e);
        } else {
            println!("{} libSQL / SQLite WAL optimizations active on {:?}", "✓".green(), db_file);
        }

        // 1. Start PHP FastCGI
        println!("{} Starting PHP FastCGI on 127.0.0.1:{}...", "==>".blue(), self.config.fastcgi_port);
        let php_child = Command::new(&self.config.php_path)
            .args([
                "-b",
                &format!("127.0.0.1:{}", self.config.fastcgi_port),
                "-c",
                "config/php/php.ini",
            ])
            .stdout(Stdio::null())
            .stderr(Stdio::inherit())
            .spawn()
            .with_context(|| format!("Failed to start PHP-CGI from {:?}", self.config.php_path))?;
        active_children.push(("PHP-CGI", php_child));
        println!("{} PHP FastCGI engine running.", "✓".green());

        // 2. Start Caddy Web Server
        println!("{} Starting Caddy Web Server on port {}...", "==>".blue(), self.config.server_port);
        let caddy_child = Command::new(&self.config.caddy_path)
            .args(["run", "--config", "config/caddy/Caddyfile"])
            .stdout(Stdio::inherit())
            .stderr(Stdio::inherit())
            .spawn()
            .with_context(|| format!("Failed to start Caddy from {:?}", self.config.caddy_path))?;
        active_children.push(("Caddy", caddy_child));
        println!("{} Caddy Web Server running.", "✓".green());

        // 3. Start Cloudflare Tunnel (if token is set)
        if let Some(ref token) = self.config.cloudflare_token {
            if !token.trim().is_empty() {
                match tunnel::spawn_tunnel(&self.config.cloudflared_path, token) {
                    Ok(tunnel_child) => {
                        active_children.push(("Cloudflared", tunnel_child));
                    }
                    Err(e) => {
                        eprintln!("{} Failed to start Cloudflare Tunnel: {}", "[!]".red(), e);
                    }
                }
            }
        } else {
            println!("{} Cloudflare Tunnel token not configured. (Use 'set-token' to enable)", "ℹ".yellow());
        }

        println!("\n{} Nextcloud Server is active! Local URL: {}", "🚀".bold(), format!("http://localhost:{}", self.config.server_port).cyan());
        println!("Press Ctrl+C to shut down all services gracefully.\n");

        // Handle Ctrl+C signal for graceful shutdown
        let running_clone = Arc::clone(&self.running);
        ctrlc::set_handler(move || {
            println!("\n{} Shutdown signal received. Stopping services...", "==>".yellow());
            running_clone.store(false, Ordering::SeqCst);
        })
        .expect("Error setting Ctrl-C handler");

        // Supervisor loop: periodically monitor child health
        while self.running.load(Ordering::SeqCst) {
            std::thread::sleep(Duration::from_millis(1000));

            for (name, child) in active_children.iter_mut() {
                if let Ok(Some(exit_status)) = child.try_wait() {
                    println!("{} Process '{}' exited unexpectedly with status: {}", "[!]".red(), name, exit_status);
                    self.running.store(false, Ordering::SeqCst);
                    break;
                }
            }
        }

        // Cleanup: gracefully kill all spawned processes
        println!("{} Stopping child processes...", "==>".blue());
        for (name, mut child) in active_children {
            println!("  Terminating {}...", name);
            let _ = child.kill();
            let _ = child.wait();
        }
        println!("{} All server components stopped.", "✓".green());

        Ok(())
    }
}
