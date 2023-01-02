use std::sync::MutexGuard;

use crate::{
    key_box::{KeyBox, KeyTextsLayout},
    press_time_map::PressTimesMap,
};

use egui::{Color32, Sense, Ui, Vec2};

use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, PartialEq, Deserialize, Serialize)]
pub enum KeyboardType {
    QwertyMac,
    Qwerty87,
}

impl KeyboardType {
    pub fn description(&self) -> &'static str {
        match self {
            KeyboardType::QwertyMac => "MacBook",
            KeyboardType::Qwerty87 => "87 Keys",
        }
    }
}

const FN_KEYS_PAIRS: [(&str, rdev::Key); 12] = [
    ("F1", rdev::Key::F1),
    ("F2", rdev::Key::F2),
    ("F3", rdev::Key::F3),
    ("F4", rdev::Key::F4),
    ("F5", rdev::Key::F5),
    ("F6", rdev::Key::F6),
    ("F7", rdev::Key::F7),
    ("F8", rdev::Key::F8),
    ("F9", rdev::Key::F9),
    ("F10", rdev::Key::F10),
    ("F11", rdev::Key::F11),
    ("F12", rdev::Key::F12),
];
const NUM_KEY_LINE_PAIRS: [(&str, &str, rdev::Key); 13] = [
    ("`", "~", rdev::Key::BackQuote),
    ("!", "1", rdev::Key::Num1),
    ("@", "2", rdev::Key::Num2),
    ("#", "3", rdev::Key::Num3),
    ("$", "4", rdev::Key::Num4),
    ("%", "5", rdev::Key::Num5),
    ("^", "6", rdev::Key::Num6),
    ("&", "7", rdev::Key::Num7),
    ("*", "8", rdev::Key::Num8),
    ("(", "9", rdev::Key::Num9),
    (")", "0", rdev::Key::Num0),
    ("-", "_", rdev::Key::Minus),
    ("+", "=", rdev::Key::Equal),
];

const FIRST_ALPHA_LINE_PAIRS: [(&str, rdev::Key); 10] = [
    ("Q", rdev::Key::KeyQ),
    ("W", rdev::Key::KeyW),
    ("E", rdev::Key::KeyE),
    ("R", rdev::Key::KeyR),
    ("T", rdev::Key::KeyT),
    ("Y", rdev::Key::KeyY),
    ("U", rdev::Key::KeyU),
    ("I", rdev::Key::KeyI),
    ("O", rdev::Key::KeyO),
    ("P", rdev::Key::KeyP),
];

const SECOND_ALPHA_LINE_PAIRS: [(&str, rdev::Key); 9] = [
    ("A", rdev::Key::KeyA),
    ("S", rdev::Key::KeyS),
    ("D", rdev::Key::KeyD),
    ("F", rdev::Key::KeyF),
    ("G", rdev::Key::KeyG),
    ("H", rdev::Key::KeyH),
    ("J", rdev::Key::KeyJ),
    ("K", rdev::Key::KeyK),
    ("L", rdev::Key::KeyL),
];

const THIRD_ALPHA_LINE_PAIRS: [(&str, rdev::Key); 7] = [
    ("Z", rdev::Key::KeyZ),
    ("X", rdev::Key::KeyX),
    ("C", rdev::Key::KeyC),
    ("V", rdev::Key::KeyV),
    ("B", rdev::Key::KeyB),
    ("N", rdev::Key::KeyN),
    ("M", rdev::Key::KeyM),
];

const SECTION_SPACE: f32 = 15.;

pub struct Keyboard {
    // different from key numbers (and OSs)
    keyboard_type: KeyboardType,
    // [0, 1], the hue of the color
    hue: f32,
}

impl Keyboard {
    pub fn new(keyboard_type: KeyboardType, hue: f32) -> Self {
        Self { keyboard_type, hue }
    }
}
impl Keyboard {
    pub fn draw(&mut self, map: &MutexGuard<PressTimesMap>, ui: &mut Ui) {
        match self.keyboard_type {
            KeyboardType::QwertyMac => self.draw_mac_keyboard(map, ui),
            KeyboardType::Qwerty87 => self.draw_87_keyboard(map, ui),
        }
    }

