use egui::Color32;

pub fn size_color(size: u64, max_size: u64) -> Color32 {
    if max_size == 0 || size == 0 { return Color32::from_rgb(100, 100, 110); }
    // Use log scale for better distribution
    let log_size = (size as f64).ln();
    let log_max = (max_size as f64).ln();
    let t = if log_max > 0.0 { (log_size / log_max).clamp(0.0, 1.0) } else { 0.0 };

    if t < 0.2 {
        let u = t / 0.2;
        lerp(Color32::from_rgb(50, 180, 80), Color32::from_rgb(80, 190, 60), u)
    } else if t < 0.4 {
        let u = (t - 0.2) / 0.2;
        lerp(Color32::from_rgb(80, 190, 60), Color32::from_rgb(180, 190, 40), u)
    } else if t < 0.6 {
        let u = (t - 0.4) / 0.2;
        lerp(Color32::from_rgb(180, 190, 40), Color32::from_rgb(220, 160, 30), u)
    } else if t < 0.8 {
        let u = (t - 0.6) / 0.2;
        lerp(Color32::from_rgb(220, 160, 30), Color32::from_rgb(220, 90, 30), u)
    } else {
        let u = (t - 0.8) / 0.2;
        lerp(Color32::from_rgb(220, 90, 30), Color32::from_rgb(210, 40, 40), u)
    }
}

pub fn ext_color(extension: &str) -> Color32 {
    match extension {
        "mp4" | "mkv" | "avi" | "mov" | "wmv" | "flv" | "webm" | "m4v" => Color32::from_rgb(70, 130, 200),
        "mp3" | "wav" | "flac" | "aac" | "ogg" | "wma" | "m4a" => Color32::from_rgb(150, 80, 200),
        "jpg" | "jpeg" | "png" | "gif" | "bmp" | "svg" | "webp" | "ico" | "tiff" | "psd" | "raw" | "cr2" | "nef" => Color32::from_rgb(80, 180, 80),
        "pdf" | "doc" | "docx" | "xls" | "xlsx" | "ppt" | "pptx" | "txt" | "rtf" | "csv" | "odt" => Color32::from_rgb(220, 150, 50),
        "zip" | "rar" | "7z" | "tar" | "gz" | "bz2" | "xz" | "iso" | "cab" => Color32::from_rgb(200, 80, 80),
        "rs" | "py" | "js" | "ts" | "jsx" | "tsx" | "java" | "c" | "cpp" | "h" | "cs" | "go" | "rb" | "php" | "html" | "css" | "json" | "xml" | "yaml" | "sql" => Color32::from_rgb(60, 180, 180),
        "exe" | "msi" | "dll" | "sys" | "so" | "dylib" | "app" => Color32::from_rgb(220, 100, 160),
        "db" | "sqlite" | "sqlite3" | "mdb" => Color32::from_rgb(210, 200, 60),
        "ttf" | "otf" | "woff" | "woff2" | "eot" => Color32::from_rgb(160, 120, 60),
        "stl" | "obj" | "fbx" | "blend" => Color32::from_rgb(120, 200, 60),
        "vmdk" | "vhd" | "vhdx" | "img" => Color32::from_rgb(40, 80, 160),
        _ => Color32::from_rgb(120, 120, 130),
    }
}

fn lerp(a: Color32, b: Color32, t: f64) -> Color32 {
    let t = t as f32;
    let r = (a.r() as f32 + (b.r() as f32 - a.r() as f32) * t) as u8;
    let g = (a.g() as f32 + (b.g() as f32 - a.g() as f32) * t) as u8;
    let bl = (a.b() as f32 + (b.b() as f32 - a.b() as f32) * t) as u8;
    Color32::from_rgb(r, g, bl)
}

pub fn format_size(bytes: u64) -> String {
    let b = bytes as f64;
    const KB: f64 = 1024.0;
    const MB: f64 = KB * 1024.0;
    const GB: f64 = MB * 1024.0;
    const TB: f64 = GB * 1024.0;
    if b >= TB { format!("{:.1} TB", b / TB) }
    else if b >= GB { format!("{:.1} GB", b / GB) }
    else if b >= MB { format!("{:.1} MB", b / MB) }
    else if b >= KB { format!("{:.1} KB", b / KB) }
    else { format!("{} B", bytes) }
}

pub fn format_percent(value: u64, total: u64) -> String {
    if total == 0 { return "0%".to_string(); }
    let p = (value as f64 / total as f64) * 100.0;
    if p >= 10.0 { format!("{:.0}%", p) }
    else if p >= 1.0 { format!("{:.1}%", p) }
    else if p >= 0.01 { format!("{:.2}%", p) }
    else { format!("{:.3}%", p) }
}
