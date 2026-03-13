use std::collections::VecDeque;

use crate::{
    key_box::KeyTextsLayout,
    keyboard::{self, KeyPreviewSpec, KeyboardType},
};

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

    pub fn preview_keycaps(&self, keyboard_type: KeyboardType) -> Vec<KeyPreviewSpec> {
        let preview_len = self.entries.len().saturating_sub(PREVIEW_ITEMS);
        self.entries
            .iter()
            .skip(preview_len)
            .map(|entry| keycap_for_event(entry, keyboard_type))
            .collect()
    }
}

fn keycap_for_event(entry: &LoggedKey, keyboard_type: KeyboardType) -> KeyPreviewSpec {
    if let Some(spec) = keyboard::key_preview_spec(keyboard_type, entry.key) {
        return spec;
    }

    match entry
        .fallback_name
        .as_deref()
        .map(str::trim)
        .filter(|value| !value.is_empty())
    {
        Some(" ") => fallback_keycap("Space"),
        Some("\n") => fallback_keycap("Enter"),
        Some("\t") => fallback_keycap("Tab"),
        Some(value) => fallback_keycap(value),
        None => fallback_keycap(&format!("<{:?}>", entry.key)),
    }
}

fn fallback_keycap(text: &str) -> KeyPreviewSpec {
    let width_units = match text.len() {
        0..=2 => 1.0,
        3..=4 => 1.2,
        5..=6 => 1.5,
        _ => 1.8,
    };
    KeyPreviewSpec {
        layout: KeyTextsLayout::Center1(text.to_string()),
        width_units,
    }
}
