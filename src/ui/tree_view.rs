use egui::{self, Color32, Rect, Pos2, Vec2};
use crate::scanner::ScanEntry;
use crate::treemap::renderer::{format_size, format_percent, size_color};
use crate::ui::themes::Theme;

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum SortMode {
    SizeDesc,
    SizeAsc,
    NameAz,
    NameZa,
}

pub struct TreeView {
    pub sort_mode: SortMode,
    pub filter_min_bytes: u64,
}

impl Default for TreeView {
    fn default() -> Self {
        Self { sort_mode: SortMode::SizeDesc, filter_min_bytes: 0 }
    }
}

impl TreeView {
    pub fn show(
        &mut self,
        ui: &mut egui::Ui,
        entries: &[ScanEntry],
        root_idx: usize,
        total_size: u64,
        selected_idx: Option<usize>,
        theme: &Theme,
    ) -> Option<usize> {
        let mut clicked = None;
        let w = ui.available_width();

        // Sort controls
        ui.horizontal(|ui| {
            ui.label(egui::RichText::new("Sort:").color(theme.text_dim).size(10.0));
            for (mode, label) in [
                (SortMode::SizeDesc, "Largest"),
                (SortMode::SizeAsc, "Smallest"),
                (SortMode::NameAz, "A-Z"),
                (SortMode::NameZa, "Z-A"),
            ] {
                let active = self.sort_mode == mode;
                let c = if active { theme.accent } else { theme.text_dim };
                if ui.selectable_label(active, egui::RichText::new(label).color(c).size(10.0)).clicked() {
                    self.sort_mode = mode;
                }
            }
        });

        // Filter
        ui.horizontal(|ui| {
            ui.label(egui::RichText::new("Min:").color(theme.text_dim).size(10.0));
            for (bytes, label) in [(0u64, "All"), (1024, "1KB"), (1024*1024, "1MB"), (100*1024*1024, "100MB"), (1024*1024*1024, "1GB")] {
                let active = self.filter_min_bytes == bytes;
                let c = if active { theme.accent } else { theme.text_dim };
                if ui.selectable_label(active, egui::RichText::new(label).color(c).size(10.0)).clicked() {
                    self.filter_min_bytes = bytes;
                }
            }
        });

        ui.add_space(4.0);

        // Header
        let hdr = ui.allocate_response(Vec2::new(w, 18.0), egui::Sense::hover());
        ui.painter().rect_filled(hdr.rect, 0.0, theme.header_bg);
        let hx = hdr.rect.center().y;

        ui.painter().text(Pos2::new(hdr.rect.min.x + 8.0, hx), egui::Align2::LEFT_CENTER, "Name", egui::FontId::proportional(10.0), theme.text_dim);
        ui.painter().text(Pos2::new(hdr.rect.max.x - 160.0, hx), egui::Align2::LEFT_CENTER, "Size", egui::FontId::proportional(10.0), theme.text_dim);
        ui.painter().text(Pos2::new(hdr.rect.max.x - 70.0, hx), egui::Align2::LEFT_CENTER, "% Disk", egui::FontId::proportional(10.0), theme.text_dim);

        ui.add_space(1.0);

        // Collect and sort
        let mut indices: Vec<usize> = entries[root_idx].children.iter().copied()
            .filter(|&i| entries[i].size >= self.filter_min_bytes)
            .collect();

        let max_child_size = indices.iter().map(|&i| entries[i].size).max().unwrap_or(1);

        match self.sort_mode {
            SortMode::SizeDesc => indices.sort_by(|a, b| entries[*b].size.cmp(&entries[*a].size)),
            SortMode::SizeAsc => indices.sort_by(|a, b| entries[*a].size.cmp(&entries[*b].size)),
            SortMode::NameAz => indices.sort_by(|a, b| entries[*a].name.to_lowercase().cmp(&entries[*b].name.to_lowercase())),
            SortMode::NameZa => indices.sort_by(|a, b| entries[*b].name.to_lowercase().cmp(&entries[*a].name.to_lowercase())),
        }

        // Rows
        egui::ScrollArea::vertical().max_height(f32::INFINITY).show(ui, |ui| {
            for &idx in &indices {
                let is_sel = Some(idx) == selected_idx;
                let e = &entries[idx];

                let row = ui.allocate_response(Vec2::new(w, 18.0), egui::Sense::click());
                let rr = row.rect;

                // bg
                if is_sel {
                    ui.painter().rect_filled(rr, 0.0, theme.selected_bg);
                    ui.painter().rect_filled(
                        Rect::from_min_size(rr.min, Vec2::new(3.0, rr.height())),
                        0.0, theme.accent,
                    );
                }

                let cy = rr.center().y;
                let color = size_color(e.size, max_child_size);

                // Color swatch
                let swatch = Rect::from_min_size(Pos2::new(rr.min.x + 4.0, cy - 4.0), Vec2::new(8.0, 8.0));
                ui.painter().rect_filled(swatch, 1.0, color);

                // Name
                let icon = if e.is_dir { " " } else { "" };
                let nc = if is_sel { Color32::WHITE } else { theme.text };
                let name_str = format!("{}{}", icon, e.name);
                ui.painter().text(Pos2::new(rr.min.x + 16.0, cy), egui::Align2::LEFT_CENTER, &name_str, egui::FontId::proportional(11.0), nc);

                // Size
                ui.painter().text(Pos2::new(rr.max.x - 160.0, cy), egui::Align2::LEFT_CENTER, format_size(e.size), egui::FontId::proportional(10.0), theme.text_dim);

                // Percent
                ui.painter().text(Pos2::new(rr.max.x - 70.0, cy), egui::Align2::LEFT_CENTER, format_percent(e.size, total_size), egui::FontId::proportional(10.0), theme.text_dim);

                if row.clicked() { clicked = Some(idx); }
                ui.add_space(1.0);
            }
        });

        clicked
    }
}
