#[cfg(target_os = "windows")]
use std::process::Command;

/// Checks if Nextcloud AIO is registered in Windows Startup (HKCU Run key)
pub fn is_autostart_enabled() -> bool {
    #[cfg(target_os = "windows")]
    {
        let output = Command::new("reg")
            .args(["query", "HKCU\\Software\\Microsoft\\Windows\\CurrentVersion\\Run", "/v", "NextcloudAIO"])
            .output();
        if let Ok(out) = output {
            return out.status.success();
        }
        false
    }
    #[cfg(not(target_os = "windows"))]
    {
        false
    }
}

/// Enables or disables automatic startup when Windows boots
pub fn set_autostart(enable: bool) -> anyhow::Result<()> {
    #[cfg(target_os = "windows")]
    {
        if enable {
            let exe_path = std::env::current_exe()?;
            let exe_str = exe_path.to_string_lossy().to_string();
            let status = Command::new("reg")
                .args([
                    "add",
                    "HKCU\\Software\\Microsoft\\Windows\\CurrentVersion\\Run",
                    "/v",
                    "NextcloudAIO",
                    "/t",
                    "REG_SZ",
                    "/d",
                    &format!("\"{}\"", exe_str),
                    "/f",
                ])
                .status()?;
            if !status.success() {
                anyhow::bail!("Failed to register Nextcloud AIO in Windows Run registry key");
            }
        } else {
            let _ = Command::new("reg")
                .args([
                    "delete",
                    "HKCU\\Software\\Microsoft\\Windows\\CurrentVersion\\Run",
                    "/v",
                    "NextcloudAIO",
                    "/f",
                ])
                .status();
        }
        Ok(())
    }
    #[cfg(not(target_os = "windows"))]
    {
        let _ = enable;
        Ok(())
    }
}
