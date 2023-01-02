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
}
