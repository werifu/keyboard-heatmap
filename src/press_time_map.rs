use std::collections::HashMap;

pub struct PressTimesMap {
    pub map: HashMap<rdev::Key, u32>,
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

    pub fn total_presses(&self) -> u64 {
        self.map.values().map(|&count| u64::from(count)).sum()
    }

    pub fn persisted_entries(&self) -> Vec<(String, u32)> {
        let mut entries: Vec<_> = self
            .map
            .iter()
            .map(|(&key, &count)| (key_to_id(key), count))
            .collect();
        entries.sort_by(|a, b| a.0.cmp(&b.0));
        entries
    }

    pub fn from_persisted_entries(entries: Vec<(String, u32)>) -> Self {
        let mut map = HashMap::new();
        for (id, count) in entries {
            if let Some(key) = key_from_id(&id) {
                map.insert(key, count);
            }
        }
        Self { map }
    }
}

fn key_to_id(key: rdev::Key) -> String {
    match key {
        rdev::Key::Unknown(code) => format!("Unknown:{code}"),
        _ => format!("{key:?}"),
    }
}

fn key_from_id(id: &str) -> Option<rdev::Key> {
    if let Some(code) = id.strip_prefix("Unknown:") {
        return code.parse().ok().map(rdev::Key::Unknown);
    }

    Some(match id {
        "Alt" => rdev::Key::Alt,
        "AltGr" => rdev::Key::AltGr,
        "BackQuote" => rdev::Key::BackQuote,
        "BackSlash" => rdev::Key::BackSlash,
        "Backspace" => rdev::Key::Backspace,
        "CapsLock" => rdev::Key::CapsLock,
        "Comma" => rdev::Key::Comma,
        "ControlLeft" => rdev::Key::ControlLeft,
        "ControlRight" => rdev::Key::ControlRight,
        "Delete" => rdev::Key::Delete,
        "Dot" => rdev::Key::Dot,
        "DownArrow" => rdev::Key::DownArrow,
        "End" => rdev::Key::End,
        "Equal" => rdev::Key::Equal,
        "Escape" => rdev::Key::Escape,
        "F1" => rdev::Key::F1,
        "F10" => rdev::Key::F10,
        "F11" => rdev::Key::F11,
        "F12" => rdev::Key::F12,
        "F2" => rdev::Key::F2,
        "F3" => rdev::Key::F3,
        "F4" => rdev::Key::F4,
        "F5" => rdev::Key::F5,
        "F6" => rdev::Key::F6,
        "F7" => rdev::Key::F7,
        "F8" => rdev::Key::F8,
        "F9" => rdev::Key::F9,
        "Function" => rdev::Key::Function,
        "Home" => rdev::Key::Home,
        "Insert" => rdev::Key::Insert,
        "KeyA" => rdev::Key::KeyA,
        "KeyB" => rdev::Key::KeyB,
        "KeyC" => rdev::Key::KeyC,
        "KeyD" => rdev::Key::KeyD,
        "KeyE" => rdev::Key::KeyE,
        "KeyF" => rdev::Key::KeyF,
        "KeyG" => rdev::Key::KeyG,
        "KeyH" => rdev::Key::KeyH,
        "KeyI" => rdev::Key::KeyI,
        "KeyJ" => rdev::Key::KeyJ,
        "KeyK" => rdev::Key::KeyK,
        "KeyL" => rdev::Key::KeyL,
        "KeyM" => rdev::Key::KeyM,
        "KeyN" => rdev::Key::KeyN,
        "KeyO" => rdev::Key::KeyO,
        "KeyP" => rdev::Key::KeyP,
        "KeyQ" => rdev::Key::KeyQ,
        "KeyR" => rdev::Key::KeyR,
        "KeyS" => rdev::Key::KeyS,
        "KeyT" => rdev::Key::KeyT,
        "KeyU" => rdev::Key::KeyU,
        "KeyV" => rdev::Key::KeyV,
        "KeyW" => rdev::Key::KeyW,
        "KeyX" => rdev::Key::KeyX,
        "KeyY" => rdev::Key::KeyY,
        "KeyZ" => rdev::Key::KeyZ,
        "LeftArrow" => rdev::Key::LeftArrow,
        "LeftBracket" => rdev::Key::LeftBracket,
        "MetaLeft" => rdev::Key::MetaLeft,
        "MetaRight" => rdev::Key::MetaRight,
        "Minus" => rdev::Key::Minus,
        "Num0" => rdev::Key::Num0,
        "Num1" => rdev::Key::Num1,
        "Num2" => rdev::Key::Num2,
        "Num3" => rdev::Key::Num3,
        "Num4" => rdev::Key::Num4,
        "Num5" => rdev::Key::Num5,
        "Num6" => rdev::Key::Num6,
        "Num7" => rdev::Key::Num7,
        "Num8" => rdev::Key::Num8,
        "Num9" => rdev::Key::Num9,
        "PageDown" => rdev::Key::PageDown,
        "PageUp" => rdev::Key::PageUp,
        "Pause" => rdev::Key::Pause,
        "PrintScreen" => rdev::Key::PrintScreen,
        "Quote" => rdev::Key::Quote,
        "Return" => rdev::Key::Return,
        "RightArrow" => rdev::Key::RightArrow,
        "RightBracket" => rdev::Key::RightBracket,
        "ScrollLock" => rdev::Key::ScrollLock,
        "SemiColon" => rdev::Key::SemiColon,
        "ShiftLeft" => rdev::Key::ShiftLeft,
        "ShiftRight" => rdev::Key::ShiftRight,
        "Slash" => rdev::Key::Slash,
        "Space" => rdev::Key::Space,
        "Tab" => rdev::Key::Tab,
        "UpArrow" => rdev::Key::UpArrow,
        _ => return None,
    })
}
