//! The main window: a slim navigation rail plus one content webview per service.
//! Teams loads at startup; Outlook is created on first use so people who never
//! open Mail never pay for its WebKit process.

use crate::{config::Config, links, tray, webkit, AppState, BRIDGE_JS, MAIN};
use serde::Serialize;
use std::sync::atomic::Ordering;
use tauri::{
    webview::NewWindowResponse, AppHandle, Emitter, LogicalPosition, LogicalSize, Manager, UserAttentionType,
    Webview, WebviewBuilder, WebviewUrl, Window, WindowBuilder, Wry,
};
use tauri_plugin_opener::OpenerExt;

pub const RAIL: &str = "rail";
pub const TEAMS: &str = "teams";
pub const MAIL: &str = "mail";
const RAIL_WIDTH: f64 = 68.0;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Default)]
#[serde(rename_all = "lowercase")]
pub enum View {
    #[default]
    Teams,
    Mail,
}

impl View {
    pub fn parse(name: &str) -> Option<Self> {
        match name {
            TEAMS => Some(Self::Teams),
            MAIL => Some(Self::Mail),
            _ => None,
        }
    }

    pub fn label(self) -> &'static str {
        match self {
            Self::Teams => TEAMS,
            Self::Mail => MAIL,
        }
    }
}

#[derive(Debug, Default)]
pub struct ViewState {
    pub active: View,
    /// A mail notification arrived while Mail was not on screen.
    pub mail_attention: bool,
}

#[derive(Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct RailState {
    active: View,
    teams_unread: u32,
    mail_attention: bool,
    mail_enabled: bool,
}

pub fn rail_state(app: &AppHandle) -> RailState {
    let state = app.state::<AppState>();
    let views = state.views();
    RailState {
        active: views.active,
        teams_unread: state.unread(),
        mail_attention: views.mail_attention,
        mail_enabled: state.config().mail_enabled,
    }
}

pub fn notify_rail(app: &AppHandle) {
    let _ = app.emit_to(tauri::EventTarget::webview(RAIL), "rail-state", rail_state(app));
}

pub fn build(app: &AppHandle, config: &Config) -> tauri::Result<()> {
    let window = WindowBuilder::new(app, MAIN)
        .title("Teams for Linux")
        .inner_size(1320.0, 840.0)
        .min_inner_size(820.0, 540.0)
        .visible(!config.start_minimized)
        .build()?;
    let (width, height) = logical_size(&window);

    // On Linux child webviews are packed into the window's GtkBox and their
    // positions are ignored, so lay the box out horizontally and let GTK size
    // the rail and the visible content view.
    #[cfg(target_os = "linux")]
    {
        use gtk::prelude::OrientableExt;
        window.default_vbox()?.set_orientation(gtk::Orientation::Horizontal);
    }

    let rail = window.add_child(
        WebviewBuilder::new(RAIL, WebviewUrl::App("rail.html".into())),
        LogicalPosition::new(0.0, 0.0),
        LogicalSize::new(RAIL_WIDTH, height),
    )?;
    #[cfg(target_os = "linux")]
    rail.with_webview(|platform| {
        use gtk::prelude::{BoxExt, Cast, WidgetExt};
        let view = platform.inner();
        view.set_size_request(RAIL_WIDTH as i32, -1);
        if let Some(parent) = view.parent().and_then(|p| p.downcast::<gtk::Box>().ok()) {
            parent.set_child_packing(&view, false, true, 0, gtk::PackType::Start);
        }
    })?;
    #[cfg(not(target_os = "linux"))]
    let _ = rail;
    let teams = window.add_child(
        content_builder(app, View::Teams, &config.url, config),
        LogicalPosition::new(RAIL_WIDTH, 0.0),
        LogicalSize::new(width - RAIL_WIDTH, height),
    )?;
    prepare(&teams, config);
    Ok(())
}

fn prepare(webview: &Webview, config: &Config) {
    let _ = webview.set_zoom(config.zoom);
    webkit::tune(webview, config);
}

