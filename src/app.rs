use std::{
    fs,
    path::PathBuf,
    sync::{mpsc, Arc, Mutex},
    thread,
};

use crate::{
    color,
    key_box::KeyTextsLayout,
    keyboard::{self, KeyboardType},
    listen,
    press_time_map::PressTimesMap,
    tray::{TrayCommand, TrayController},
    typing_log::TypingLog,
    window_visibility,
};
use chrono::prelude::DateTime;
use eframe::{App, CreationContext, Frame};
use egui::{
    pos2, vec2, Align, Align2, Color32, Event, FontId, Layout, Margin, Rect, ScrollArea, Stroke,
    Theme, UserData, Vec2, ViewportCommand, Window,
};
use serde::{Deserialize, Serialize};

const APP_ID: &str = "keyboard-heatmap";
const STATE_FILE: &str = "heatmap-state.json";

pub struct State {
    keyboard_type: KeyboardType,
    hue: f32,
    start_time: DateTime<chrono::Local>,
    show_log_window: bool,
    recording_enabled: bool,
}

#[derive(Serialize, Deserialize)]
struct PersistedState {
    keyboard_type: KeyboardType,
    hue: f32,
    start_time: DateTime<chrono::Local>,
    #[serde(default)]
    show_log_window: bool,
    press_entries: Vec<(String, u32)>,
}

struct KeyboardHeatmap {
    state: Arc<Mutex<State>>,
    press_map: Arc<Mutex<PressTimesMap>>,
    typing_log: Arc<Mutex<TypingLog>>,
    tray_controller: Option<TrayController>,
    pending_screenshot_path: Option<PathBuf>,
    viewport_keyboard_type: KeyboardType,
    window_visible: bool,
    allow_root_close: bool,
}

impl eframe::App for KeyboardHeatmap {
    fn update(&mut self, ctx: &egui::Context, frame: &mut eframe::Frame) {
        if ctx.input(|input| input.viewport().close_requested()) && !self.allow_root_close {
            ctx.send_viewport_cmd(ViewportCommand::CancelClose);
            self.set_window_visibility(ctx, frame, false);
        }

        for command in self.poll_tray_commands() {
            match command {
                TrayCommand::ToggleWindow => {
                    self.set_window_visibility(ctx, frame, !self.window_visible);
                }
                TrayCommand::ToggleRecording => {
                    let mut state = self.state.lock().unwrap();
                    state.recording_enabled = !state.recording_enabled;
                }
                TrayCommand::ClearData => {
                    let mut state = self.state.lock().unwrap();
                    state.start_time = chrono::Local::now();
                    self.press_map.lock().unwrap().map.clear();
                    self.typing_log.lock().unwrap().clear();
                }
                TrayCommand::Quit => {
                    self.allow_root_close = true;
                    ctx.send_viewport_cmd(ViewportCommand::Close);
                }
            }
        }

        let mut state = self.state.lock().unwrap();
        let mut resize_viewport = false;

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

            ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                ui.menu_button("☰", |ui| {
                    ui.set_min_width(220.0);

                    ui.label(format!(
                        "Recording since {}",
                        state.start_time.format("%y-%m-%d %H:%M:%S")
                    ));
                    ui.label(format!("Total presses: {}", press_map.total_presses()));
                    ui.label(format!(
                        "Log buffer: {}/{}",
                        typing_log.len(),
                        typing_log.capacity()
                    ));
                    ui.checkbox(&mut state.show_log_window, "Show log buffer window");
                    ui.separator();

                    ui.label("Keyboard");
                    resize_viewport |= ui
                        .radio_value(
                            &mut state.keyboard_type,
                            KeyboardType::QwertyMac,
                            KeyboardType::QwertyMac.description(),
                        )
                        .changed();
                    resize_viewport |= ui
                        .radio_value(
                            &mut state.keyboard_type,
                            KeyboardType::Qwerty87,
                            KeyboardType::Qwerty87.description(),
                        )
                        .changed();

                    ui.separator();
                    ui.label("Theme Palette");
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

                        ui.close();
                    }

                    if ui.button("Clear data").clicked() {
                        state.start_time = chrono::Local::now();
                        press_map.map.clear();
                        typing_log.clear();
                        ui.close();
                    }
                });

