pub mod types;
pub mod fallback;
pub mod mft;

pub use types::*;

use std::path::Path;
use std::sync::atomic::AtomicBool;
use std::sync::Arc;

pub fn is_admin() -> bool {
    use std::process::Command;
    Command::new("net")
        .args(["session"])
        .output()
        .map(|o| o.status.success())
        .unwrap_or(false)
}

pub fn get_drives() -> Vec<String> {
    use windows::Win32::Storage::FileSystem::GetLogicalDrives;

    let mut drives = Vec::new();
    unsafe {
        let bitmask = GetLogicalDrives();
        for i in 0..26 {
            if bitmask & (1 << i) != 0 {
                let letter = (b'A' + i as u8) as char;
                drives.push(format!("{}:\\", letter));
            }
        }
    }
    drives
}

pub fn get_fs_type(drive: &str) -> Option<String> {
    use windows::Win32::Storage::FileSystem::GetVolumeInformationW;
    use windows::core::PCWSTR;

    let wide: Vec<u16> = drive.encode_utf16().chain(std::iter::once(0)).collect();
    let mut fs_name = [0u16; 256];

    unsafe {
        let result = GetVolumeInformationW(
            PCWSTR(wide.as_ptr()),
            None,
            None,
            None,
            None,
            Some(&mut fs_name),
        );

        if result.is_ok() {
            let len = fs_name.iter().take_while(|&&c| c != 0).count();
            if len > 0 {
                return Some(String::from_utf16_lossy(&fs_name[..len]));
            }
        }
    }
    None
}

pub fn create_scanner(path: &Path) -> Box<dyn DiskScanner> {
    let is_ntfs = get_fs_type(&path.to_string_lossy())
        .map(|fs| fs.eq_ignore_ascii_case("NTFS"))
        .unwrap_or(false);

    if is_admin() && is_ntfs {
        Box::new(mft::MftScanner::new())
    } else {
        Box::new(fallback::FallbackScanner::new())
    }
}
