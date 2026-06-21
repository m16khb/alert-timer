use crate::models::OverlayFramePayload;
use tauri::{AppHandle, Emitter, Manager, PhysicalPosition, PhysicalSize};

pub fn configure(app: &AppHandle) -> tauri::Result<()> {
    let Some(window) = app.get_webview_window("overlay") else {
        return Ok(());
    };

    let _ = window.set_shadow(false);
    if let Some(monitor) = window.primary_monitor()? {
        let position = monitor.position();
        let size = monitor.size();
        window.set_size(PhysicalSize::new(size.width, size.height))?;
        window.set_position(PhysicalPosition::new(position.x, position.y))?;
        window.set_size(PhysicalSize::new(size.width, size.height))?;
    }

    let _ = window.set_ignore_cursor_events(true);
    window.hide()?;
    Ok(())
}

pub fn publish(app: &AppHandle, payload: Option<OverlayFramePayload>) {
    let Some(window) = app.get_webview_window("overlay") else {
        return;
    };

    match payload {
        Some(payload) => {
            let _ = window.show();
            let _ = window.emit("overlay://frame", payload);
        }
        None => {
            let _ = window.emit(
                "overlay://frame",
                OverlayFramePayload {
                    active: false,
                    color: "transparent".to_string(),
                    visible: false,
                    intensity: "none".to_string(),
                    border_thickness_px: 0,
                    alerts: Vec::new(),
                },
            );
            let _ = window.hide();
        }
    }
}
