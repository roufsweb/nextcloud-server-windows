use std::path::Path;
use std::process::{Child, Command, Stdio};
use anyhow::{bail, Context, Result};
use colored::*;

#[cfg(target_os = "windows")]
use std::os::windows::process::CommandExt;
#[cfg(target_os = "windows")]
const CREATE_NO_WINDOW: u32 = 0x08000000;

/// Validates Cloudflare Tunnel Token format
pub fn validate_token(token: &str) -> Result<()> {
    let trimmed = token.trim();
    if trimmed.is_empty() {
        bail!("Cloudflare token cannot be empty.");
    }
    // Cloudflare Tunnel tokens are Base64-encoded JSON Web Tokens typically starting with eyJh
    if !trimmed.starts_with("eyJh") || trimmed.len() < 30 {
        bail!("Invalid Cloudflare Tunnel token format. It should be a base64 token starting with 'eyJh'.");
    }
    Ok(())
}

/// Spawns the cloudflared process with the given token
pub fn spawn_tunnel<P: AsRef<Path>>(cloudflared_bin: P, token: &str) -> Result<Child> {
    let bin_path = cloudflared_bin.as_ref();
    println!("{} Spawning Cloudflare Tunnel daemon...", "==>".blue());

    let mut cmd = Command::new(bin_path);
    cmd.args(["tunnel", "run", "--token", token.trim()])
       .stdout(Stdio::null())
       .stderr(Stdio::null());

    #[cfg(target_os = "windows")]
    {
        cmd.creation_flags(CREATE_NO_WINDOW);
    }

    let child = cmd.spawn()
        .with_context(|| format!("Failed to spawn cloudflared at {:?}", bin_path))?;

    println!("{} Cloudflare Tunnel started successfully.", "✓".green());
    Ok(child)
}
