use egui::{self, RichText, Color32, Vec2};
use crate::scanner::get_drives;
use crate::ui::themes::{Theme, THEMES};

pub struct Toolbar {
    pub selected_drive: String,
    pub search_text: String,
    pub is_scanning: bool,
    pub theme_index: usize,
}

impl Default for Toolbar {
    fn default() -> Self {
        let drives = get_drives();
        let selected_drive = drives.first().cloned().unwrap_or_else(|| "C:\\".to_string());
        Self { selected_drive, search_text: String::new(), is_scanning: false, theme_index: 0 }
    }
}

pub enum ToolbarAction {
    None,
    StartScan,
    StopScan,
    ExportHtml,
    ThemeChanged(usize),
}

impl Toolbar {
    pub fn show(&mut self, ui: &mut egui::Ui, custom_path: &mut String, theme: &Theme) -> ToolbarAction {
        let mut action = ToolbarAction::None;

        ui.horizontal(|ui| {
            ui.label(RichText::new("Fix Space").size(15.0).strong().color(theme.accent));
            ui.separator();

            // Drive
            ui.label(RichText::new("Drive:").color(theme.text_dim).size(11.0));
            let drives = get_drives();
            egui::ComboBox::from_id_salt("drive").selected_text(&self.selected_drive).width(70.0).show_ui(ui, |ui| {
                for d in &drives {
                    if ui.selectable_label(self.selected_drive == *d, d).clicked() {
                        self.selected_drive = d.clone();
                        custom_path.clear();
                    }
                }
            });

            ui.separator();

            // Path
            ui.label(RichText::new("Path:").color(theme.text_dim).size(11.0));
            let r = ui.add(egui::TextEdit::singleline(custom_path).desired_width(180.0).hint_text("C:\\Users"));
            if r.lost_focus() && ui.input(|i| i.key_pressed(egui::Key::Enter)) {
                if !custom_path.is_empty() { self.selected_drive = custom_path.clone(); }
            }

            ui.separator();

            // Scan / Stop
            if self.is_scanning {
                if ui.add(egui::Button::new(RichText::new("Stop").color(Color32::from_rgb(255, 80, 80))).fill(Color32::from_rgb(60, 20, 20))).clicked() {
                    action = ToolbarAction::StopScan;
                }
                ui.spinner();
            } else {
                if ui.add(egui::Button::new(RichText::new("Scan").strong().color(Color32::WHITE)).fill(theme.accent).min_size(Vec2::new(55.0, 22.0))).clicked() {
                    action = ToolbarAction::StartScan;
                }
            }

            ui.separator();

            // Theme
            egui::ComboBox::from_id_salt("theme").selected_text(THEMES[self.theme_index].name).width(75.0).show_ui(ui, |ui| {
                for (i, t) in THEMES.iter().enumerate() {
                    if ui.selectable_label(self.theme_index == i, t.name).clicked() {
                        self.theme_index = i;
                        action = ToolbarAction::ThemeChanged(i);
                    }
                }
            });

            // Right side
            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                if ui.add(egui::Button::new(RichText::new("Export").color(theme.text_dim)).fill(theme.surface)).clicked() {
                    action = ToolbarAction::ExportHtml;
                }
            });
        });

        action
    }
}
