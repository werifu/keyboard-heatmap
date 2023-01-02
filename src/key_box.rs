use egui::{Vec2, Ui, Sense, Color32, Align2, Stroke, RichText};

use crate::color::{get_color, get_strike_color};

/// Layout in a key box, shows how to display the key contents
#[derive(Clone)]
pub enum KeyTextsLayout {
    TopBottom((String, String)),
    Center1(String),
}
/// component of a key on a keyboard
pub struct KeyBox {
    size: Vec2,
    rounding: f32,
    stroke_width: f32,
    layout: KeyTextsLayout,
    key: rdev::Key,
    press_times: u32,
    hue: f32,
}

impl KeyBox {
    pub fn new(
        size: Vec2,
        texts: KeyTextsLayout,
        key: rdev::Key,
        press_times: u32,
        hue: f32,
    ) -> KeyBox {
        Self {
            size,
            rounding: 5.0,
            stroke_width: 2.0,
            layout: texts,
            key,
            press_times,
            hue,
        }
    }
}
impl KeyBox {
    pub fn ui(&mut self, ui: &mut Ui) {
        let (rect, resp) = ui.allocate_exact_size(self.size, Sense::hover());
        let filled_color = get_color(self.hue, self.press_times);
        ui.painter().rect_filled(rect, self.rounding, filled_color);
        match &self.layout {
            KeyTextsLayout::TopBottom(top_bottom) => {
                ui.painter().text(
                    rect.center(),
                    Align2::CENTER_BOTTOM,
                    top_bottom.0.clone(),
                    egui::FontId::monospace(13.),
                    Color32::from_rgb(32, 5, 64),
                );
                ui.painter().text(
                    rect.center(),
                    Align2::CENTER_TOP,
                    top_bottom.1.clone(),
                    egui::FontId::monospace(13.),
                    Color32::from_rgb(32, 5, 64),
                );
            }
            KeyTextsLayout::Center1(text) => {
                ui.painter().text(
                    rect.center(),
                    Align2::CENTER_CENTER,
                    text,
                    egui::FontId::monospace(13.),
                    Color32::from_rgb(32, 5, 64),
                );
            }
        }

        ui.painter().rect_stroke(
            rect,
            self.rounding,
            Stroke {
                width: self.stroke_width,
                color: get_strike_color(filled_color),
            },
        );

        let hover_ui = |ui: &mut Ui| {
            ui.label(RichText::new(format!("{}", self.press_times)));
        };
        resp.on_hover_ui(hover_ui);
    }
}
