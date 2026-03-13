use std::collections::VecDeque;

const DEFAULT_LOG_CAPACITY: usize = 1024;
const PREVIEW_ITEMS: usize = 24;

pub struct TypingLog {
    entries: VecDeque<String>,
    capacity: usize,
}

impl TypingLog {
    pub fn new() -> Self {
        Self::with_capacity(DEFAULT_LOG_CAPACITY)
    }

    pub fn with_capacity(capacity: usize) -> Self {
        Self {
            entries: VecDeque::with_capacity(capacity),
            capacity,
        }
    }

    pub fn push_event(&mut self, event: &rdev::Event) {
        let rdev::EventType::KeyPress(key) = event.event_type else {
            return;
        };

        if self.entries.len() == self.capacity {
            self.entries.pop_front();
        }

        self.entries
            .push_back(format_key_event(key, event.name.as_deref()));
    }

    pub fn clear(&mut self) {
        self.entries.clear();
    }

    pub fn len(&self) -> usize {
        self.entries.len()
    }

    pub fn capacity(&self) -> usize {
        self.capacity
    }

    pub fn preview(&self) -> String {
        let preview_len = self.entries.len().saturating_sub(PREVIEW_ITEMS);
        self.entries
            .iter()
            .skip(preview_len)
            .cloned()
            .collect::<Vec<_>>()
            .join(" ")
    }
}

fn format_key_event(key: rdev::Key, name: Option<&str>) -> String {
    match name.map(str::trim).filter(|value| !value.is_empty()) {
        Some(" ") => "Space".to_string(),
        Some("\n") => "Enter".to_string(),
        Some("\t") => "Tab".to_string(),
        Some(value) => value.to_string(),
        None => format!("<{key:?}>"),
    }
}
