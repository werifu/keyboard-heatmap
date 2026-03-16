use eframe::egui;
use std::{error::Error, sync::OnceLock};
use tray_icon::{
    menu::{Menu, MenuEvent, MenuItem},
    Icon, MouseButton, MouseButtonState, TrayIcon, TrayIconBuilder, TrayIconEvent,
};

const TRAY_ICON_WIDTH: u32 = 16;
const TRAY_ICON_HEIGHT: u32 = 16;
const TOGGLE_WINDOW_ID: &str = "toggle-window";
const QUIT_ID: &str = "quit";
static TRAY_COMMAND_SENDER: OnceLock<crossbeam_channel::Sender<TrayCommand>> = OnceLock::new();
static TRAY_COMMAND_RECEIVER: OnceLock<crossbeam_channel::Receiver<TrayCommand>> = OnceLock::new();

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TrayCommand {
    ToggleWindow,
    Quit,
}

pub struct TrayController {
    _tray_icon: TrayIcon,
}

impl TrayController {
    pub fn new() -> Result<Self, Box<dyn Error + Send + Sync>> {
        let menu = Menu::new();
        let toggle_window = MenuItem::with_id(TOGGLE_WINDOW_ID, "Show / Hide Window", true, None);
        let quit = MenuItem::with_id(QUIT_ID, "Quit", true, None);
        menu.append(&toggle_window)?;
        menu.append(&quit)?;

        let tray_icon = TrayIconBuilder::new()
            .with_menu(Box::new(menu))
            .with_icon(tray_icon_image()?)
            .with_icon_as_template(cfg!(target_os = "macos"))
            .with_tooltip("Keyboard Heatmap")
            .with_menu_on_left_click(false)
            .build()?;

        Ok(Self {
            _tray_icon: tray_icon,
        })
    }

    pub fn install_repaint_forwarder(ctx: &egui::Context) {
        let (sender, receiver) = crossbeam_channel::unbounded();
        let _ = TRAY_COMMAND_SENDER.set(sender);
        let _ = TRAY_COMMAND_RECEIVER.set(receiver);

        let tray_ctx = ctx.clone();
        TrayIconEvent::set_event_handler(Some(move |event| {
            if let Some(command) = tray_event_command(&event) {
                send_tray_command(command);
            }
            tray_ctx.request_repaint();
        }));

        let menu_ctx = ctx.clone();
        MenuEvent::set_event_handler(Some(move |event| {
            if let Some(command) = menu_event_command(&event) {
                send_tray_command(command);
            }
            menu_ctx.request_repaint();
        }));
    }

    pub fn poll(&self) -> Vec<TrayCommand> {
        let mut commands = Vec::new();
        if let Some(receiver) = TRAY_COMMAND_RECEIVER.get() {
            while let Ok(command) = receiver.try_recv() {
                commands.push(command);
            }
        }

        commands
    }
}

fn tray_event_command(event: &TrayIconEvent) -> Option<TrayCommand> {
    match event {
        TrayIconEvent::Click {
            button: MouseButton::Left,
            button_state: MouseButtonState::Up,
            ..
        }
        | TrayIconEvent::DoubleClick {
            button: MouseButton::Left,
            ..
        } => Some(TrayCommand::ToggleWindow),
        _ => None,
    }
}

fn menu_event_command(event: &MenuEvent) -> Option<TrayCommand> {
    if event.id().0 == TOGGLE_WINDOW_ID {
        Some(TrayCommand::ToggleWindow)
    } else if event.id().0 == QUIT_ID {
        Some(TrayCommand::Quit)
    } else {
        None
    }
}

fn tray_icon_image() -> Result<Icon, tray_icon::BadIcon> {
    Icon::from_rgba(tray_icon_rgba(), TRAY_ICON_WIDTH, TRAY_ICON_HEIGHT)
}

fn tray_icon_rgba() -> Vec<u8> {
    let mut rgba = vec![0; (TRAY_ICON_WIDTH * TRAY_ICON_HEIGHT * 4) as usize];

    for y in 2..14 {
        for x in 1..15 {
            let border = y == 2 || y == 13 || x == 1 || x == 14;
            let key_area = (4..=6).contains(&y) || (9..=11).contains(&y);
            let alpha = if border || key_area { 255 } else { 0 };
            let shade = if key_area { 70 } else { 40 };
            let index = ((y * TRAY_ICON_WIDTH + x) * 4) as usize;
            rgba[index] = shade;
            rgba[index + 1] = shade;
            rgba[index + 2] = shade;
            rgba[index + 3] = alpha;
        }
    }

    rgba
}

fn send_tray_command(command: TrayCommand) {
    if let Some(sender) = TRAY_COMMAND_SENDER.get() {
        let _ = sender.send(command);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn tray_icon_pixels_match_expected_size() {
        let rgba = tray_icon_rgba();
        assert_eq!(
            rgba.len(),
            (TRAY_ICON_WIDTH * TRAY_ICON_HEIGHT * 4) as usize
        );
    }

    #[test]
    fn tray_left_click_toggles_window() {
        let event = TrayIconEvent::Click {
            id: Default::default(),
            position: tray_icon::dpi::PhysicalPosition::new(0.0, 0.0),
            rect: tray_icon::Rect::default(),
            button: MouseButton::Left,
            button_state: MouseButtonState::Up,
        };

        assert_eq!(tray_event_command(&event), Some(TrayCommand::ToggleWindow));
    }
}
