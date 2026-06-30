use super::types::*;
use std::path::Path;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::time::Instant;

pub struct MftScanner;

impl MftScanner {
    pub fn new() -> Self {
        Self
    }
}

impl DiskScanner for MftScanner {
    fn scan(
        &self,
        path: &Path,
        cancel_flag: &Arc<AtomicBool>,
        progress_cb: Option<Box<dyn Fn(usize, usize) + Send>>,
    ) -> Result<ScanResult, ScanError> {
        if cancel_flag.load(Ordering::Relaxed) {
            return Err(ScanError::Cancelled);
        }

        let fallback = super::fallback::FallbackScanner::new();
        let mut result = fallback.scan(path, cancel_flag, progress_cb)?;
        result.scan_mode = "MFT (via fallback)".to_string();
        Ok(result)
    }
}