    fn draw_mac_keyboard(&mut self, map: &MutexGuard<PressTimesMap>, ui: &mut Ui) {
        let basic_size = Vec2 { x: 50., y: 50. };
        // 1st line
        ui.horizontal(|ui| {
            self.draw_single_label_key(map, Vec2 { x: 70., y: 50. }, rdev::Key::Escape, "Esc", ui);

            for key_pair in FN_KEYS_PAIRS.iter() {
                self.draw_single_label_key(map, basic_size, key_pair.1, key_pair.0, ui)
            }

            self.draw_single_label_key(map, basic_size, rdev::Key::Unknown(0), "Power", ui);
        });
        ui.add_space(3.);

        // 2nd line
        ui.horizontal(|ui| {
            for key_pair in NUM_KEY_LINE_PAIRS.iter() {
                self.draw_double_labels_key(
                    map, basic_size, key_pair.2, key_pair.0, key_pair.1, ui,
                );
            }

            self.draw_single_label_key(
                map,
                Vec2 { x: 70., y: 50. },
                rdev::Key::Backspace,
                "Back",
                ui,
            );
        });
        ui.add_space(3.);

        // tab line
        ui.horizontal(|ui| {
            self.draw_single_label_key(map, Vec2 { x: 70., y: 50. }, rdev::Key::Tab, "Tab", ui);

            for key_pair in FIRST_ALPHA_LINE_PAIRS.iter() {
                self.draw_single_label_key(map, basic_size, key_pair.1, key_pair.0, ui);
            }

            for key_pair in [
                ("[", "{", rdev::Key::LeftBracket),
                ("]", "}", rdev::Key::RightBracket),
                ("\\", "|", rdev::Key::BackSlash),
            ] {
                self.draw_double_labels_key(
                    map, basic_size, key_pair.2, key_pair.0, key_pair.1, ui,
                );
            }
        });
        ui.add_space(3.);

        // caps lock line
        ui.horizontal(|ui| {
            self.draw_single_label_key(
                map,
                Vec2 { x: 85., y: 50. },
                rdev::Key::CapsLock,
                "Caps\nLock",
                ui,
            );

            for key_pair in SECOND_ALPHA_LINE_PAIRS.iter() {
                self.draw_single_label_key(map, basic_size, key_pair.1, key_pair.0, ui);
            }

            for key_pair in [
                (":", ";", rdev::Key::SemiColon),
                ("\"", "'", rdev::Key::Quote),
            ] {
                self.draw_double_labels_key(
                    map, basic_size, key_pair.2, key_pair.0, key_pair.1, ui,
                );
            }

            // enter
            self.draw_single_label_key(
                map,
                Vec2 { x: 93., y: 50. },
                rdev::Key::Return,
                "Enter",
                ui,
            );
        });
        ui.add_space(3.);

        // shift line
        ui.horizontal(|ui| {
            self.draw_single_label_key(
                map,
                Vec2 { x: 118., y: 50. },
                rdev::Key::ShiftLeft,
                "Shift",
                ui,
            );

            for key_pair in THIRD_ALPHA_LINE_PAIRS.iter() {
                self.draw_single_label_key(map, basic_size, key_pair.1, key_pair.0, ui)
            }

            for key_pair in [
                ("<", ",", rdev::Key::Comma),
                (">", ".", rdev::Key::Dot),
                ("?", "/", rdev::Key::Slash),
            ] {
                self.draw_double_labels_key(
                    map, basic_size, key_pair.2, key_pair.0, key_pair.1, ui,
                );
            }

            // right shift
            self.draw_single_label_key(
                map,
                Vec2 { x: 118., y: 50. },
                rdev::Key::ShiftRight,
                "Shift",
                ui,
            );
        });
        ui.add_space(3.);

        // last line
        ui.horizontal(|ui| {
            self.draw_single_label_key(map, basic_size, rdev::Key::Function, "Fn", ui);
            self.draw_single_label_key(map, basic_size, rdev::Key::ControlLeft, "Ctrl", ui);
            self.draw_single_label_key(map, basic_size, rdev::Key::Alt, "Opt", ui);
            self.draw_single_label_key(
                map,
                Vec2 { x: 61., y: 50. },
                rdev::Key::MetaLeft,
                "Cmd",
                ui,
            );
            self.draw_single_label_key(map, Vec2 { x: 280., y: 50. }, rdev::Key::Space, " ", ui);
            self.draw_single_label_key(
                map,
                Vec2 { x: 61., y: 50. },
                rdev::Key::MetaRight,
                "Cmd",
                ui,
            );
            self.draw_single_label_key(map, basic_size, rdev::Key::AltGr, "Opt", ui);

            let left_times = map.get_key_times(rdev::Key::LeftArrow);
            let mut left_key = KeyBox::new(
                Vec2 { x: 50., y: 24. },
                KeyTextsLayout::Center1("←".to_string()),
                rdev::Key::LeftArrow,
                left_times,
                self.hue,
            );
            ui.vertical(|ui| {
                let (rect, _) = ui.allocate_exact_size(Vec2 { x: 50., y: 24. }, Sense::hover());
                ui.painter().rect_filled(rect, 0., Color32::TRANSPARENT);
                left_key.ui(ui);
            });

            let up_times = map.get_key_times(rdev::Key::UpArrow);
            let mut up_key = KeyBox::new(
                Vec2 { x: 50., y: 24. },
                KeyTextsLayout::Center1("↑".to_string()),
                rdev::Key::UpArrow,
                up_times,
                self.hue,
            );
            let down_times = map.get_key_times(rdev::Key::DownArrow);
            let mut down_key = KeyBox::new(
                Vec2 { x: 50., y: 24. },
                KeyTextsLayout::Center1("↓".to_string()),
                rdev::Key::DownArrow,
                down_times,
                self.hue,
            );
            ui.vertical(|ui| {
                up_key.ui(ui);
                down_key.ui(ui);
            });

            // ->
            let right_times = map.get_key_times(rdev::Key::RightArrow);
            let mut right_key = KeyBox::new(
                Vec2 { x: 50., y: 24. },
                KeyTextsLayout::Center1("→".to_string()),
                rdev::Key::RightArrow,
                right_times,
                self.hue,
            );
            ui.vertical(|ui| {
                let (rect, _) = ui.allocate_exact_size(Vec2 { x: 50., y: 25. }, Sense::hover());
                ui.painter().rect_filled(rect, 0., Color32::TRANSPARENT);
                right_key.ui(ui);
            });
        });
    }

