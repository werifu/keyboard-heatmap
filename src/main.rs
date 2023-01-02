#![windows_subsystem = "windows"]
use eframe::{egui, Theme};
use egui::Vec2;
mod app;
mod color;
mod key_box;
mod keyboard;
mod listen;

fn main() {
    let native_options = eframe::NativeOptions {
        min_window_size: Some(Vec2 { x: 900., y: 450. }),
        initial_window_size: Some(Vec2 { x: 1120., y: 450. }),
        default_theme: Theme::Light,
        resizable: false,
        ..Default::default()
    };
    eframe::run_native("Keyboard Heatmap", native_options, Box::new(app::setup_ui))
}
