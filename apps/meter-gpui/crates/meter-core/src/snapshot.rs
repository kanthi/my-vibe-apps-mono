use rusqlite::{Connection, OpenFlags};
use std::path::Path;
use tempfile::TempDir;

/// A connection to a private copy of a database and its -wal/-shm files, so a reader can never
/// interfere with the app that owns the live file (locks, checkpoints, -shm writes).
/// The copy is deleted when this value is dropped.
pub struct SqliteSnapshot {
    pub conn: Connection,
    _dir: TempDir,
}

impl SqliteSnapshot {
    pub fn open(source: &Path) -> Result<Self, Box<dyn std::error::Error + Send + Sync>> {
        let dir = tempfile::Builder::new().prefix("meter-snap-").tempdir()?;
        let name = source.file_name().ok_or("no file name")?.to_string_lossy().to_string();
        for suffix in ["", "-wal", "-shm"] {
            let from = source.with_file_name(format!("{name}{suffix}"));
            if from.exists() {
                std::fs::copy(&from, dir.path().join(format!("{name}{suffix}")))?;
            }
        }
        // Read-write on the copy lets SQLite recover the copied journal; the original is never opened.
        let conn = Connection::open_with_flags(dir.path().join(&name), OpenFlags::SQLITE_OPEN_READ_WRITE)?;
        Ok(SqliteSnapshot { conn, _dir: dir })
    }
}
