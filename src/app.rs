use std::{
    path::PathBuf,
    sync::{mpsc, Arc, Mutex},
    thread,
};

use crate::{
    color,
    keyboard::{self, KeyboardType},
    listen,
    press_time_map::PressTimesMap,
};
use chrono::prelude::DateTime;
use eframe::{App, CreationContext};
use egui::{Color32, Event, Margin, Theme, UserData, ViewportCommand};

pub struct State {
    keyboard_type: KeyboardType,
    hue: f32,
    start_time: DateTime<chrono::Local>,
}

struct KeyboardHeatmap {
    state: Arc<Mutex<State>>,
    press_map: Arc<Mutex<PressTimesMap>>,
    pending_screenshot_path: Option<PathBuf>,
}

impl eframe::App for KeyboardHeatmap {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        let Self {
            state,
            press_map: _,
            pending_screenshot_path: _,
        } = self;
        let mut state = state.lock().unwrap();

        let frame = egui::Frame::new()
            .inner_margin(Margin::same(30))
            .fill(Color32::WHITE);

        egui::CentralPanel::default().frame(frame).show(ctx, |ui| {
            let press_map = &mut self.press_map.lock().unwrap();
            let mut keyboard = keyboard::Keyboard::new(state.keyboard_type, state.hue);
            keyboard.draw(press_map, ui);

            ui.add_space(30.);
            ui.separator();

            ui.horizontal(|ui| {
                ui.label(format!(
                    "Recording since {}",
                    state.start_time.format("%y-%m-%d %H:%M:%S")
                ));

                if ui.button("Clear").clicked() {
                    state.start_time = chrono::Local::now();
                    press_map.map.clear();
                }

                ui.separator();

                ui.label("Keyboard: ");
                egui::ComboBox::from_label("")
                    .selected_text(state.keyboard_type.description())
                    .show_ui(ui, |ui| {
                        ui.selectable_value(
                            &mut state.keyboard_type,
                            KeyboardType::QwertyMac,
                            KeyboardType::QwertyMac.description(),
                        );
                        ui.selectable_value(
                            &mut state.keyboard_type,
                            KeyboardType::Qwerty87,
                            KeyboardType::Qwerty87.description(),
                        );
                    });

                ui.separator();

                ui.label("Theme Palette: ");
                color::color_slider_1d(ui, &mut state.hue, |h| {
                    egui::ecolor::HsvaGamma {
                        h,
                        s: 1.0,
                        v: 1.0,
                        a: 1.0,
                    }
                    .into()
                });

                ui.separator();
                if ui.button("Save as PNG").clicked() {
                    let path = native_dialog::DialogBuilder::file()
                        .set_filename("keyboard-heatmap.png")
                        .add_filter("PNG Image", ["png"])
                        .save_single_file()
                        .show()
                        .unwrap();

                    if let Some(path) = path {
                        self.pending_screenshot_path = Some(path);
                        ctx.send_viewport_cmd(ViewportCommand::Screenshot(UserData::default()));
                    }
                }
            });
        });
    }

    fn raw_input_hook(&mut self, _ctx: &egui::Context, raw_input: &mut egui::RawInput) {
        for event in &raw_input.events {
            if let Event::Screenshot { image, .. } = event {
                if let Some(path) = self.pending_screenshot_path.take() {
                    if let Err(err) = image::save_buffer(
                        path,
                        image.as_raw(),
                        image.width() as u32,
                        image.height() as u32,
                        image::ColorType::Rgba8,
                    ) {
                        eprintln!("failed to save screenshot: {err:?}");
                    }
                }
            }
        }
    }
}

impl KeyboardHeatmap {
    fn new(state: Arc<Mutex<State>>, press_map: Arc<Mutex<PressTimesMap>>) -> Self {
        Self {
            state,
            press_map,
            pending_screenshot_path: None,
        }
    }
}

pub fn setup_ui(
    cc: &CreationContext<'_>,
) -> Result<Box<dyn App>, Box<dyn std::error::Error + Send + Sync>> {
    cc.egui_ctx.set_theme(Theme::Light);

    let (sender, receiver) = mpsc::sync_channel(1);

    let state = Arc::new(Mutex::new(State {
        keyboard_type: KeyboardType::Qwerty87,
        hue: 220. / 360.,
        start_time: chrono::Local::now(),
    }));
    let press_map = Arc::new(Mutex::new(PressTimesMap::new()));

    {
        thread::spawn(move || {
            listen::listen_keyboard(sender.clone());
        });
    }

    {
        let press_map = press_map.clone();
        let egui_ctx = cc.egui_ctx.clone();
        thread::spawn(move || loop {
            if let Ok(event_type) = receiver.recv() {
                let mut press_map = press_map.lock().unwrap();
                if let rdev::EventType::KeyPress(key) = event_type {
                    press_map.key_press(key);
                    egui_ctx.request_repaint();
                }
            }
        });
    }

    Ok(Box::new(KeyboardHeatmap::new(state, press_map)))
}
