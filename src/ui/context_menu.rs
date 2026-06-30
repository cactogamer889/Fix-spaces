use egui::{self};
use crate::scanner::ScanEntry;

pub struct ContextMenu;

impl ContextMenu {
    pub fn show(
        &self,
        ui: &mut egui::Ui,
        entries: &[ScanEntry],
        selected_idx: Option<usize>,
    ) -> ContextMenuAction {
        let mut action = ContextMenuAction::None;

        if let Some(idx) = selected_idx {
            let entry = &entries[idx];

            ui.heading(&entry.name);
            ui.separator();

            if ui.button("Open in Explorer").clicked() {
                let path = entry.path.to_string_lossy().to_string();
                let _ = open::that(&path);
                action = ContextMenuAction::Close;
            }

            if ui.button("Copy Path").clicked() {
                ui.output_mut(|o| o.copied_text = entry.path.to_string_lossy().to_string());
                action = ContextMenuAction::Close;
            }

            ui.separator();

            if ui.button("Delete").clicked() {
                action = ContextMenuAction::Delete(idx);
            }
        }

        action
    }
}

#[derive(Debug)]
pub enum ContextMenuAction {
    None,
    Close,
    Delete(usize),
}
