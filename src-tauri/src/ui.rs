//! The application windows: the settings window and the floating overlay.

use tauri::{AppHandle, Manager, WebviewUrl, WebviewWindow, WebviewWindowBuilder};

use crate::errors::{GlasopisError, Result};
use crate::state::AppState;

pub const MAIN_WINDOW: &str = "main";
pub const OVERLAY_WINDOW: &str = "overlay";

/// A small draggable badge, not a window-sized panel — big enough for the
/// icon and a contained level glow (which must not exceed the window, or it
/// gets visibly clipped), small enough to stay out of the way.
const OVERLAY_SIZE: f64 = 72.0;
/// Distance from the bottom edge of the monitor, used only the first time the
/// overlay is shown (before the user has dragged it anywhere). `monitor.size()`
/// is the full display resolution, not the work area — this has to clear the
/// Windows taskbar (and a taller one, at 150%+ scaling) on its own.
const OVERLAY_BOTTOM_MARGIN: f64 = 160.0;

/// Shows (and creates, if needed) the settings window.
pub fn show_main_window(app: &AppHandle) -> Result<WebviewWindow> {
    if let Some(window) = app.get_webview_window(MAIN_WINDOW) {
        let _ = window.show();
        let _ = window.unminimize();
        bring_to_front(&window);
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
    bring_to_front(&window);
    Ok(window)
}

/// `set_focus` alone often loses to Windows' foreground-lock: a process that
/// wasn't just given input (e.g. relaunched by the updater's installer,
/// rather than clicked by the user) is not allowed to steal focus from
/// whatever window is currently active. Briefly forcing the window topmost
/// bypasses that restriction, unlike `SetForegroundWindow`.
fn bring_to_front(window: &WebviewWindow) {
    let _ = window.set_focus();
    let _ = window.set_always_on_top(true);
    let _ = window.set_always_on_top(false);
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
    .inner_size(OVERLAY_SIZE, OVERLAY_SIZE)
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
    // Positioned once, here, at creation: after that the window stays where
    // the user last dragged it (`show_overlay` never re-centres it).
    let saved = app.state::<AppState>().settings().general.overlay_position;
    position_overlay(&window, saved);
    Ok(window)
}

/// Places the overlay at `saved` (clamped to the current monitor, in case the
/// screen configuration changed since it was saved) or, the first time, at
/// the bottom centre of the monitor the cursor is on — like a system toast.
fn position_overlay(window: &WebviewWindow, saved: Option<(i32, i32)>) {
    let Ok(Some(monitor)) = window.current_monitor() else {
        return;
    };
    let size = monitor.size().to_logical::<f64>(monitor.scale_factor());
    let position = monitor.position().to_logical::<f64>(monitor.scale_factor());

    let (x, y) = match saved {
        Some((x, y)) => {
            let max_x = position.x + size.width - OVERLAY_SIZE;
            let max_y = position.y + size.height - OVERLAY_SIZE;
            (
                (x as f64).clamp(position.x, max_x.max(position.x)),
                (y as f64).clamp(position.y, max_y.max(position.y)),
            )
        }
        None => (
            position.x + (size.width - OVERLAY_SIZE) / 2.0,
            position.y + size.height - OVERLAY_SIZE - OVERLAY_BOTTOM_MARGIN,
        ),
    };
    let _ = window.set_position(tauri::LogicalPosition::new(x, y));
}

pub fn show_overlay(app: &AppHandle) {
    match ensure_overlay(app) {
        Ok(window) => {
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
