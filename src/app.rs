use std::{
    collections::HashMap,
    sync::{mpsc, Arc, Mutex},
    thread,
};

use crate::{
    color,
    keyboard::{self, KeyboardType},
    listen,
};
use chrono::prelude::DateTime;
use eframe::{
    epaint::HsvaGamma,
    glow::{self, HasContext},
    App, CreationContext,
};
use egui::{Color32, Layout, Sense, Vec2};

pub struct State {
    keyboard_type: KeyboardType,
    hue: f32,
    start_time: DateTime<chrono::Local>,
}

pub struct PressTimesMap {
    map: HashMap<rdev::Key, u32>,
}

impl PressTimesMap {
    pub fn new() -> Self {
        Self {
            map: HashMap::new(),
        }
    }
    pub fn key_press(&mut self, key: rdev::Key) {
        // incompatitive cases
        let key = match key {
            rdev::Key::Unknown(160) => rdev::Key::F3,
            rdev::Key::Unknown(177) => rdev::Key::F4,
            rdev::Key::Unknown(176) => rdev::Key::F5,
            rdev::Key::Unknown(178) => rdev::Key::F6,
            rdev::Key::Unknown(62) => rdev::Key::ControlRight,
            rdev::Key::Unknown(114) => rdev::Key::Insert,
            rdev::Key::Unknown(115) => rdev::Key::Home,
            rdev::Key::Unknown(116) => rdev::Key::PageUp,
            rdev::Key::Unknown(117) => rdev::Key::Delete,
            rdev::Key::Unknown(119) => rdev::Key::End,
            rdev::Key::Unknown(121) => rdev::Key::PageDown,
            rdev::Key::Unknown(105) => rdev::Key::PrintScreen,
            rdev::Key::Unknown(107) => rdev::Key::ScrollLock,
            rdev::Key::Unknown(113) => rdev::Key::Pause,
            _ => key,
        };
        match self.map.get(&key) {
            Some(v) => self.map.insert(key, v + 1),
            None => self.map.insert(key, 1),
        };
    }
    pub fn get_key_times(&self, key: rdev::Key) -> u32 {
        match self.map.get(&key) {
            Some(&v) => v,
            None => 0,
        }
    }
}
struct KeyboardHeatmap {
    state: Arc<Mutex<State>>,
    press_map: Arc<Mutex<PressTimesMap>>,
    take_screenshot: bool,
}

impl eframe::App for KeyboardHeatmap {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        let Self {
            state,
            press_map: _,
            take_screenshot: _,
        } = self;
        let mut state = state.lock().unwrap();

        let frame = egui::Frame::none()
            .inner_margin(egui::style::Margin {
                left: 30.,
                right: 30.,
                top: 30.,
                bottom: 30.,
            })
            .fill(Color32::WHITE);

        egui::CentralPanel::default().frame(frame).show(ctx, |ui| {
            let press_map = &mut self.press_map.lock().unwrap();
            let mut keyboard = keyboard::Keyboard::new(state.keyboard_type, state.hue);
            keyboard.draw(press_map, ui);

            ui.add_space(30.);
            ui.separator();

            // toolbar
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
                    HsvaGamma {
                        h,
                        s: 1.0,
                        v: 1.0,
                        a: 1.0,
                    }
                    .into()
                });

                ui.separator();
                if ui.button("Save as PNG").clicked() {
                    self.take_screenshot = true;
                }
            });
        });
    }

    /// export the screenshot into a png file.
    fn post_rendering(&mut self, screen_size_px: [u32; 2], frame: &eframe::Frame) {
        if !self.take_screenshot {
            return;
        }
        self.take_screenshot = false;

        // (0, 0) is at the left bottom
        let toolbar_height = 120;
        let state = self.state.lock().unwrap();
        let Some(gl) = frame.gl() else { return };
        let [w, h] = screen_size_px;
        let w = match state.keyboard_type {
            KeyboardType::QwertyMac => w - 20 - 420,
            KeyboardType::Qwerty87 => w - 20,
        };

        let h = h - toolbar_height;
        let mut buf = vec![0u8; w as usize * h as usize * 4];
        let pixels = glow::PixelPackData::Slice(&mut buf[..]);
        unsafe {
            gl.read_pixels(
                0,
                toolbar_height as i32,
                w as i32,
                h as i32,
                glow::RGBA,
                glow::UNSIGNED_BYTE,
                pixels,
            );
        }

        // Flip vertically:
        let mut rows: Vec<Vec<u8>> = buf
            .chunks(w as usize * 4)
            .into_iter()
            .map(|chunk| chunk.to_vec())
            .collect();
        rows.reverse();
        let buf: Vec<u8> = rows.into_iter().flatten().collect();

        // save as image file
        let path = native_dialog::FileDialog::new()
            .set_location("~/Desktop")
            .add_filter("PNG Image", &["png"])
            .show_save_single_file()
            .unwrap();
        if let Some(path) = path {
            if let Err(err) =
                image::save_buffer(path, &buf[..], w as u32, h as u32, image::ColorType::Rgba8)
            {
                println!("err {:?}", err);
            };
        }
    }
}

impl KeyboardHeatmap {
    fn new(state: Arc<Mutex<State>>, press_map: Arc<Mutex<PressTimesMap>>) -> Self {
        Self {
            state,
            press_map,
            take_screenshot: false,
        }
    }
}

pub fn setup_ui(_cc: &CreationContext) -> Box<dyn App> {
    let (sender, receiver) = mpsc::sync_channel(1);

    let state = Arc::new(Mutex::new(State {
        keyboard_type: KeyboardType::Qwerty87,
        hue: 220. / 360.,
        start_time: chrono::Local::now(),
    }));
    let press_map = Arc::new(Mutex::new(PressTimesMap::new()));

    // listen to keyboard press events
    {
        thread::spawn(move || {
            listen::listen_keyboard(sender.clone());
        });
    }
    // handle keyboard press events
    {
        let press_map = press_map.clone();
        thread::spawn(move || loop {
            if let Some(event_type) = receiver.recv().ok() {
                let mut press_map = press_map.lock().unwrap();
                if let rdev::EventType::KeyPress(key) = event_type {
                    press_map.key_press(key);
                }
            }
        });
    }

    Box::new(KeyboardHeatmap::new(state, press_map))
}
