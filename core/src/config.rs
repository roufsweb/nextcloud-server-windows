use serde::{Deserialize, Serialize};
use std::fs;
use std::path::{Path, PathBuf};
use anyhow::Result;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct AppConfig {
    pub server_port: u16,
    pub fastcgi_port: u16,
    pub cloudflare_token: Option<String>,
    pub data_dir: PathBuf,
    pub caddy_path: PathBuf,
    pub php_path: PathBuf,
    pub cloudflared_path: PathBuf,
    pub external_drives: Vec<PathBuf>,
}

impl Default for AppConfig {
    fn default() -> Self {
        #[cfg(target_os = "windows")]
        let (caddy_bin, php_bin, cloudflared_bin, default_data) = (
            PathBuf::from("runtime/caddy/caddy.exe"),
            PathBuf::from("runtime/php/php-cgi.exe"),
            PathBuf::from("runtime/cloudflared/cloudflared.exe"),
            PathBuf::from("C:\\NextcloudAIO\\data"),
        );

        #[cfg(not(target_os = "windows"))]
        let (caddy_bin, php_bin, cloudflared_bin, default_data) = (
            PathBuf::from("caddy"),
            PathBuf::from("php"),
            PathBuf::from("cloudflared"),
            PathBuf::from("./data"),
        );

        Self {
            server_port: 8080,
            fastcgi_port: 9000,
            cloudflare_token: None,
            data_dir: default_data,
            caddy_path: caddy_bin,
            php_path: php_bin,
            cloudflared_path: cloudflared_bin,
            external_drives: Vec::new(),
        }
    }
}

impl AppConfig {
    pub fn load_or_default<P: AsRef<Path>>(path: P) -> Self {
        if let Ok(content) = fs::read_to_string(path.as_ref()) {
            if let Ok(cfg) = serde_json::from_str(&content) {
                return cfg;
            }
        }
        Self::default()
    }

    pub fn save<P: AsRef<Path>>(&self, path: P) -> Result<()> {
        if let Some(parent) = path.as_ref().parent() {
            fs::create_dir_all(parent)?;
        }
        let json = serde_json::to_string_pretty(self)?;
        fs::write(path, json)?;
        Ok(())
    }
}
