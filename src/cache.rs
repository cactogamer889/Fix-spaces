use rusqlite::{Connection, params};
use std::path::Path;
use std::sync::Mutex;
use std::time::SystemTime;

pub struct DiskCache {
    conn: Mutex<Connection>,
}

#[derive(Debug, Clone)]
pub struct CacheEntry {
    pub path: String,
    pub size: u64,
    pub is_dir: bool,
    pub mtime: f64,
}

impl DiskCache {
    pub fn new(db_path: &Path) -> Self {
        let conn = Connection::open(db_path).expect("Failed to open cache DB");
        conn.execute_batch(
            "CREATE TABLE IF NOT EXISTS entries (
                path TEXT PRIMARY KEY,
                size INTEGER NOT NULL,
                is_dir INTEGER NOT NULL,
                mtime REAL NOT NULL,
                cached_at REAL NOT NULL
            );
            CREATE INDEX IF NOT EXISTS idx_parent ON entries(path);",
        ).expect("Failed to init cache DB");

        Self {
            conn: Mutex::new(conn),
        }
    }

    pub fn get(&self, path: &str) -> Option<CacheEntry> {
        let conn = self.conn.lock().unwrap();
        let mut stmt = conn
            .prepare("SELECT size, is_dir, mtime FROM entries WHERE path = ?1")
            .ok()?;
        let mut rows = stmt.query_map(params![path], |row| {
            Ok(CacheEntry {
                path: path.to_string(),
                size: row.get(0)?,
                is_dir: row.get::<_, bool>(1)?,
                mtime: row.get(2)?,
            })
        })
        .ok()?;
        rows.next()?.ok()
    }

    pub fn is_valid(&self, path: &str) -> bool {
        let entry = match self.get(path) {
            Some(e) => e,
            None => return false,
        };

        let current_mtime = std::fs::metadata(path)
            .ok()
            .and_then(|m| m.modified().ok())
            .and_then(|t| t.duration_since(SystemTime::UNIX_EPOCH).ok())
            .map(|d| d.as_secs_f64())
            .unwrap_or(0.0);

        (current_mtime - entry.mtime).abs() < 0.01
    }

    pub fn set(&self, entry: &CacheEntry) {
        let conn = self.conn.lock().unwrap();
        let now = SystemTime::now()
            .duration_since(SystemTime::UNIX_EPOCH)
            .map(|d| d.as_secs_f64())
            .unwrap_or(0.0);

        conn.execute(
            "INSERT OR REPLACE INTO entries (path, size, is_dir, mtime, cached_at) VALUES (?1, ?2, ?3, ?4, ?5)",
            params![entry.path, entry.size, entry.is_dir, entry.mtime, now],
        )
        .ok();
    }

    pub fn clear(&self) {
        let conn = self.conn.lock().unwrap();
        conn.execute("DELETE FROM entries", []).ok();
    }

    pub fn stats(&self) -> (usize, u64) {
        let conn = self.conn.lock().unwrap();
        let count: usize = conn
            .query_row("SELECT COUNT(*) FROM entries", [], |row| row.get(0))
            .unwrap_or(0);
        let total: u64 = conn
            .query_row("SELECT COALESCE(SUM(size), 0) FROM entries", [], |row| row.get(0))
            .unwrap_or(0);
        (count, total)
    }
}
