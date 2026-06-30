use eframe::egui;
use egui::{Color32, Vec2};
use crate::scanner::{self, ScanResult};
use crate::treemap::{self, TreemapRect};
use crate::treemap::renderer::{size_color, format_size, format_percent};
use crate::ui::{tree_view, extension_list, toolbar, themes};
use std::path::PathBuf;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, mpsc};

pub struct DiskVizApp {
    scan_result: Option<ScanResult>,
    toolbar: toolbar::Toolbar,
    tree_view: tree_view::TreeView,
    extension_list: extension_list::ExtensionList,
    treemap_rects: Vec<TreemapRect>,
    selected_idx: Option<usize>,
    treemap_hovered: Option<usize>,
    status: String,
    cancel_flag: Arc<AtomicBool>,
    custom_path: String,
    rx: Option<mpsc::Receiver<Result<ScanResult, String>>>,
    scan_start: Option<std::time::Instant>,
    theme_index: usize,
}

impl Default for DiskVizApp {
    fn default() -> Self {
        Self {
            scan_result: None,
            toolbar: toolbar::Toolbar::default(),
            tree_view: tree_view::TreeView::default(),
            extension_list: extension_list::ExtensionList,
            treemap_rects: Vec::new(),
            selected_idx: None,
            treemap_hovered: None,
            status: "Ready.".to_string(),
            cancel_flag: Arc::new(AtomicBool::new(false)),
            custom_path: String::new(),
            rx: None,
            scan_start: None,
            theme_index: 0,
        }
    }
}

impl eframe::App for DiskVizApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        ctx.request_repaint();

        let theme = themes::get_theme(self.theme_index);

        // Poll scan
        if let Some(rx) = &self.rx {
            match rx.try_recv() {
                Ok(Ok(mut result)) => {
                    self.toolbar.is_scanning = false;
                    let t = self.scan_start.map(|s| s.elapsed().as_secs_f64()).unwrap_or(0.0);
                    result.scan_duration = std::time::Duration::from_secs_f64(t);
                    self.status = format!("Done: {} files, {} dirs in {:.1}s [{}]", result.file_count, result.dir_count, t, result.scan_mode);
                    self.build_treemap(&result);
                    self.scan_result = Some(result);
                    self.rx = None;
                }
                Ok(Err(e)) => {
                    self.toolbar.is_scanning = false;
                    self.status = format!("Error: {}", e);
                    self.rx = None;
                }
                Err(mpsc::TryRecvError::Empty) => {
                    if self.toolbar.is_scanning {
                        let t = self.scan_start.map(|s| s.elapsed().as_secs_f64()).unwrap_or(0.0);
                        self.status = format!("Scanning... {:.0}s", t);
                    }
                }
                Err(_) => {
                    self.toolbar.is_scanning = false;
                    self.status = "Thread crashed.".to_string();
                    self.rx = None;
                }
            }
        }

        // --- UI ---
        egui::TopBottomPanel::top("tb")
            .frame(egui::Frame::new().inner_margin(6.0).fill(theme.panel))
            .show(ctx, |ui| {
                let action = self.toolbar.show(ui, &mut self.custom_path, theme);
                match action {
                    toolbar::ToolbarAction::StartScan => self.start_scan(),
                    toolbar::ToolbarAction::StopScan => self.stop_scan(),
                    toolbar::ToolbarAction::ExportHtml => self.export_html(),
                    toolbar::ToolbarAction::ThemeChanged(i) => self.theme_index = i,
                    toolbar::ToolbarAction::None => {}
                }
            });

        egui::TopBottomPanel::bottom("st")
            .frame(egui::Frame::new().inner_margin(5.0).fill(theme.panel))
            .show(ctx, |ui| {
                ui.horizontal(|ui| {
                    ui.label(egui::RichText::new(&self.status).color(theme.text_dim).size(10.0));
                    if self.toolbar.is_scanning { ui.spinner(); }
                });
            });

        egui::SidePanel::left("tree")
            .resizable(true).default_width(440.0).min_width(260.0)
            .frame(egui::Frame::new().inner_margin(4.0).fill(theme.panel))
            .show(ctx, |ui| {
                ui.label(egui::RichText::new("Files & Folders").color(theme.accent).size(13.0).strong());
                ui.add_space(3.0);
                if let Some(ref sr) = self.scan_result {
                    if let Some(idx) = self.tree_view.show(ui, &sr.entries, sr.root_index, sr.total_size, self.selected_idx, theme) {
                        self.selected_idx = Some(idx);
                        self.treemap_hovered = Some(idx);
                    }
                } else if self.toolbar.is_scanning {
                    ui.centered_and_justified(|ui| { ui.spinner(); });
                } else {
                    ui.centered_and_justified(|ui| {
                        ui.label(egui::RichText::new("Select drive and Scan").color(theme.text_dim));
                    });
                }
            });

        egui::CentralPanel::default()
            .frame(egui::Frame::new().inner_margin(6.0).fill(theme.bg))
            .show(ctx, |ui| {
                // Treemap header
                ui.horizontal(|ui| {
                    ui.label(egui::RichText::new("Treemap").color(theme.accent).size(13.0).strong());
                    if let Some(ref sr) = self.scan_result {
                        ui.separator();
                        ui.label(egui::RichText::new(format!("{} files | {} dirs | {} | {}", sr.file_count, sr.dir_count, format_size(sr.total_size), sr.scan_mode)).color(theme.text_dim).size(10.0));
                    }
                });
                ui.add_space(3.0);

                let avail = ui.available_size();
                let th = (avail.y - 160.0).max(60.0);
                let area = egui::Rect::from_min_size(ui.cursor().min, Vec2::new(avail.x, th));

                if !self.treemap_rects.is_empty() && area.width() > 10.0 && area.height() > 10.0 {
                    self.render_treemap(ui, area, theme);
                }

                ui.add_space(4.0);
                ui.separator();
                ui.add_space(3.0);

                ui.label(egui::RichText::new("Extensions").color(theme.accent).size(13.0).strong());
                ui.add_space(3.0);
                if let Some(ref sr) = self.scan_result {
                    self.extension_list.show(ui, &sr.entries, sr.total_size, theme);
                }
            });
    }
}

