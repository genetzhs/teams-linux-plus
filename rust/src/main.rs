//! Teams for Linux (Rust edition): a thin native shell around Teams and Outlook
//! on the system WebKitGTK, instead of a bundled Chromium.

#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod commands;
mod config;
mod links;
mod tray;
mod usage;
mod views;
mod webkit;

use config::Config;
use std::sync::{
    atomic::{AtomicU32, Ordering},
    Mutex, MutexGuard,
};
use tauri::{webview::PageLoadEvent, AppHandle, Manager, WebviewUrl, WebviewWindowBuilder, WindowEvent};
use views::{View, ViewState};

pub const MAIN: &str = "main";
pub const BRIDGE_JS: &str = include_str!("../inject/bridge.js");

pub struct AppState {
    config: Mutex<Config>,
    /// Config the process started with; restart-only options are compared against it.
    launched: Config,
    unread: AtomicU32,
    views: Mutex<ViewState>,
}

fn lock<T>(mutex: &Mutex<T>) -> MutexGuard<'_, T> {
    mutex.lock().unwrap_or_else(|e| e.into_inner())
}

impl AppState {
    pub fn config(&self) -> Config {
        lock(&self.config).clone()
    }

    pub fn update<T>(&self, f: impl FnOnce(&mut Config) -> T) -> T {
        f(&mut lock(&self.config))
    }

    pub fn persist(&self) {
        if let Err(err) = config::save(&self.config()) {
            eprintln!("[config] could not save: {err}");
        }
    }

    pub fn unread(&self) -> u32 {
        self.unread.load(Ordering::Relaxed)
    }

    pub fn views(&self) -> MutexGuard<'_, ViewState> {
        lock(&self.views)
    }

    pub fn update_views(&self, f: impl FnOnce(&mut ViewState)) {
        f(&mut lock(&self.views))
    }
}

pub fn show_main(app: &AppHandle) {
    if let Some(window) = app.get_window(MAIN) {
        let _ = window.unminimize();
        let _ = window.show();
        let _ = window.set_focus();
    }
}

/// Opens (or focuses) one of the bundled dialog windows.
pub fn open_window(app: &AppHandle, name: &str) {
    if let Some(window) = app.get_webview_window(name) {
        let _ = window.unminimize();
        let _ = window.show();
        let _ = window.set_focus();
        return;
    }
    let (title, page, size, resizable) = match name {
        "settings" => ("Settings", "settings.html", (900.0, 640.0), true),
        "join" => ("Join a meeting", "join.html", (480.0, 360.0), false),
        _ => return,
    };
    let built = WebviewWindowBuilder::new(app, name, WebviewUrl::App(page.into()))
        .title(title)
        .inner_size(size.0, size.1)
        .min_inner_size(size.0.min(720.0), size.1.min(480.0))
        .resizable(resizable)
        .center()
        // Revealed once loaded and styled, avoiding a white flash.
        .visible(false)
        .on_page_load(|window, payload| {
            if payload.event() == PageLoadEvent::Finished {
                let _ = window.show();
                let _ = window.set_focus();
            }
        })
        .build();
    if let Err(err) = built {
        eprintln!("[window] could not open {name}: {err}");
    }
}

/// `teams-for-linux-rs <meeting link>` or an `msteams:` URL handed over by the desktop.
fn handle_args(app: &AppHandle, args: &[String]) {
    let Some(url) = args.iter().skip(1).find_map(|a| links::meeting_url(a)) else { return };
    if let Some(teams) = views::content(app, View::Teams) {
        let _ = teams.navigate(url);
    }
    let _ = views::switch(app, View::Teams);
    show_main(app);
}

fn main() {
    let config = config::load();
    // WebKitGTK's DMA-BUF renderer fails on many NVIDIA, VM and restricted
    // /dev/dri setups ("DRM_IOCTL_MODE_CREATE_DUMB failed"), leaving a blank
    // window. Fall back to the shared-memory path unless the user forces the GPU
    // on or set the variable themselves. Must run before GTK starts threads.
    #[cfg(target_os = "linux")]
    if config.gpu != config::GpuPolicy::Always && std::env::var_os("WEBKIT_DISABLE_DMABUF_RENDERER").is_none() {
        std::env::set_var("WEBKIT_DISABLE_DMABUF_RENDERER", "1");
    }

    let state = AppState {
        config: Mutex::new(config.clone()),
        launched: config,
        unread: AtomicU32::new(0),
        views: Mutex::new(ViewState::default()),
    };

    tauri::Builder::default()
        .plugin(tauri_plugin_single_instance::init(|app, argv, _cwd| {
            show_main(app);
            handle_args(app, &argv);
        }))
        .plugin(tauri_plugin_notification::init())
        .plugin(tauri_plugin_opener::init())
        .manage(state)
        .invoke_handler(tauri::generate_handler![
            commands::notify,
            commands::open_external,
            commands::set_zoom,
            commands::open_window,
            commands::switch_view,
            commands::rail_state,
            commands::get_config,
            commands::save_config,
            commands::join_meeting,
            commands::resource_usage,
            commands::app_info,
            commands::restart,
        ])
        .setup(|app| {
            let handle = app.handle();
            let config = handle.state::<AppState>().config();
            views::build(handle, &config)?;
            tray::create(handle)?;
            handle_args(handle, &std::env::args().collect::<Vec<_>>());
            Ok(())
        })
        .on_window_event(|window, event| {
            if window.label() != MAIN {
                return;
            }
            match event {
                WindowEvent::Resized(_) | WindowEvent::ScaleFactorChanged { .. } => views::layout(window),
                WindowEvent::CloseRequested { api, .. }
                    if window.app_handle().state::<AppState>().config().close_to_tray =>
                {
                    api.prevent_close();
                    let _ = window.hide();
                }
                WindowEvent::Destroyed => window.app_handle().exit(0),
                _ => {}
            }
        })
        .run(tauri::generate_context!())
        .expect("error while running Teams for Linux");
}
