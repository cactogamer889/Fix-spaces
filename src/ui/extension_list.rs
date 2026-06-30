use egui::{self, Color32, Vec2};
use crate::scanner::ScanEntry;
use crate::treemap::renderer::{format_size, format_percent, ext_color};
use crate::ui::themes::Theme;
use std::collections::HashMap;

pub struct ExtensionList;

impl ExtensionList {
    pub fn show(
        &self,
        ui: &mut egui::Ui,
        entries: &[ScanEntry],
        total_size: u64,
        theme: &Theme,
    ) {
        let mut ext_map: HashMap<String, (usize, u64)> = HashMap::new();
        for e in entries {
            if !e.is_dir && !e.extension.is_empty() {
                let entry = ext_map.entry(e.extension.clone()).or_insert((0, 0));
                entry.0 += 1;
                entry.1 += e.size;
            }
        }

        let mut stats: Vec<(String, usize, u64)> = ext_map.into_iter().map(|(ext, (c, s))| (ext, c, s)).collect();
        stats.sort_by(|a, b| b.2.cmp(&a.2));

        let total_ext_size: u64 = stats.iter().map(|(_, _, s)| *s).sum();

        // Show as horizontal chips
        egui::ScrollArea::horizontal().show(ui, |ui| {
            ui.horizontal_wrapped(|ui| {
                ui.spacing_mut().item_spacing = Vec2::new(6.0, 4.0);

                for (ext, count, size) in stats.iter().take(40) {
                    let color = ext_color(ext);
                    let pct = format_percent(*size, total_ext_size);
                    let label = format!(".{} {} ({})", ext, format_size(*size), count);

                    let resp = ui.allocate_response(Vec2::new(0.0, 0.0), egui::Sense::click());
                    let rect = resp.rect.expand(2.0);
                    ui.painter().rect_filled(rect, 4.0, Color32::from_rgba_premultiplied(
                        theme.surface.r(), theme.surface.g(), theme.surface.b(), 180,
                    ));

                    ui.horizontal(|ui| {
                        let (sw, _) = ui.allocate_exact_size(Vec2::new(8.0, 8.0), egui::Sense::hover());
                        ui.painter().rect_filled(sw, 2.0, color);
                        ui.label(egui::RichText::new(&label).color(theme.text).size(10.0));
                    });
                }
            });
        });
    }
}
