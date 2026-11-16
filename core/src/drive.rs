use std::path::Path;
use std::process::Command;
use anyhow::{Context, Result};
use colored::*;

/// Grants full permissions and prepares external drives/directories for direct Nextcloud access
pub fn grant_drive_permissions<P: AsRef<Path>>(drive_path: P) -> Result<()> {
    let path = drive_path.as_ref();
    let path_str = path.to_string_lossy();

    println!("{} Preparing drive path: {}", "==>".blue(), path_str.cyan());

    if !path.exists() {
        anyhow::bail!("Path does not exist: {}", path_str);
    }

    #[cfg(target_os = "windows")]
    {
        println!("{} Applying Windows NTFS ACLs (Full Control with inheritance)...", " ->".yellow());
        // Use universal SID *S-1-1-0 ("Everyone") to work across all language editions of Windows
        let output = Command::new("icacls")
            .arg(format!("\"{}\"", path_str))
            .arg("/grant")
            .arg("*S-1-1-0:(OI)(CI)F")
            .arg("/T")
            .arg("/C")
            .arg("/Q")
            .output()
            .with_context(|| format!("Failed to execute icacls on {}", path_str))?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            println!("{} Warning from icacls: {}", "[!]".yellow(), stderr);
        } else {
            println!("{} Successfully granted full NTFS control on: {}", "✓".green(), path_str);
        }
    }

    #[cfg(not(target_os = "windows"))]
    {
        println!("{} Setting standard POSIX permissions on: {}", " ->".yellow(), path_str);
        let status = Command::new("chmod")
            .args(["-R", "u+rwX,g+rwX,o+rX", path.to_str().unwrap()])
            .status()
            .with_context(|| format!("Failed to chmod {}", path_str))?;

        if status.success() {
            println!("{} Successfully updated permissions on: {}", "✓".green(), path_str);
        }
    }

    Ok(())
}

/// Verifies whether the process can read and write to the drive directory
pub fn verify_drive_read_write<P: AsRef<Path>>(drive_path: P) -> Result<bool> {
    let test_file = drive_path.as_ref().join(".nc_write_test.tmp");
    if let Err(e) = std::fs::write(&test_file, b"nextcloud_rw_verify") {
        eprintln!("Write test failed: {}", e);
        return Ok(false);
    }
    let _ = std::fs::remove_file(test_file);
    Ok(true)
}
