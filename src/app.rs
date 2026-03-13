use std::{
    fs,
    path::PathBuf,
    sync::{mpsc, Arc, Mutex},
    thread,
};

use crate::{
    color,
    keyboard::{self, KeyboardType},
    listen,
    press_time_map::PressTimesMap,
    typing_log::TypingLog,
};
use chrono::prelude::DateTime;
use eframe::{App, CreationContext};
use egui::{Color32, Event, Margin, Theme, UserData, ViewportCommand};
use serde::{Deserialize, Serialize};

const APP_ID: &str = "keyboard-heatmap";
const STATE_FILE: &str = "heatmap-state.json";

pub struct State {
    keyboard_type: KeyboardType,
    hue: f32,
    start_time: DateTime<chrono::Local>,
}

#[derive(Serialize, Deserialize)]
struct PersistedState {
    keyboard_type: KeyboardType,
    hue: f32,
    start_time: DateTime<chrono::Local>,
    press_entries: Vec<(String, u32)>,
}

struct KeyboardHeatmap {
    state: Arc<Mutex<State>>,
    press_map: Arc<Mutex<PressTimesMap>>,
    typing_log: Arc<Mutex<TypingLog>>,
    pending_screenshot_path: Option<PathBuf>,
}

impl eframe::App for KeyboardHeatmap {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        let Self {
            state,
            press_map: _,
            typing_log: _,
            pending_screenshot_path: _,
        } = self;
        let mut state = state.lock().unwrap();

        let frame = egui::Frame::new()
            .inner_margin(Margin::same(30))
            .fill(Color32::WHITE);

        egui::CentralPanel::default().frame(frame).show(ctx, |ui| {
            let press_map = &mut self.press_map.lock().unwrap();
            let mut typing_log = self.typing_log.lock().unwrap();
            let mut keyboard = keyboard::Keyboard::new(state.keyboard_type, state.hue);
            keyboard.draw(press_map, ui);

            ui.add_space(30.);
            ui.separator();

            ui.horizontal(|ui| {
                ui.label(format!(
                    "Recording since {}",
                    state.start_time.format("%y-%m-%d %H:%M:%S")
                ));

                ui.separator();
                ui.label(format!("Total presses: {}", press_map.total_presses()));
                let log_label = ui.label(format!(
                    "Log buffer: {}/{}",
                    typing_log.len(),
                    typing_log.capacity()
                ));
                let preview = typing_log.preview();
                if !preview.is_empty() {
                    log_label.on_hover_text(preview);
                }

                if ui.button("Clear").clicked() {
                    state.start_time = chrono::Local::now();
                    press_map.map.clear();
                    typing_log.clear();
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

    fn on_exit(&mut self, _gl: Option<&eframe::glow::Context>) {
        if let Err(err) = self.save_to_disk() {
            eprintln!("failed to save state: {err}");
        }
    }
}

impl KeyboardHeatmap {
    fn new(
        state: Arc<Mutex<State>>,
        press_map: Arc<Mutex<PressTimesMap>>,
        typing_log: Arc<Mutex<TypingLog>>,
    ) -> Self {
        Self {
            state,
            press_map,
            typing_log,
            pending_screenshot_path: None,
        }
    }

    fn save_to_disk(&self) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let state = self.state.lock().unwrap();
        let press_map = self.press_map.lock().unwrap();
        let persisted = PersistedState {
            keyboard_type: state.keyboard_type,
            hue: state.hue,
            start_time: state.start_time,
            press_entries: press_map.persisted_entries(),
        };

        let path = state_file_path();
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent)?;
        }
        fs::write(path, serde_json::to_vec_pretty(&persisted)?)?;
        Ok(())
    }
}

fn default_state() -> State {
    State {
        keyboard_type: KeyboardType::Qwerty87,
        hue: 220. / 360.,
        start_time: chrono::Local::now(),
    }
}

fn load_state() -> (State, PressTimesMap) {
    let path = state_file_path();
    let Ok(bytes) = fs::read(path) else {
        return (default_state(), PressTimesMap::new());
    };
    let Ok(saved) = serde_json::from_slice::<PersistedState>(&bytes) else {
        return (default_state(), PressTimesMap::new());
    };

    (
        State {
            keyboard_type: saved.keyboard_type,
            hue: saved.hue,
            start_time: saved.start_time,
        },
        PressTimesMap::from_persisted_entries(saved.press_entries),
    )
}

fn state_file_path() -> PathBuf {
    app_data_dir().join(STATE_FILE)
}

fn app_data_dir() -> PathBuf {
    #[cfg(target_os = "windows")]
    {
        std::env::var_os("APPDATA")
            .map(PathBuf::from)
            .unwrap_or_else(|| PathBuf::from("."))
            .join(APP_ID)
    }

    #[cfg(target_os = "macos")]
    {
        std::env::var_os("HOME")
            .map(PathBuf::from)
            .unwrap_or_else(|| PathBuf::from("."))
            .join("Library")
            .join("Application Support")
            .join(APP_ID)
    }

    #[cfg(all(not(target_os = "windows"), not(target_os = "macos")))]
    {
        std::env::var_os("XDG_DATA_HOME")
            .map(PathBuf::from)
            .or_else(|| {
                std::env::var_os("HOME")
                    .map(PathBuf::from)
                    .map(|path| path.join(".local").join("share"))
            })
            .unwrap_or_else(|| PathBuf::from("."))
            .join(APP_ID)
    }
}

pub fn setup_ui(
    cc: &CreationContext<'_>,
) -> Result<Box<dyn App>, Box<dyn std::error::Error + Send + Sync>> {
    cc.egui_ctx.set_theme(Theme::Light);

    let (sender, receiver) = mpsc::sync_channel(1);
    let (saved_state, saved_press_map) = load_state();

    let state = Arc::new(Mutex::new(saved_state));
    let press_map = Arc::new(Mutex::new(saved_press_map));
    let typing_log = Arc::new(Mutex::new(TypingLog::new()));

    {
        thread::spawn(move || {
            listen::listen_keyboard(sender.clone());
        });
    }

    {
        let press_map = press_map.clone();
        let typing_log = typing_log.clone();
        let egui_ctx = cc.egui_ctx.clone();
        thread::spawn(move || loop {
            if let Ok(event) = receiver.recv() {
                let mut press_map = press_map.lock().unwrap();
                let mut typing_log = typing_log.lock().unwrap();
                if let rdev::EventType::KeyPress(key) = event.event_type {
                    press_map.key_press(key);
                    typing_log.push_event(&event);
                    egui_ctx.request_repaint();
                }
            }
        });
    }

    Ok(Box::new(KeyboardHeatmap::new(state, press_map, typing_log)))
}
