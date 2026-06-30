mod app;
mod cache;
mod colors;
mod scanner;
mod treemap;
mod ui;

use app::DiskVizApp;

fn main() -> eframe::Result<()> {
    env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("info")).init();

    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([1280.0, 850.0])
            .with_min_inner_size([900.0, 500.0])
            .with_title("Fix Space - Disk Analyzer"),
        ..Default::default()
    };

    eframe::run_native(
        "Fix Space",
        options,
        Box::new(|cc| Ok(Box::new(DiskVizApp::new(cc)))),
    )
}
