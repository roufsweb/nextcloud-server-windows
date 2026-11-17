mod config;
mod drive;
mod gui;
mod server;
mod supervisor;
mod tunnel;

use clap::{Parser, Subcommand};
use colored::*;
use config::AppConfig;
use std::path::PathBuf;
use std::process::Command;
use std::sync::Arc;
use tokio::sync::Mutex;

const CONFIG_PATH: &str = "config/app_config.json";

#[derive(Parser)]
#[command(name = "nextcloud-aio")]
#[command(about = "All-In-One Nextcloud Server for Windows - Desktop Supervisor & Drive Access Manager", long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Start Nextcloud services (Caddy, PHP FastCGI, Cloudflared tunnel) + in-app dashboard
    Start {
        /// Dashboard UI port
        #[arg(short, long, default_value_t = 8081)]
        dashboard_port: u16,

        /// Automatically open the standalone in-app desktop GUI window
        #[arg(long, default_value_t = false)]
        gui: bool,
    },

    /// Launch the standalone Modern Desktop Dashboard GUI
    Gui {
        /// Dashboard UI port
        #[arg(short, long, default_value_t = 8081)]
        port: u16,
    },

    /// Check health and status of all components
    Status,

    /// Configure or update the Cloudflare Tunnel token
    SetToken {
        /// Cloudflare Tunnel Token (Base64 JWT starting with 'eyJh')
        token: String,
    },

    /// Grant full Windows NTFS permissions for direct Nextcloud external drive access
    GrantDrive {
        /// Target drive or directory path (e.g. D:\ or D:\NextcloudData)
        path: PathBuf,
    },

    /// Run Nextcloud's command-line tool (occ) via the bundled PHP engine
    Occ {
        /// Arguments to pass to occ
        #[arg(trailing_var_arg = true, allow_hyphen_values = true)]
        args: Vec<String>,
    },

    /// Run the dashboard REST server only (headless / CI / test mode)
    Server {
        /// Dashboard UI port
        #[arg(short, long, default_value_t = 8081)]
        port: u16,

        /// Custom config path
        #[arg(short, long, default_value = "config/app_config.json")]
        config: String,
    },
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let cli = Cli::parse();
    let config = AppConfig::load_or_default(CONFIG_PATH);
    let shared_config = Arc::new(Mutex::new(config.clone()));

    match cli.command {
        Commands::Start { dashboard_port, gui } => {
            // Spawn internal dashboard HTTP server in background
            let srv_cfg = Arc::clone(&shared_config);
            tokio::spawn(async move {
                if let Err(e) = server::run_dashboard_server(dashboard_port, srv_cfg, CONFIG_PATH).await {
                    eprintln!("Dashboard server error: {}", e);
                }
            });

            if gui {
                // Wait briefly for HTTP server to bind before launching in-app GUI
                tokio::spawn(async move {
                    tokio::time::sleep(tokio::time::Duration::from_millis(500)).await;
                    let _ = gui::spawn_in_app_gui(dashboard_port);
                });
            }

            // Start backend process supervisor
            let sup = supervisor::ServerSupervisor::new(config);
            sup.start()?;
        }
        Commands::Gui { port } => {
            println!("{}", "=========================================================".cyan());
            println!("{}", "   Nextcloud AIO Standalone Desktop Dashboard (In-App GUI) ".bold().green());
            println!("{}", "=========================================================".cyan());

            let srv_cfg = Arc::clone(&shared_config);
            tokio::spawn(async move {
                let _ = server::run_dashboard_server(port, srv_cfg, CONFIG_PATH).await;
            });

            tokio::time::sleep(tokio::time::Duration::from_millis(500)).await;
            if let Ok(mut p) = gui::spawn_in_app_gui(port) {
                let _ = p.wait();
            }
        }
        Commands::Status => {
            println!("{}", "=== Nextcloud Server AIO Status ===".bold().cyan());
            println!("Server Port:       {}", config.server_port);
            println!("FastCGI Port:      {}", config.fastcgi_port);
            println!("Data Directory:    {}", config.data_dir.display());
            println!("Cloudflare Tunnel: {}", match &config.cloudflare_token {
                Some(t) if !t.is_empty() => "Configured (Active)".green(),
                _ => "Not configured".yellow(),
            });
            println!("Configured Drives: {}", config.external_drives.len());
            for d in &config.external_drives {
                let rw_ok = drive::verify_drive_read_write(d).unwrap_or(false);
                println!("  - {}: {}", d.display(), if rw_ok { "Accessible (Full R/W)".green() } else { "Permission Issue / Inaccessible".red() });
            }
        }
        Commands::SetToken { token } => {
            tunnel::validate_token(&token)?;
            let mut cfg = shared_config.lock().await;
            cfg.cloudflare_token = Some(token.trim().to_string());
            cfg.save(CONFIG_PATH)?;
            println!("{} Cloudflare Tunnel token configured successfully!", "✓".green());
        }
        Commands::GrantDrive { path } => {
            drive::grant_drive_permissions(&path)?;
            let mut cfg = shared_config.lock().await;
            if !cfg.external_drives.contains(&path) {
                cfg.external_drives.push(path.clone());
                cfg.save(CONFIG_PATH)?;
            }
            println!("{} Drive registered and permissions verified for Nextcloud direct access.", "✓".green());
        }
        Commands::Occ { args } => {
            println!("{} Executing Nextcloud occ...", "==>".blue());
            let mut cmd = Command::new(&config.php_path);
            cmd.arg("nextcloud/occ");
            cmd.args(&args);
            let status = cmd.status()?;
            std::process::exit(status.code().unwrap_or(1));
        }
        Commands::Server { port, config: cfg_file } => {
            let static_cfg: &'static str = Box::leak(cfg_file.into_boxed_str());
            let loaded_config = AppConfig::load_or_default(static_cfg);
            let shared_config = Arc::new(Mutex::new(loaded_config));
            println!("{} Starting Nextcloud AIO Dashboard server on port {}...", "==>".blue(), port);
            server::run_dashboard_server(port, shared_config, static_cfg).await?;
        }
    }

    Ok(())
}