fn content_builder(app: &AppHandle, view: View, url: &str, config: &Config) -> WebviewBuilder<Wry> {
    let url: url::Url = url.parse().expect("sanitized config has valid URLs");
    let injected = serde_json::json!({ "customCss": config.custom_css, "view": view.label() });
    let opener = app.clone();

    let builder = WebviewBuilder::new(view.label(), WebviewUrl::External(url))
        .user_agent(config.effective_user_agent())
        .initialization_script(format!("window.__TFL__ = {injected};\n{BRIDGE_JS}"))
        .zoom_hotkeys_enabled(false)
        .on_new_window(move |url, _features| {
            if links::opens_in_app(&url) {
                return NewWindowResponse::Allow;
            }
            if links::is_safe_external(&url) {
                let _ = opener.opener().open_url(url.as_str(), None::<&str>);
            }
            NewWindowResponse::Deny
        });

    if view != View::Teams {
        return builder;
    }
    let app = app.clone();
    builder.on_document_title_changed(move |webview, title| {
        let count = tray::unread_from_title(&title);
        let previous = app.state::<AppState>().unread.swap(count, Ordering::Relaxed);
        if count == previous {
            return;
        }
        tray::set_unread(&app, count);
        notify_rail(&app);
        let window = webview.window();
        if count > previous && !window.is_focused().unwrap_or(true) {
            let _ = window.request_user_attention(Some(UserAttentionType::Informational));
        }
    })
}

fn logical_size(window: &Window) -> (f64, f64) {
    let scale = window.scale_factor().unwrap_or(1.0);
    window
        .inner_size()
        .map(|s| {
            let s = s.to_logical::<f64>(scale);
            (s.width, s.height)
        })
        .unwrap_or((1320.0, 840.0))
}

/// Keeps the rail at a fixed width and stretches the content views. GTK does
/// this itself on Linux (see `build`).
pub fn layout(window: &Window) {
    if cfg!(target_os = "linux") {
        return;
    }
    let (width, height) = logical_size(window);
    for webview in window.webviews() {
        if webview.label() == RAIL {
            let _ = webview.set_size(LogicalSize::new(RAIL_WIDTH, height));
        } else {
            let _ = webview.set_position(LogicalPosition::new(RAIL_WIDTH, 0.0));
            let _ = webview.set_size(LogicalSize::new((width - RAIL_WIDTH).max(1.0), height));
        }
    }
}

pub fn content(app: &AppHandle, view: View) -> Option<Webview> {
    app.get_webview(view.label())
}

pub fn switch(app: &AppHandle, view: View) -> Result<(), String> {
    let state = app.state::<AppState>();
    let config = state.config();
    if view == View::Mail && !config.mail_enabled {
        return Err("Mail is turned off in Settings.".into());
    }
    let window = app.get_window(MAIN).ok_or("Main window is not available.")?;

    let target = match content(app, view) {
        Some(webview) => webview,
        None => {
            let (width, height) = logical_size(&window);
            let webview = window
                .add_child(
                    content_builder(app, view, &config.mail_url, &config),
                    LogicalPosition::new(RAIL_WIDTH, 0.0),
                    LogicalSize::new(width - RAIL_WIDTH, height),
                )
                .map_err(|e| e.to_string())?;
            prepare(&webview, &config);
            webview
        }
    };

    for other in [View::Teams, View::Mail].into_iter().filter(|v| *v != view) {
        if let Some(webview) = content(app, other) {
            let _ = webview.hide();
        }
    }
    let _ = target.show();
    let _ = target.set_focus();

    state.update_views(|v| {
        v.active = view;
        if view == View::Mail {
            v.mail_attention = false;
        }
    });
    notify_rail(app);
    Ok(())
}

/// Applies settings that can change while running.
pub fn apply_live(app: &AppHandle, config: &Config) {
    for view in [View::Teams, View::Mail] {
        if let Some(webview) = content(app, view) {
            let _ = webview.set_zoom(config.zoom);
        }
    }
    if !config.mail_enabled {
        if app.state::<AppState>().views().active == View::Mail {
            let _ = switch(app, View::Teams);
        }
        // Free Outlook's web process instead of just hiding it.
        if let Some(mail) = content(app, View::Mail) {
            let _ = mail.close();
        }
    }
    notify_rail(app);
}
