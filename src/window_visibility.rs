use eframe::{egui, Frame};

#[cfg(target_os = "windows")]
use raw_window_handle::{HasWindowHandle, RawWindowHandle};

pub fn set_window_visibility(_frame: &Frame, ctx: &egui::Context, visible: bool) {
    #[cfg(target_os = "windows")]
    windows::set_native_window_visibility(_frame, visible);

    ctx.send_viewport_cmd(egui::ViewportCommand::Visible(visible));
    if visible {
        ctx.send_viewport_cmd(egui::ViewportCommand::Minimized(false));
        ctx.send_viewport_cmd(egui::ViewportCommand::Focus);
    }
}

#[cfg(target_os = "windows")]
mod windows {
    use super::*;
    use windows_sys::Win32::{
        Foundation::HWND,
        UI::WindowsAndMessaging::{SetForegroundWindow, ShowWindow, SW_HIDE, SW_RESTORE},
    };

    pub fn set_native_window_visibility(frame: &Frame, visible: bool) {
        let Ok(handle) = frame.window_handle() else {
            return;
        };

        let RawWindowHandle::Win32(handle) = handle.as_raw() else {
            return;
        };

        let hwnd = handle.hwnd.get() as HWND;
        unsafe {
            ShowWindow(hwnd, if visible { SW_RESTORE } else { SW_HIDE });
            if visible {
                SetForegroundWindow(hwnd);
            }
        }
    }
}
