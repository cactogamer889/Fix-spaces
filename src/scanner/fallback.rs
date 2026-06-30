use super::types::*;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::time::Instant;
use windows::Win32::Storage::FileSystem::{
    FindClose, FindFirstFileW, FindNextFileW, WIN32_FIND_DATAW,
};
use windows::core::PCWSTR;

pub struct FallbackScanner;

impl FallbackScanner {
    pub fn new() -> Self { Self }
}

fn to_wide(s: &Path) -> Vec<u16> {
    use std::os::windows::ffi::OsStrExt;
    s.as_os_str().encode_wide().chain(std::iter::once(0)).collect()
}

impl DiskScanner for FallbackScanner {
    fn scan(
        &self,
        path: &Path,
        cancel_flag: &Arc<AtomicBool>,
        _progress_cb: Option<Box<dyn Fn(usize, usize) + Send>>,
    ) -> Result<ScanResult, ScanError> {
        let start = Instant::now();
        let mut entries: Vec<ScanEntry> = Vec::new();

        // Root entry
        entries.push(ScanEntry {
            path: path.to_path_buf(),
            name: path.file_name().map(|n| n.to_string_lossy().to_string()).unwrap_or_else(|| path.to_string_lossy().to_string()),
            extension: String::new(),
            size: 0,
            is_dir: true,
            parent: None,
            children: Vec::new(),
            depth: 0,
        });

        let mut stack: Vec<(PathBuf, usize, u32)> = vec![(path.to_path_buf(), 0, 0)];
        let mut file_count: usize = 0;
        let mut dir_count: usize = 1;

        while let Some((dir_path, parent_idx, depth)) = stack.pop() {
            if cancel_flag.load(Ordering::Relaxed) {
                return Err(ScanError::Cancelled);
            }

            // Skip deep recursion to avoid slowness
            if depth > 8 { continue; }

            let search = dir_path.join("*");
            let pattern = to_wide(&search);
            let mut fd = WIN32_FIND_DATAW::default();

            let handle = match unsafe { FindFirstFileW(PCWSTR(pattern.as_ptr()), &mut fd) } {
                Ok(h) => h,
                Err(_) => continue,
            };

            loop {
                if cancel_flag.load(Ordering::Relaxed) {
                    unsafe { let _ = FindClose(handle); }
                    return Err(ScanError::Cancelled);
                }

                let name = unsafe {
                    let p = fd.cFileName.as_ptr();
                    let len = (0..260).take_while(|&i| *p.add(i) != 0).count();
                    String::from_utf16_lossy(std::slice::from_raw_parts(p, len))
                };

                if name == "." || name == ".." {
                    if unsafe { FindNextFileW(handle, &mut fd).is_err() } { break; }
                    continue;
                }

                let attrs = fd.dwFileAttributes;
                let is_dir = (attrs & 0x10) != 0;
                let is_reparse = (attrs & 0x400) != 0;
                if is_reparse {
                    if unsafe { FindNextFileW(handle, &mut fd).is_err() } { break; }
                    continue;
                }

                let size = ((fd.nFileSizeHigh as u64) << 32) | (fd.nFileSizeLow as u64);
                let full_path = dir_path.join(&name);
                let ext = if is_dir { String::new() }
                    else { full_path.extension().map(|e| e.to_string_lossy().to_lowercase()).unwrap_or_default() };

                let idx = entries.len();
                entries.push(ScanEntry {
                    path: full_path,
                    name,
                    extension: ext,
                    size,
                    is_dir,
                    parent: Some(parent_idx),
                    children: Vec::new(),
                    depth,
                });
                entries[parent_idx].children.push(idx);

                if is_dir {
                    dir_count += 1;
                    stack.push((entries[idx].path.clone(), idx, depth + 1));
                } else {
                    file_count += 1;
                }

                if unsafe { FindNextFileW(handle, &mut fd).is_err() } { break; }
            }
            unsafe { let _ = FindClose(handle); }
        }

        // Calculate directory sizes bottom-up
        calc_dir_sizes(&mut entries, 0);

        let total_size: u64 = entries[0].size;

        log::info!("Scan: {} entries, {} files, {} dirs, {} bytes", entries.len(), file_count, dir_count, total_size);

        Ok(ScanResult {
            entries,
            root_index: 0,
            total_size,
            scan_duration: start.elapsed(),
            file_count,
            dir_count,
            scan_mode: "FindFirstFileW".to_string(),
        })
    }
}

fn calc_dir_sizes(entries: &mut Vec<ScanEntry>, idx: usize) -> u64 {
    if !entries[idx].is_dir {
        return entries[idx].size;
    }
    let children: Vec<usize> = entries[idx].children.clone();
    let mut total: u64 = 0;
    for child in children {
        total += calc_dir_sizes(entries, child);
    }
    entries[idx].size = total;
    total
}
