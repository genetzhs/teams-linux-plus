//! System tray icon with menu and an unread badge drawn directly into the RGBA
//! buffer (no image crate, no extra allocations beyond one icon-sized copy).

use crate::{open_window, show_main, views, AppState};
use tauri::{
    image::Image,
    menu::{Menu, MenuItem, PredefinedMenuItem},
    tray::{MouseButton, MouseButtonState, TrayIconBuilder, TrayIconEvent},
    AppHandle, Manager,
};

const TRAY_ID: &str = "main";
const TRAY_PNG: &[u8] = include_bytes!("../icons/tray.png");

pub fn create(app: &AppHandle) -> tauri::Result<()> {
    let menu = Menu::with_items(
        app,
        &[
            &MenuItem::with_id(app, "show", "Open Teams", true, None::<&str>)?,
            &MenuItem::with_id(app, "mail", "Open Mail", true, None::<&str>)?,
            &MenuItem::with_id(app, "join", "Join meeting…", true, None::<&str>)?,
            &PredefinedMenuItem::separator(app)?,
            &MenuItem::with_id(app, "settings", "Settings", true, None::<&str>)?,
            &PredefinedMenuItem::separator(app)?,
            &MenuItem::with_id(app, "quit", "Quit", true, None::<&str>)?,
        ],
    )?;

    TrayIconBuilder::with_id(TRAY_ID)
        .icon(Image::from_bytes(TRAY_PNG)?)
        .tooltip("Teams for Linux")
        .menu(&menu)
        .show_menu_on_left_click(false)
        .on_menu_event(|app, event| match event.id.as_ref() {
            "show" | "mail" => {
                let view = if event.id.as_ref() == "mail" { views::View::Mail } else { views::View::Teams };
                let _ = views::switch(app, view);
                show_main(app);
            }
            "join" => open_window(app, "join"),
            "settings" => open_window(app, "settings"),
            "quit" => app.exit(0),
            _ => {}
        })
        .on_tray_icon_event(|tray, event| {
            // Not delivered by AppIndicator on most Linux desktops; the menu
            // covers that case.
            if let TrayIconEvent::Click { button: MouseButton::Left, button_state: MouseButtonState::Up, .. } = event {
                show_main(tray.app_handle());
            }
        })
        .build(app)?;
    Ok(())
}

pub fn set_unread(app: &AppHandle, count: u32) {
    let Some(tray) = app.tray_by_id(TRAY_ID) else { return };
    let badge = app.state::<AppState>().config().tray_badge;
    let icon = Image::from_bytes(TRAY_PNG).map(|base| if count > 0 && badge { with_badge(&base) } else { base });
    if let Ok(icon) = icon {
        let _ = tray.set_icon(Some(icon));
    }
    let tooltip = match count {
        0 => "Teams for Linux".to_string(),
        1 => "Teams for Linux — 1 unread".to_string(),
        n => format!("Teams for Linux — {n} unread"),
    };
    let _ = tray.set_tooltip(Some(&tooltip));
}

/// Red dot with a white ring in the top-right corner, antialiased by coverage.
fn with_badge(base: &Image<'_>) -> Image<'static> {
    let (w, h) = (base.width(), base.height());
    let mut px = base.rgba().to_vec();
    let size = w.min(h) as f32;
    let r = size * 0.24;
    let ring = size * 0.05;
    let (cx, cy) = (w as f32 - r - ring, r + ring);

    for y in 0..h {
        for x in 0..w {
            let d = ((x as f32 + 0.5 - cx).powi(2) + (y as f32 + 0.5 - cy).powi(2)).sqrt();
            let outer = (r + ring + 0.5 - d).clamp(0.0, 1.0);
            if outer == 0.0 {
                continue;
            }
            let inner = (r + 0.5 - d).clamp(0.0, 1.0);
            // Fill colour blends the white ring into the #D13438 dot.
            let fill = [
                255.0 + (209.0 - 255.0) * inner,
                255.0 + (52.0 - 255.0) * inner,
                255.0 + (56.0 - 255.0) * inner,
            ];
            let i = ((y * w + x) * 4) as usize;
            for c in 0..3 {
                px[i + c] = (px[i + c] as f32 * (1.0 - outer) + fill[c] * outer).round() as u8;
            }
            px[i + 3] = px[i + 3].max((outer * 255.0) as u8);
        }
    }
    Image::new_owned(px, w, h)
}

/// Teams prefixes the document title with the unread count, e.g. `(3) Chat | Microsoft Teams`.
pub fn unread_from_title(title: &str) -> u32 {
    title
        .strip_prefix('(')
        .and_then(|rest| rest.split_once(')'))
        .and_then(|(n, _)| n.parse().ok())
        .unwrap_or(0)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_title_count() {
        assert_eq!(unread_from_title("(12) Chat | Microsoft Teams"), 12);
        assert_eq!(unread_from_title("Chat | Microsoft Teams"), 0);
        assert_eq!(unread_from_title("(x) Chat"), 0);
    }
}
