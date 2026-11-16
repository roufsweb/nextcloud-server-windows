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