                ui.label(format!("{} presses", press_map.total_presses()));
                if !state.recording_enabled {
                    ui.separator();
                    ui.label("Paused");
                }
            });
        });

        if resize_viewport && self.viewport_keyboard_type != state.keyboard_type {
            let window_size = state.keyboard_type.window_size();
            ctx.send_viewport_cmd(ViewportCommand::MinInnerSize(window_size));
            ctx.send_viewport_cmd(ViewportCommand::InnerSize(window_size));
            self.viewport_keyboard_type = state.keyboard_type;
        }

        if state.show_log_window {
            let preview_keycaps = typing_log_preview(&self.typing_log, state.keyboard_type);
            Window::new("Log Buffer")
                .default_width(PREVIEW_VIEWPORT_WIDTH + 24.0)
                .resizable(true)
                .open(&mut state.show_log_window)
                .show(ctx, |ui| {
                    ui.label(format!("Buffer: {}", preview_keycaps.len()));
                    ui.add_space(6.0);
                    let rows = preview_keycap_rows(&preview_keycaps, PREVIEW_VIEWPORT_WIDTH);
                    ScrollArea::vertical()
                        .max_height(preview_window_height())
                        .stick_to_bottom(true)
                        .show(ui, |ui| {
                            draw_preview_keycaps_rows(
                                ui,
                                &rows,
                                rows.len().max(PREVIEW_VISIBLE_ROWS),
                            );
                        });
                });
        }
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
        tray_controller: Option<TrayController>,
    ) -> Self {
        let viewport_keyboard_type = state.lock().unwrap().keyboard_type;
        Self {
            viewport_keyboard_type,
            state,
            press_map,
            typing_log,
            tray_controller,
            pending_screenshot_path: None,
            window_visible: true,
            allow_root_close: false,
        }
    }

    fn poll_tray_commands(&self) -> Vec<TrayCommand> {
        self.tray_controller
            .as_ref()
            .map(TrayController::poll)
            .unwrap_or_default()
    }

    fn set_window_visibility(&mut self, ctx: &egui::Context, _frame: &Frame, visible: bool) {
        window_visibility::set_window_visibility(_frame, ctx, visible);
        self.window_visible = visible;
    }

    fn save_to_disk(&self) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let state = self.state.lock().unwrap();
        let press_map = self.press_map.lock().unwrap();
        let persisted = PersistedState {
            keyboard_type: state.keyboard_type,
            hue: state.hue,
            start_time: state.start_time,
            show_log_window: state.show_log_window,
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

pub fn initial_keyboard_type() -> KeyboardType {
    load_persisted_state()
        .map(|saved| saved.keyboard_type)
        .unwrap_or_else(default_keyboard_type_for_current_os)
}

fn default_state() -> State {
    State {
        keyboard_type: default_keyboard_type_for_current_os(),
        hue: 220. / 360.,
        start_time: chrono::Local::now(),
        show_log_window: false,
        recording_enabled: true,
    }
}

fn load_state() -> (State, PressTimesMap) {
    let Some(saved) = load_persisted_state() else {
        return (default_state(), PressTimesMap::new());
    };

    (
        State {
            keyboard_type: saved.keyboard_type,
            hue: saved.hue,
            start_time: saved.start_time,
            show_log_window: saved.show_log_window,
            recording_enabled: true,
        },
        PressTimesMap::from_persisted_entries(saved.press_entries),
    )
}

fn load_persisted_state() -> Option<PersistedState> {
    let path = state_file_path();
    let bytes = fs::read(path).ok()?;
    serde_json::from_slice::<PersistedState>(&bytes).ok()
}

fn default_keyboard_type_for_current_os() -> KeyboardType {
    match std::env::consts::OS {
        "macos" => KeyboardType::QwertyMac,
        _ => KeyboardType::Qwerty87,
    }
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

const PREVIEW_KEYCAP_HEIGHT: f32 = 34.0;
const PREVIEW_KEYCAP_BASE_WIDTH: f32 = 34.0;
const PREVIEW_KEYCAP_GAP: f32 = 6.0;
const PREVIEW_VIEWPORT_WIDTH: f32 = 260.0;
const PREVIEW_VISIBLE_ROWS: usize = 3;

fn typing_log_preview(
    typing_log: &Arc<Mutex<TypingLog>>,
    keyboard_type: KeyboardType,
) -> Vec<keyboard::KeyPreviewSpec> {
    typing_log.lock().unwrap().preview_keycaps(keyboard_type)
}

fn preview_window_height() -> f32 {
    PREVIEW_VISIBLE_ROWS as f32 * PREVIEW_KEYCAP_HEIGHT
        + PREVIEW_VISIBLE_ROWS.saturating_sub(1) as f32 * PREVIEW_KEYCAP_GAP
}

fn draw_preview_keycaps_rows(
    ui: &mut egui::Ui,
    rows: &[Vec<keyboard::KeyPreviewSpec>],
    row_count: usize,
) {
    let row_count = row_count.max(1) as f32;
    let height = row_count * PREVIEW_KEYCAP_HEIGHT + (row_count - 1.0) * PREVIEW_KEYCAP_GAP;
    let (rect, _) = ui.allocate_exact_size(
        Vec2::new(PREVIEW_VIEWPORT_WIDTH, height),
        egui::Sense::hover(),
    );
    let painter = ui.painter().with_clip_rect(rect);

    let mut y = rect.top();
    for row in rows {
        let mut x = rect.left();

        for keycap in row {
            let size = preview_keycap_size(keycap.width_units);
            let key_rect = Rect::from_min_size(pos2(x, y), size);
            paint_preview_keycap(&painter, key_rect, &keycap.layout);
            x += size.x + PREVIEW_KEYCAP_GAP;
        }

        y += PREVIEW_KEYCAP_HEIGHT + PREVIEW_KEYCAP_GAP;
    }
}

fn preview_keycap_rows(
    preview_keycaps: &[keyboard::KeyPreviewSpec],
    viewport_width: f32,
) -> Vec<Vec<keyboard::KeyPreviewSpec>> {
    let mut current_row: Vec<keyboard::KeyPreviewSpec> = Vec::new();
    let mut rows: Vec<Vec<keyboard::KeyPreviewSpec>> = Vec::new();
    let mut current_width = 0.0;

    for keycap in preview_keycaps {
        let key_width = preview_keycap_size(keycap.width_units).x;
        let next_width = if current_row.is_empty() {
            key_width
        } else {
            current_width + PREVIEW_KEYCAP_GAP + key_width
        };

        if !current_row.is_empty() && next_width > viewport_width {
            rows.push(std::mem::take(&mut current_row));
            current_row.push(keycap.clone());
            current_width = key_width;
        } else {
            current_row.push(keycap.clone());
            current_width = next_width;
        }
    }

    if !current_row.is_empty() {
        rows.push(current_row);
    }

    rows
}

fn preview_keycap_size(width_units: f32) -> Vec2 {
    Vec2::new(
        PREVIEW_KEYCAP_BASE_WIDTH * width_units,
        PREVIEW_KEYCAP_HEIGHT,
    )
}

fn paint_preview_keycap(painter: &egui::Painter, rect: Rect, layout: &KeyTextsLayout) {
    let fill = Color32::from_rgb(247, 247, 247);
    let stroke = Stroke::new(1.0, Color32::from_rgb(170, 170, 170));
    let text_color = Color32::from_rgb(52, 52, 52);

    painter.rect_filled(rect, 5.0, fill);
    painter.rect_stroke(rect, 5.0, stroke, egui::StrokeKind::Inside);

    match layout {
        KeyTextsLayout::Center1(text) => {
            painter.text(
                rect.center(),
                Align2::CENTER_CENTER,
                text,
                FontId::monospace(12.0),
                text_color,
            );
        }
        KeyTextsLayout::TopBottom((top, bottom)) => {
            painter.text(
                rect.center_top() + vec2(0.0, 10.0),
                Align2::CENTER_CENTER,
                top,
                FontId::monospace(11.0),
                text_color,
            );
            painter.text(
                rect.center_bottom() - vec2(0.0, 10.0),
                Align2::CENTER_CENTER,
                bottom,
                FontId::monospace(11.0),
                text_color,
            );
        }
    }
}

pub fn setup_ui(
    cc: &CreationContext<'_>,
) -> Result<Box<dyn App>, Box<dyn std::error::Error + Send + Sync>> {
    cc.egui_ctx.set_theme(Theme::Light);
    TrayController::install_repaint_forwarder(&cc.egui_ctx);

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
        let state = state.clone();
        let egui_ctx = cc.egui_ctx.clone();
        thread::spawn(move || loop {
            if let Ok(event) = receiver.recv() {
                if !state.lock().unwrap().recording_enabled {
                    continue;
                }
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

    let tray_controller = TrayController::new().ok();
    Ok(Box::new(KeyboardHeatmap::new(
        state,
        press_map,
        typing_log,
        tray_controller,
    )))
}