impl DiskVizApp {
    pub fn new(_cc: &eframe::CreationContext<'_>) -> Self { Self::default() }

    fn render_treemap(&mut self, ui: &mut egui::Ui, area: egui::Rect, theme: &themes::Theme) {
        let _r = ui.allocate_rect(area, egui::Sense::click());
        let painter = ui.painter();
        painter.rect_filled(area, 0.0, theme.surface);

        let total = self.scan_result.as_ref().map(|sr| sr.total_size).unwrap_or(1);
        // Max size among visible treemap items (not global)
        let max_size = self.treemap_rects.iter()
            .map(|tmr| tmr.size)
            .max()
            .unwrap_or(1);

        let mut new_hovered = None;
        let mut clicked = None;

        for tmr in &self.treemap_rects {
            if tmr.rect.width() < 2.0 || tmr.rect.height() < 2.0 { continue; }
            if let Some(ref sr) = self.scan_result {
                let e = &sr.entries[tmr.index];
                let base = if e.is_dir { size_color(e.size, max_size) } else { crate::treemap::renderer::ext_color(&e.extension) };

                let is_sel = Some(tmr.index) == self.selected_idx;
                let is_hov = Some(tmr.index) == self.treemap_hovered;

                let color = if is_sel { brighten(base, 60) } else if is_hov { brighten(base, 30) } else { base };
                let inner = tmr.rect.shrink(1.0);

                // shadow
                painter.rect_filled(egui::Rect::from_min_size(inner.min + Vec2::new(2.0, 2.0), inner.size()), egui::CornerRadius::same(3), Color32::from_rgba_premultiplied(0, 0, 0, 40));

                // gradient
                let top = brighten(color, 20);
                paint_gradient(painter, inner, egui::CornerRadius::same(3), top, color);

                // border
                let stroke = if is_sel { egui::Stroke::new(2.0, Color32::WHITE) }
                    else if is_hov { egui::Stroke::new(1.5, Color32::from_rgba_premultiplied(200, 220, 255, 160)) }
                    else { egui::Stroke::new(0.5, Color32::from_rgba_premultiplied(0, 0, 0, 50)) };
                painter.rect_stroke(inner, egui::CornerRadius::same(3), stroke, egui::StrokeKind::Inside);

                // label
                if inner.width() > 35.0 && inner.height() > 14.0 {
                    let br = 0.299 * color.r() as f32 + 0.587 * color.g() as f32 + 0.114 * color.b() as f32;
                    let tc = if is_sel || is_hov { Color32::WHITE } else if br > 130.0 { Color32::BLACK } else { Color32::from_rgb(230, 230, 230) };
                    let fs = if inner.width() > 70.0 && inner.height() > 24.0 { 10.0 } else { 8.0 };
                    let label = if e.name.len() > 18 && inner.width() < 100.0 { format!("{}...", &e.name[..15.min(e.name.len())]) } else { e.name.clone() };
                    painter.text(inner.center(), egui::Align2::CENTER_CENTER, &label, egui::FontId::proportional(fs), tc);
                }

                if ui.rect_contains_pointer(inner) {
                    new_hovered = Some(tmr.index);
                    if ui.input(|i| i.pointer.any_click()) { clicked = Some(tmr.index); }
                    if ui.input(|i| i.pointer.secondary_clicked()) {
                        self.selected_idx = Some(tmr.index);
                    }
                }
            }
        }

        self.treemap_hovered = new_hovered;
        if let Some(idx) = clicked { self.selected_idx = Some(idx); }
    }

    fn start_scan(&mut self) {
        let path_str = if !self.custom_path.is_empty() { self.custom_path.clone() } else { self.toolbar.selected_drive.clone() };
        let path = PathBuf::from(&path_str);
        if !path.exists() { self.status = format!("Not found: {}", path_str); return; }

        self.toolbar.is_scanning = true;
        self.status = format!("Scanning {}...", path_str);
        self.scan_result = None;
        self.treemap_rects.clear();
        self.selected_idx = None;
        self.scan_start = Some(std::time::Instant::now());

        self.cancel_flag.store(false, Ordering::Relaxed);
        let cf = self.cancel_flag.clone();
        let scanner = scanner::create_scanner(&path);
        let (tx, rx) = mpsc::channel();
        self.rx = Some(rx);

        std::thread::spawn(move || {
            match scanner.scan(&path, &cf, None) {
                Ok(r) => { let _ = tx.send(Ok(r)); }
                Err(e) => { let _ = tx.send(Err(format!("{}", e))); }
            }
        });
    }

    fn stop_scan(&mut self) {
        self.cancel_flag.store(true, Ordering::Relaxed);
        self.toolbar.is_scanning = false;
        self.status = "Cancelled.".to_string();
    }

    fn build_treemap(&mut self, result: &ScanResult) {
        let items: Vec<(usize, u64, String)> = result.entries[result.root_index].children.iter()
            .map(|&i| { let e = &result.entries[i]; (i, e.size, e.name.clone()) }).collect();
        let area = egui::Rect::from_min_size(egui::Pos2::ZERO, Vec2::new(1000.0, 600.0));
        self.treemap_rects = treemap::squarify(&items, area);
    }

    fn export_html(&self) {
        if let Some(ref sr) = self.scan_result {
            let mut h = String::from("<!DOCTYPE html><html><head><meta charset='utf-8'><title>Fix Space Report</title><style>body{font-family:Segoe UI,sans-serif;margin:20px;background:#1a1a2e;color:#eee}h1{color:#00d4ff}table{border-collapse:collapse;width:100%}th,td{padding:8px;text-align:left;border-bottom:1px solid #333}th{background:#16213e;color:#00d4ff}.s{color:#ff6b6b}.e{color:#4ecdc4}.b{height:8px;background:linear-gradient(90deg,#4ecdc4,#ff6b6b);border-radius:4px}</style></head><body>");
            h.push_str(&format!("<h1>Fix Space Report</h1><p>{:.1}s | {} files | {} dirs | {}</p>", sr.scan_duration.as_secs_f64(), sr.file_count, sr.dir_count, format_size(sr.total_size)));
            h.push_str("<table><tr><th>Name</th><th>Size</th><th>%</th><th>Type</th></tr>");
            let mut items: Vec<_> = sr.entries[sr.root_index].children.iter().map(|&i| &sr.entries[i]).collect();
            items.sort_by(|a, b| b.size.cmp(&a.size));
            for e in items.iter().take(200) {
                let p = if sr.total_size > 0 { (e.size as f64 / sr.total_size as f64) * 100.0 } else { 0.0 };
                h.push_str(&format!("<tr><td>{} {}</td><td class='s'>{}</td><td><div class='b' style='width:{:.0}%'></div>{:.1}%</td><td class='e'>{}</td></tr>",
                    if e.is_dir { "\u{1F4C1}" } else { "\u{1F4C4}" }, e.name, format_size(e.size), p, p, if e.extension.is_empty() { "-" } else { &e.extension }));
            }
            h.push_str("</table></body></html>");
            if let Some(p) = rfd::FileDialog::new().set_title("Save Report").add_filter("HTML", &["html"]).save_file() {
                std::fs::write(&p, &h).ok();
            }
        }
    }
}

fn brighten(c: Color32, a: u8) -> Color32 {
    Color32::from_rgb((c.r() as u16 + a as u16).min(255) as u8, (c.g() as u16 + a as u16).min(255) as u8, (c.b() as u16 + a as u16).min(255) as u8)
}

fn paint_gradient(p: &egui::Painter, rect: egui::Rect, cr: egui::CornerRadius, top: Color32, bot: Color32) {
    let n = 8;
    let h = rect.height() / n as f32;
    for i in 0..n {
        let t = i as f32 / (n - 1) as f32;
        let r = (top.r() as f32 * (1.0 - t) + bot.r() as f32 * t) as u8;
        let g = (top.g() as f32 * (1.0 - t) + bot.g() as f32 * t) as u8;
        let b = (top.b() as f32 * (1.0 - t) + bot.b() as f32 * t) as u8;
        let strip = egui::Rect::from_min_size(egui::pos2(rect.min.x, rect.min.y + i as f32 * h), Vec2::new(rect.width(), h + 0.5));
        let c = if i == 0 || i == n - 1 { cr } else { egui::CornerRadius::ZERO };
        p.rect_filled(strip, c, Color32::from_rgb(r, g, b));
    }
}
