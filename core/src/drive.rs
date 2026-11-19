use std::path::Path;
use std::process::Command;
use anyhow::{Context, Result};
use colored::*;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct AvailableDrive {
    pub letter: String,
    pub path: String,
    pub total_gb: u64,
    pub free_gb: u64,
    pub used_gb: u64,
    pub is_mounted: bool,
}

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
            .arg(&*path_str)
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
#[allow(dead_code)]
pub fn verify_drive_read_write<P: AsRef<Path>>(drive_path: P) -> Result<bool> {
    let test_file = drive_path.as_ref().join(".nc_write_test.tmp");
    if let Err(e) = std::fs::write(&test_file, b"nextcloud_rw_verify") {
        eprintln!("Write test failed: {}", e);
        return Ok(false);
    }
    let _ = std::fs::remove_file(test_file);
    Ok(true)
}

/// Opens a path in Windows Explorer or Linux file manager
pub fn open_in_file_manager<P: AsRef<Path>>(path: P) -> Result<()> {
    let p = path.as_ref();
    if !p.exists() {
        let _ = std::fs::create_dir_all(p);
    }

    #[cfg(target_os = "windows")]
    {
        use std::os::windows::process::CommandExt;
        const CREATE_NO_WINDOW: u32 = 0x08000000;
        let mut cmd = Command::new("explorer");
        cmd.arg(p.as_os_str())
           .creation_flags(CREATE_NO_WINDOW);
        cmd.spawn()
            .with_context(|| format!("Failed to launch explorer for {:?}", p))?;
    }

    #[cfg(not(target_os = "windows"))]
    {
        Command::new("xdg-open")
            .arg(p.as_os_str())
            .spawn()
            .with_context(|| format!("Failed to open file manager for {:?}", p))?;
    }

    Ok(())
}

/// Native Windows folder selection dialog
pub fn pick_folder_dialog(description: &str) -> Option<String> {
    #[cfg(target_os = "windows")]
    {
        use std::os::windows::process::CommandExt;
        const CREATE_NO_WINDOW: u32 = 0x08000000;
        let escaped_desc = description.replace('\'', "''").replace('\"', "\\\"");
        let ps_cmd = format!(
            "Add-Type -AssemblyName System.Windows.Forms; $f = New-Object System.Windows.Forms.FolderBrowserDialog; $f.Description = '{}'; $f.ShowNewFolderButton = $true; if ($f.ShowDialog() -eq [System.Windows.Forms.DialogResult]::OK) {{ [Console]::Out.Write($f.SelectedPath) }}",
            escaped_desc
        );
        let mut cmd = Command::new("powershell");
        cmd.args(["-NoProfile", "-NonInteractive", "-Command", &ps_cmd])
           .creation_flags(CREATE_NO_WINDOW);
        if let Ok(output) = cmd.output() {
            if output.status.success() {
                let res = String::from_utf8_lossy(&output.stdout).trim().to_string();
                if !res.is_empty() {
                    return Some(res);
                }
            }
        }
    }
    let _ = description;
    None
}

/// Detects available physical drives on Windows
#[cfg(target_os = "windows")]
pub fn get_available_windows_drives(mounted_paths: &[std::path::PathBuf]) -> Vec<AvailableDrive> {
    extern "system" {
        fn GetLogicalDrives() -> u32;
    }

    let mask = unsafe { GetLogicalDrives() };
    let mut list = Vec::new();

    for i in 0..26 {
        if (mask & (1 << i)) != 0 {
            let letter = (b'A' + i as u8) as char;
            let path_str = format!("{}:\\", letter);
            let path_buf = std::path::PathBuf::from(&path_str);
            let (tot, used, free) = get_disk_space(&path_buf);
            
            // Filter out virtual empty drives (e.g. unmounted CD-ROMs with 0 bytes)
            if tot > 0 {
                let is_mounted = mounted_paths.iter().any(|m| {
                    m == &path_buf || m.to_string_lossy().to_uppercase().starts_with(&path_str.to_uppercase())
                });

                list.push(AvailableDrive {
                    letter: format!("{}:", letter),
                    path: path_str,
                    total_gb: tot,
                    free_gb: free,
                    used_gb: used,
                    is_mounted,
                });
            }
        }
    }
    list
}

#[cfg(not(target_os = "windows"))]
pub fn get_available_windows_drives(_mounted_paths: &[std::path::PathBuf]) -> Vec<AvailableDrive> {
    Vec::new()
}

/// Retrieves disk storage metrics (Total GB, Used GB, Free GB)
#[cfg(not(target_os = "windows"))]
pub fn get_disk_space<P: AsRef<Path>>(path: P) -> (u64, u64, u64) {
    use std::ffi::CString;
    use std::mem::MaybeUninit;
    let path_str = path.as_ref().to_str().unwrap_or("/");
    if let Ok(c_path) = CString::new(path_str) {
        unsafe {
            let mut stat: MaybeUninit<libc::statvfs> = MaybeUninit::uninit();
            if libc::statvfs(c_path.as_ptr(), stat.as_mut_ptr()) == 0 {
                let stat = stat.assume_init();
                let block_size = stat.f_frsize as u64;
                let total_bytes = stat.f_blocks as u64 * block_size;
                let free_bytes = stat.f_bavail as u64 * block_size;
                let used_bytes = total_bytes.saturating_sub(free_bytes);
                return (
                    total_bytes / (1024 * 1024 * 1024),
                    used_bytes / (1024 * 1024 * 1024),
                    free_bytes / (1024 * 1024 * 1024),
                );
            }
        }
    }
    (500, 150, 350)
}

/// Retrieves disk storage metrics on Windows via GetDiskFreeSpaceExW (kernel32.dll)
#[cfg(target_os = "windows")]
pub fn get_disk_space<P: AsRef<Path>>(path: P) -> (u64, u64, u64) {
    use std::os::windows::ffi::OsStrExt;
    let wide_path: Vec<u16> = path.as_ref().as_os_str().encode_wide().chain(std::iter::once(0)).collect();
    let mut free_bytes_available: u64 = 0;
    let mut total_bytes: u64 = 0;
    let mut total_free_bytes: u64 = 0;

    extern "system" {
        fn GetDiskFreeSpaceExW(
            lpDirectoryName: *const u16,
            lpFreeBytesAvailableToCaller: *mut u64,
            lpTotalNumberOfBytes: *mut u64,
            lpTotalNumberOfFreeBytes: *mut u64,
        ) -> i32;
    }

    unsafe {
        if GetDiskFreeSpaceExW(
            wide_path.as_ptr(),
            &mut free_bytes_available,
            &mut total_bytes,
            &mut total_free_bytes,
        ) != 0 {
            let total_gb = total_bytes / (1024 * 1024 * 1024);
            let free_gb = total_free_bytes / (1024 * 1024 * 1024);
            let used_gb = total_gb.saturating_sub(free_gb);
            return (total_gb, used_gb, free_gb);
        }
    }
    (1000, 350, 650)
}
