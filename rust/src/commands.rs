//! IPC commands. `capabilities/teams.json` and `mail.json` expose only `notify`,
//! `open_external`, `set_zoom`, `open_window` and `switch_view` to the remote
//! pages; the rest are for the bundled rail, settings and join windows.

use crate::{
    config::Config,
    links, open_window as open_ui_window, show_main, usage,
    views::{self, View},
    AppState, MAIN,
};
use serde::Serialize;
use tauri::{AppHandle, Emitter, Manager, State, Webview};
use tauri_plugin_notification::NotificationExt;
use tauri_plugin_opener::OpenerExt;

fn truncate(text: &str, max: usize) -> String {
    match text.char_indices().nth(max) {
        Some((i, _)) => format!("{}…", &text[..i]),
        None => text.to_string(),
    }
}

#[tauri::command]
pub fn notify(app: AppHandle, webview: Webview, state: State<AppState>, title: String, body: String) {
    let source = View::parse(webview.label());
    let on_screen = source == Some(state.views().active)
        && app
            .get_window(MAIN)
            .is_some_and(|w| w.is_visible().unwrap_or(false) && w.is_focused().unwrap_or(false));
    if on_screen {
        return;
    }
    if source == Some(View::Mail) && state.views().active != View::Mail {
        state.update_views(|v| v.mail_attention = true);
        views::notify_rail(&app);
    }

    let config = state.config();
    if !config.notifications {
        return;
    }
    let body = if config.notification_preview { truncate(&body, 200) } else { "New activity in Teams".into() };
    let result = app.notification().builder().title(truncate(&title, 80)).body(body).show();
    if let Err(err) = result {
        eprintln!("[notify] failed to show notification: {err}");
    }
}

#[tauri::command]
pub fn open_external(app: AppHandle, url: String) -> Result<(), String> {
    let parsed = url::Url::parse(&url).map_err(|_| "Invalid link".to_string())?;
    if !links::is_safe_external(&parsed) {
        return Err("Blocked link scheme".into());
    }
    app.opener().open_url(parsed.as_str(), None::<&str>).map_err(|e| e.to_string())
}

/// `step` is +1 / -1 to zoom in or out by 10%, 0 to reset.
#[tauri::command]
pub fn set_zoom(app: AppHandle, state: State<AppState>, step: i32) -> f64 {
    let zoom = state.update(|c| {
        c.zoom = match step {
            0 => 1.0,
            s => (((c.zoom + 0.1 * s.signum() as f64) * 10.0).round() / 10.0).clamp(0.5, 3.0),
        };
        c.zoom
    });
    for view in [View::Teams, View::Mail] {
        if let Some(webview) = views::content(&app, view) {
            let _ = webview.set_zoom(zoom);
        }
    }
    state.persist();
    let _ = app.emit("config-changed", state.config());
    zoom
}

#[tauri::command]
pub fn open_window(app: AppHandle, name: String) -> Result<(), String> {
    match name.as_str() {
        "settings" | "join" => {
            open_ui_window(&app, &name);
            Ok(())
        }
        _ => Err("Unknown window".into()),
    }
}

#[tauri::command]
pub fn switch_view(app: AppHandle, name: String) -> Result<(), String> {
    let view = View::parse(&name).ok_or("Unknown view")?;
    views::switch(&app, view)?;
    show_main(&app);
    Ok(())
}

#[tauri::command]
pub fn rail_state(app: AppHandle) -> views::RailState {
    views::rail_state(&app)
}

#[tauri::command]
pub fn get_config(state: State<AppState>) -> Config {
    state.config()
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SaveResult {
    restart_required: bool,
}

#[tauri::command]
pub fn save_config(app: AppHandle, state: State<AppState>, config: Config) -> Result<SaveResult, String> {
    let config = config.sanitized();
    crate::config::save(&config).map_err(|e| format!("Could not save settings: {e}"))?;
    state.update(|c| *c = config.clone());

    views::apply_live(&app, &config);
    crate::tray::set_unread(&app, state.unread());
    let _ = app.emit("config-changed", &config);
    Ok(SaveResult { restart_required: state.launched.needs_restart(&config) })
}

#[tauri::command]
pub fn join_meeting(app: AppHandle, link: String) -> Result<(), String> {
    let url = links::meeting_url(&link).ok_or("That doesn't look like a Teams meeting link.")?;
    let teams = views::content(&app, View::Teams).ok_or("Teams is not available.")?;
    teams.navigate(url).map_err(|e| e.to_string())?;
    views::switch(&app, View::Teams)?;
    show_main(&app);
    if let Some(join) = app.get_webview_window("join") {
        let _ = join.close();
    }
    Ok(())
}

#[tauri::command]
pub fn resource_usage() -> usage::Usage {
    usage::sample()
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AppInfo {
    version: &'static str,
    tauri: &'static str,
    webkit: String,
}

#[tauri::command]
pub fn app_info() -> AppInfo {
    #[cfg(target_os = "linux")]
    // SAFETY: plain getters with no arguments or preconditions.
    let webkit = unsafe {
        use webkit2gtk::ffi;
        format!(
            "{}.{}.{}",
            ffi::webkit_get_major_version(),
            ffi::webkit_get_minor_version(),
            ffi::webkit_get_micro_version()
        )
    };
    #[cfg(not(target_os = "linux"))]
    let webkit = String::from("system");
    AppInfo { version: env!("CARGO_PKG_VERSION"), tauri: tauri::VERSION, webkit }
}

#[tauri::command]
pub fn restart(app: AppHandle) {
    app.restart();
}