    fn draw_87_keyboard(&mut self, map: &MutexGuard<PressTimesMap>, ui: &mut Ui) {
        let basic_size = Vec2 { x: 50., y: 50. };
        // 1st line
        ui.horizontal(|ui| {
            self.draw_single_label_key(map, basic_size, rdev::Key::Escape, "Esc", ui);

            self.draw_empty_key(basic_size, ui);

            for key_pair in FN_KEYS_PAIRS.iter() {
                self.draw_single_label_key(map, basic_size, key_pair.1, key_pair.0, ui);
                if key_pair.1 == rdev::Key::F4 || key_pair.1 == rdev::Key::F8 {
                    ui.add_space(25.);
                }
            }

            ui.add_space(SECTION_SPACE);

            for key_pair in [
                ("PrtSc", rdev::Key::PrintScreen),
                ("ScrLk", rdev::Key::ScrollLock),
                ("Pause", rdev::Key::Pause),
            ] {
                self.draw_single_label_key(map, basic_size, key_pair.1, key_pair.0, ui);
            }
        });
        ui.add_space(3.);

        // 2nd line
        ui.horizontal(|ui| {
            for key_pair in NUM_KEY_LINE_PAIRS.iter() {
                self.draw_double_labels_key(
                    map, basic_size, key_pair.2, key_pair.0, key_pair.1, ui,
                );
            }

            self.draw_single_label_key(
                map,
                Vec2 { x: 100., y: 50. },
                rdev::Key::Backspace,
                "Back",
                ui,
            );

            ui.add_space(SECTION_SPACE);

            for key_pair in [
                ("Ins", rdev::Key::Insert),
                ("Home", rdev::Key::Home),
                ("PgUp", rdev::Key::PageUp),
            ] {
                self.draw_single_label_key(map, basic_size, key_pair.1, key_pair.0, ui);
            }
        });
        ui.add_space(3.);

        // tab line
        ui.horizontal(|ui| {
            self.draw_single_label_key(map, Vec2 { x: 70., y: 50. }, rdev::Key::Tab, "Tab", ui);

            for key_pair in FIRST_ALPHA_LINE_PAIRS.iter() {
                self.draw_single_label_key(map, basic_size, key_pair.1, key_pair.0, ui);
            }

            for key_pair in [
                ("[", "{", rdev::Key::LeftBracket),
                ("]", "}", rdev::Key::RightBracket),
            ] {
                self.draw_double_labels_key(
                    map, basic_size, key_pair.2, key_pair.0, key_pair.1, ui,
                );
            }

            let key_pair = ("\\", "|", rdev::Key::BackSlash);
            self.draw_double_labels_key(
                map,
                Vec2 { x: 80., y: 50. },
                key_pair.2,
                key_pair.0,
                key_pair.1,
                ui,
            );

            ui.add_space(SECTION_SPACE);

            for key_pair in [
                ("Del", rdev::Key::Delete),
                ("End", rdev::Key::End),
                ("PgDn", rdev::Key::PageDown),
            ] {
                self.draw_single_label_key(map, basic_size, key_pair.1, key_pair.0, ui);
            }
        });
        ui.add_space(3.);

        // caps lock line
        ui.horizontal(|ui| {
            self.draw_single_label_key(
                map,
                Vec2 { x: 85., y: 50. },
                rdev::Key::CapsLock,
                "Caps\nLock",
                ui,
            );

            for key_pair in SECOND_ALPHA_LINE_PAIRS.iter() {
                self.draw_single_label_key(map, basic_size, key_pair.1, key_pair.0, ui);
            }

            for key_pair in [
                (":", ";", rdev::Key::SemiColon),
                ("\"", "'", rdev::Key::Quote),
            ] {
                self.draw_double_labels_key(
                    map, basic_size, key_pair.2, key_pair.0, key_pair.1, ui,
                );
            }

            // enter
            self.draw_single_label_key(
                map,
                Vec2 { x: 123., y: 50. },
                rdev::Key::Return,
                "Enter",
                ui,
            );

            ui.add_space(SECTION_SPACE);
            for _ in 0..3 {
                self.draw_empty_key(basic_size, ui);
            }
        });
        ui.add_space(3.);

        // shift line
        ui.horizontal(|ui| {
            self.draw_single_label_key(
                map,
                Vec2 { x: 118., y: 50. },
                rdev::Key::ShiftLeft,
                "Shift",
                ui,
            );

            for key_pair in THIRD_ALPHA_LINE_PAIRS.iter() {
                self.draw_single_label_key(map, basic_size, key_pair.1, key_pair.0, ui)
            }

            for key_pair in [
                ("<", ",", rdev::Key::Comma),
                (">", ".", rdev::Key::Dot),
                ("?", "/", rdev::Key::Slash),
            ] {
                self.draw_double_labels_key(
                    map, basic_size, key_pair.2, key_pair.0, key_pair.1, ui,
                );
            }

            // right shift
            self.draw_single_label_key(
                map,
                Vec2 { x: 148., y: 50. },
                rdev::Key::ShiftRight,
                "Shift",
                ui,
            );

            ui.add_space(SECTION_SPACE);
            self.draw_empty_key(basic_size, ui);
            self.draw_single_label_key(map, basic_size, rdev::Key::UpArrow, "↑", ui);
            self.draw_empty_key(basic_size, ui);
        });
        ui.add_space(3.);

        // last line
        ui.horizontal(|ui| {
            let ctrl_size = Vec2 { x: 60., y: 50. };
            self.draw_single_label_key(map, ctrl_size, rdev::Key::ControlLeft, "Ctrl", ui);

            self.draw_single_label_key(map, ctrl_size, rdev::Key::MetaLeft, "Win", ui);
            self.draw_single_label_key(map, ctrl_size, rdev::Key::Alt, "Alt", ui);

            self.draw_single_label_key(map, Vec2 { x: 378., y: 50. }, rdev::Key::Space, " ", ui);

            self.draw_single_label_key(map, ctrl_size, rdev::Key::AltGr, "Alt", ui);
            self.draw_single_label_key(map, ctrl_size, rdev::Key::Function, "Fn", ui);
            // no menu in rdev::Key
            self.draw_single_label_key(map, ctrl_size, rdev::Key::Unknown(110), "Menu", ui);
            self.draw_single_label_key(map, ctrl_size, rdev::Key::ControlRight, "Ctrl", ui);

            ui.add_space(SECTION_SPACE);
            self.draw_single_label_key(map, basic_size, rdev::Key::LeftArrow, "←", ui);
            self.draw_single_label_key(map, basic_size, rdev::Key::UpArrow, "↑", ui);
            self.draw_single_label_key(map, basic_size, rdev::Key::RightArrow, "→", ui);
        });
    }
    fn draw_double_labels_key(
        &mut self,
        map: &MutexGuard<PressTimesMap>,
        size: Vec2,
        key: rdev::Key,
        top_name: &str,
        bottom_name: &str,
        ui: &mut Ui,
    ) {
        let times = map.get_key_times(key);
        let mut key = KeyBox::new(
            size,
            KeyTextsLayout::TopBottom((top_name.to_string(), bottom_name.to_string())),
            key,
            times,
            self.hue,
        );
        key.ui(ui);
    }

    fn draw_single_label_key(
        &mut self,
        map: &MutexGuard<PressTimesMap>,
        size: Vec2,
        key: rdev::Key,
        name: &str,
        ui: &mut Ui,
    ) {
        let times = map.get_key_times(key);
        let mut key = KeyBox::new(
            size,
            KeyTextsLayout::Center1(name.to_string()),
            key,
            times,
            self.hue,
        );
        key.ui(ui);
    }

    fn draw_empty_key(&mut self, size: Vec2, ui: &mut Ui) {
        let (rect, _) = ui.allocate_exact_size(size, Sense::hover());
        ui.painter().rect_filled(rect, 0., Color32::TRANSPARENT);
    }
}
