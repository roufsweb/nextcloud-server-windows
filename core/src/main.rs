mod config;
mod drive;
mod supervisor;
mod tunnel;

use clap::{Parser, Subcommand};
use colored::*;
use config::AppConfig;
use std::path::PathBuf;
use std::process::Command;

const CONFIG_PATH: &str = "config/app_config.json";

#[derive(Parser)]
#[command(name = "nextcloud-aio")]
#[command(about = "All-In-One Nextcloud Server for Windows - Process Supervisor & Drive Access Manager", long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Start all Nextcloud services (Caddy, PHP FastCGI, Cloudflared tunnel)
    Start,

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
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let cli = Cli::parse();
    let mut config = AppConfig::load_or_default(CONFIG_PATH);

    match cli.command {
        Commands::Start => {
            let sup = supervisor::ServerSupervisor::new(config);
            sup.start()?;
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
            config.cloudflare_token = Some(token.trim().to_string());
            config.save(CONFIG_PATH)?;
            println!("{} Cloudflare Tunnel token configured successfully!", "✓".green());
        }
        Commands::GrantDrive { path } => {
            drive::grant_drive_permissions(&path)?;
            if !config.external_drives.contains(&path) {
                config.external_drives.push(path.clone());
                config.save(CONFIG_PATH)?;
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
    }

    Ok(())
}
