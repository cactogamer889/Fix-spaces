use std::path::PathBuf;
use std::time::Duration;

#[derive(Debug, Clone)]
pub struct ScanEntry {
    pub path: PathBuf,
    pub name: String,
    pub extension: String,
    pub size: u64,
    pub is_dir: bool,
    pub parent: Option<usize>,
    pub children: Vec<usize>,
    pub depth: u32,
}

#[derive(Debug, Clone)]
pub struct ScanResult {
    pub entries: Vec<ScanEntry>,
    pub root_index: usize,
    pub total_size: u64,
    pub scan_duration: Duration,
    pub file_count: usize,
    pub dir_count: usize,
    pub scan_mode: String,
}

impl ScanResult {
    pub fn empty() -> Self {
        Self {
            entries: Vec::new(),
            root_index: 0,
            total_size: 0,
            scan_duration: Duration::ZERO,
            file_count: 0,
            dir_count: 0,
            scan_mode: String::new(),
        }
    }
}

#[derive(Debug)]
pub enum ScanError {
    AccessDenied(String),
    IoError(std::io::Error),
    Cancelled,
}

impl std::fmt::Display for ScanError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ScanError::AccessDenied(path) => write!(f, "Access denied: {}", path),
            ScanError::IoError(e) => write!(f, "IO error: {}", e),
            ScanError::Cancelled => write!(f, "Scan cancelled"),
        }
    }
}

impl From<std::io::Error> for ScanError {
    fn from(e: std::io::Error) -> Self {
        ScanError::IoError(e)
    }
}

pub trait DiskScanner: Send {
    fn scan(
        &self,
        path: &std::path::Path,
        cancel_flag: &std::sync::Arc<std::sync::atomic::AtomicBool>,
        progress_cb: Option<Box<dyn Fn(usize, usize) + Send>>,
    ) -> Result<ScanResult, ScanError>;
}
