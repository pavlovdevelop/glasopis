//! The application windows: the settings window and the floating overlay.

use tauri::{AppHandle, Manager, WebviewUrl, WebviewWindow, WebviewWindowBuilder};

use crate::errors::{GlasopisError, Result};

pub const MAIN_WINDOW: &str = "main";
pub const OVERLAY_WINDOW: &str = "overlay";

const OVERLAY_WIDTH: f64 = 260.0;
const OVERLAY_HEIGHT: f64 = 96.0;
/// Distance from the bottom edge of the work area.
const OVERLAY_BOTTOM_MARGIN: f64 = 80.0;

/// Shows (and creates, if needed) the settings window.
pub fn show_main_window(app: &AppHandle) -> Result<WebviewWindow> {
    if let Some(window) = app.get_webview_window(MAIN_WINDOW) {
        let _ = window.show();
        let _ = window.unminimize();
        let _ = window.set_focus();
        return Ok(window);
    }
    let window = WebviewWindowBuilder::new(app, MAIN_WINDOW, WebviewUrl::App("index.html".into()))
        .title("Glasopis")
        .inner_size(980.0, 720.0)
        .min_inner_size(820.0, 600.0)
        .center()
        .resizable(true)
        .visible(true)
        .build()
        .map_err(|e| GlasopisError::other(format!("Прозорецът не може да бъде отворен: {e}")))?;
    Ok(window)
}

/// Opens the settings window on a specific page (used by the tray menu).
pub fn show_main_window_at(app: &AppHandle, route: &str) -> Result<()> {
    let window = show_main_window(app)?;
    let _ = window.eval(format!("window.location.hash = '{route}'"));
    Ok(())
}

/// Creates the floating overlay. It never takes focus, so the application the
/// user was typing in keeps it.
pub fn ensure_overlay(app: &AppHandle) -> Result<WebviewWindow> {
    if let Some(window) = app.get_webview_window(OVERLAY_WINDOW) {
        return Ok(window);
    }
    let window = WebviewWindowBuilder::new(
        app,
        OVERLAY_WINDOW,
        WebviewUrl::App("index.html#/overlay".into()),
    )
    .title("Glasopis")
    .inner_size(OVERLAY_WIDTH, OVERLAY_HEIGHT)
    .decorations(false)
    .transparent(true)
    .always_on_top(true)
    .skip_taskbar(true)
    .resizable(false)
    .shadow(false)
    .focused(false)
    .visible(false)
    .build()
    .map_err(|e| {
        GlasopisError::other(format!("Плаващият прозорец не може да бъде създаден: {e}"))
    })?;
    position_overlay(&window);
    Ok(window)
}

fn position_overlay(window: &WebviewWindow) {
    // Bottom centre of the monitor the cursor is on, like a system toast.
    if let Ok(Some(monitor)) = window.current_monitor() {
        let size = monitor.size().to_logical::<f64>(monitor.scale_factor());
        let position = monitor.position().to_logical::<f64>(monitor.scale_factor());
        let x = position.x + (size.width - OVERLAY_WIDTH) / 2.0;
        let y = position.y + size.height - OVERLAY_HEIGHT - OVERLAY_BOTTOM_MARGIN;
        let _ = window.set_position(tauri::LogicalPosition::new(x, y));
    }
}

pub fn show_overlay(app: &AppHandle) {
    match ensure_overlay(app) {
        Ok(window) => {
            position_overlay(&window);
            // `show` must not steal the focus of the target application.
            let _ = window.show();
            let _ = window.set_always_on_top(true);
        }
        Err(err) => log::warn!("плаващият прозорец не беше показан: {err}"),
    }
}

pub fn hide_overlay(app: &AppHandle) {
    if let Some(window) = app.get_webview_window(OVERLAY_WINDOW) {
        let _ = window.hide();
    }
}
