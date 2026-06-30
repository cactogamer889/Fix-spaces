use egui::{self, RichText, Color32};
use crate::scanner::ScanEntry;
use crate::treemap::renderer::{format_size, format_percent};

pub struct DetailsPanel;

impl DetailsPanel {
    pub fn show(
        &self,
        ui: &mut egui::Ui,
        entries: &[ScanEntry],
        selected_idx: Option<usize>,
        total_size: u64,
    ) {
        if let Some(idx) = selected_idx {
            let entry = &entries[idx];

            ui.horizontal(|ui| {
                ui.heading(RichText::new("Details").size(14.0).strong());
            });
            ui.add_space(4.0);

            let parent_size = entry.parent.map(|p| entries[p].size).unwrap_or(total_size);

            ui.label(RichText::new("Path").strong().size(11.0));
            ui.label(
                RichText::new(entry.path.to_string_lossy().to_string())
                    .size(11.0)
                    .color(Color32::from_rgb(150, 200, 255)),
            );
            ui.add_space(4.0);

            ui.horizontal(|ui| {
                ui.label(RichText::new("Size:").strong().size(11.0));
                ui.label(RichText::new(format_size(entry.size)).size(11.0));
            });

            ui.horizontal(|ui| {
                ui.label(RichText::new("% of Disk:").strong().size(11.0));
                ui.label(RichText::new(format_percent(entry.size, total_size)).size(11.0));
            });

            ui.horizontal(|ui| {
                ui.label(RichText::new("% of Parent:").strong().size(11.0));
                ui.label(RichText::new(format_percent(entry.size, parent_size)).size(11.0));
            });

            if entry.is_dir {
                ui.add_space(4.0);
                ui.horizontal(|ui| {
                    ui.label(RichText::new("Items:").strong().size(11.0));
                    ui.label(RichText::new(format!("{}", entry.children.len())).size(11.0));
                });
            } else {
                ui.add_space(4.0);
                ui.horizontal(|ui| {
                    ui.label(RichText::new("Type:").strong().size(11.0));
                    let ext = if entry.extension.is_empty() {
                        "(none)".to_string()
                    } else {
                        format!(".{}", entry.extension)
                    };
                    ui.label(RichText::new(ext).size(11.0));
                });
            }

            ui.add_space(8.0);

            ui.horizontal(|ui| {
                if ui.button("Open in Explorer").clicked() {
                    let path = entry.path.to_string_lossy().to_string();
                    if entry.is_dir {
                        let _ = open::that(&path);
                    } else {
                        let _ = open::that(&path);
                    }
                }
            });
        } else {
            ui.centered_and_justified(|ui| {
                ui.label(RichText::new("Select an item to view details").color(Color32::GRAY));
            });
        }
    }
}
