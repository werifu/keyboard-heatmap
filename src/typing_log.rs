use std::collections::VecDeque;

use crate::keyboard::{self, KeyboardType};

const DEFAULT_LOG_CAPACITY: usize = 1024;
const PREVIEW_ITEMS: usize = 10;

struct LoggedKey {
    key: rdev::Key,
    fallback_name: Option<String>,
}

pub struct TypingLog {
    entries: VecDeque<LoggedKey>,
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

        self.entries.push_back(LoggedKey {
            key,
            fallback_name: event.name.clone(),
        });
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

    pub fn preview(&self, keyboard_type: KeyboardType) -> String {
        let preview_len = self.entries.len().saturating_sub(PREVIEW_ITEMS);
        self.entries
            .iter()
            .skip(preview_len)
            .map(|entry| format_key_event(entry, keyboard_type))
            .collect::<Vec<_>>()
            .join(" | ")
    }
}

fn format_key_event(entry: &LoggedKey, keyboard_type: KeyboardType) -> String {
    if let Some(label) = keyboard::key_preview_label(keyboard_type, entry.key) {
        return label;
    }

    match entry
        .fallback_name
        .as_deref()
        .map(str::trim)
        .filter(|value| !value.is_empty())
    {
        Some(" ") => "Space".to_string(),
        Some("\n") => "Enter".to_string(),
        Some("\t") => "Tab".to_string(),
        Some(value) => value.to_string(),
        None => format!("<{:?}>", entry.key),
    }
}
