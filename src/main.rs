#![windows_subsystem = "windows"]
use eframe::egui::ViewportBuilder;
mod app;
mod color;
mod key_box;
mod keyboard;
mod listen;
mod press_time_map;
mod typing_log;

fn main() -> eframe::Result {
    let native_options = eframe::NativeOptions {
        viewport: ViewportBuilder::default()
            .with_inner_size([1120.0, 450.0])
            .with_min_inner_size([900.0, 450.0])
            .with_resizable(false),
        ..Default::default()
    };
    eframe::run_native("Keyboard Heatmap", native_options, Box::new(app::setup_ui))
}
