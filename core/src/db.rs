use anyhow::Result;
use rusqlite::Connection;
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::Path;
use std::time::SystemTime;

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct DatabaseStatus {
    pub engine: String,
    pub path: String,
    pub exists: bool,
    pub file_size_bytes: u64,
    pub wal_size_bytes: u64,
    pub journal_mode: String,
    pub integrity: String,
    pub page_size: u32,
    pub page_count: u64,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct OptimizeResult {
    pub status: String,
    pub bytes_freed: i64,
    pub new_size_bytes: u64,
    pub wal_size_bytes: u64,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct BackupResult {
    pub status: String,
    pub backup_file: String,
    pub size_bytes: u64,
    pub timestamp: u64,
}

/// Applies high-performance WAL and memory PRAGMAs to Nextcloud SQLite/libSQL database
pub fn apply_wal_optimizations(db_path: &Path) -> Result<()> {
    if let Some(parent) = db_path.parent() {
        fs::create_dir_all(parent)?;
    }

    let conn = Connection::open(db_path)?;
    conn.execute_batch(
        "
        PRAGMA journal_mode = WAL;
        PRAGMA synchronous = NORMAL;
        PRAGMA busy_timeout = 5000;
        PRAGMA cache_size = -64000;
        PRAGMA mmap_size = 268435456;
        PRAGMA temp_store = MEMORY;
        ",
    )?;

    Ok(())
}

/// Retrieves real-time health, size, and integrity status of the database
pub fn get_status(db_path: &Path, engine: &str) -> DatabaseStatus {
    if !db_path.exists() {
        return DatabaseStatus {
            engine: engine.to_string(),
            path: db_path.display().to_string(),
            exists: false,
            file_size_bytes: 0,
            wal_size_bytes: 0,
            journal_mode: "N/A".to_string(),
            integrity: "Database file not found (uninitialized)".to_string(),
            page_size: 0,
            page_count: 0,
        };
    }

    let file_size = fs::metadata(db_path).map(|m| m.len()).unwrap_or(0);
    let wal_path = db_path.with_extension("db-wal");
    let wal_size = fs::metadata(&wal_path).map(|m| m.len()).unwrap_or(0);

    let (journal_mode, integrity, page_size, page_count) = if let Ok(conn) = Connection::open(db_path) {
        let j_mode: String = conn
            .query_row("PRAGMA journal_mode;", [], |r| r.get(0))
            .unwrap_or_else(|_| "UNKNOWN".into());
        let integ: String = conn
            .query_row("PRAGMA integrity_check(1);", [], |r| r.get(0))
            .unwrap_or_else(|_| "UNKNOWN".into());
        let p_size: u32 = conn.query_row("PRAGMA page_size;", [], |r| r.get(0)).unwrap_or(4096);
        let p_count: u64 = conn.query_row("PRAGMA page_count;", [], |r| r.get(0)).unwrap_or(0);
        (j_mode, integ, p_size, p_count)
    } else {
        ("UNKNOWN".into(), "Locked or unreadable".into(), 0, 0)
    };

    DatabaseStatus {
        engine: engine.to_string(),
        path: db_path.display().to_string(),
        exists: true,
        file_size_bytes: file_size,
        wal_size_bytes: wal_size,
        journal_mode,
        integrity,
        page_size,
        page_count,
    }
}

/// Truncates WAL log and runs online VACUUM to reclaim free disk space
pub fn optimize(db_path: &Path) -> Result<OptimizeResult> {
    if !db_path.exists() {
        apply_wal_optimizations(db_path)?;
    }

    let old_size = fs::metadata(db_path).map(|m| m.len()).unwrap_or(0);
    let wal_path = db_path.with_extension("db-wal");
    let old_wal = fs::metadata(&wal_path).map(|m| m.len()).unwrap_or(0);

    let conn = Connection::open(db_path)?;
    conn.execute_batch(
        "
        PRAGMA wal_checkpoint(TRUNCATE);
        VACUUM;
        ",
    )?;

    let new_size = fs::metadata(db_path).map(|m| m.len()).unwrap_or(0);
    let new_wal = fs::metadata(&wal_path).map(|m| m.len()).unwrap_or(0);
    let bytes_freed = (old_size + old_wal) as i64 - (new_size + new_wal) as i64;

    Ok(OptimizeResult {
        status: "optimized".into(),
        bytes_freed: bytes_freed.max(0),
        new_size_bytes: new_size,
        wal_size_bytes: new_wal,
    })
}

/// Performs non-blocking atomic point-in-time backup using SQLite's VACUUM INTO
pub fn backup(db_path: &Path, backup_dir: &Path) -> Result<BackupResult> {
    if !db_path.exists() {
        apply_wal_optimizations(db_path)?;
    }

    fs::create_dir_all(backup_dir)?;
    let timestamp = SystemTime::now()
        .duration_since(SystemTime::UNIX_EPOCH)?
        .as_secs();
    let backup_filename = format!("nextcloud_db_backup_{}.sqlite", timestamp);
    let backup_path = backup_dir.join(&backup_filename);

    let conn = Connection::open(db_path)?;
    let escaped_path = backup_path.to_string_lossy().replace('\'', "''");
    conn.execute(&format!("VACUUM INTO '{}';", escaped_path), [])?;

    let backup_size = fs::metadata(&backup_path)?.len();

    Ok(BackupResult {
        status: "success".into(),
        backup_file: backup_filename,
        size_bytes: backup_size,
        timestamp,
    })
}
