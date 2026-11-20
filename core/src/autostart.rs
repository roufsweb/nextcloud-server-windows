#[cfg(target_os = "windows")]
use std::os::windows::process::CommandExt;
#[cfg(target_os = "windows")]
use std::process::Command;

#[cfg(target_os = "windows")]
const CREATE_NO_WINDOW: u32 = 0x08000000;

/// Checks if Nextcloud AIO is registered in Windows Startup (HKCU Run key)
pub fn is_autostart_enabled() -> bool {
    #[cfg(target_os = "windows")]
    {
        let mut cmd = Command::new("reg");
        cmd.args(["query", "HKCU\\Software\\Microsoft\\Windows\\CurrentVersion\\Run", "/v", "NextcloudAIO"])
           .creation_flags(CREATE_NO_WINDOW);
        if let Ok(out) = cmd.output() {
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
            let mut cmd = Command::new("reg");
            cmd.args([
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
            .creation_flags(CREATE_NO_WINDOW);
            let status = cmd.status()?;
            if !status.success() {
                anyhow::bail!("Failed to register Nextcloud AIO in Windows Run registry key");
            }
        } else {
            let mut cmd = Command::new("reg");
            cmd.args([
                "delete",
                "HKCU\\Software\\Microsoft\\Windows\\CurrentVersion\\Run",
                "/v",
                "NextcloudAIO",
                "/f",
            ])
            .creation_flags(CREATE_NO_WINDOW);
            let _ = cmd.status();
        }
        Ok(())
    }
    #[cfg(not(target_os = "windows"))]
    {
        let _ = enable;
        Ok(())
    }
}
