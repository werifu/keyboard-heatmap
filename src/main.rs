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
    let initial_keyboard_type = app::initial_keyboard_type();
    let initial_window_size = initial_keyboard_type.window_size();
    let native_options = eframe::NativeOptions {
        viewport: ViewportBuilder::default()
            .with_inner_size(initial_window_size)
            .with_min_inner_size(keyboard::KeyboardType::QwertyMac.window_size())
            .with_resizable(false),
        ..Default::default()
    };
    eframe::run_native("Keyboard Heatmap", native_options, Box::new(app::setup_ui))
}
