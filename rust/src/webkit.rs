//! WebKitGTK tuning for the Teams view: media permissions for calls and the
//! memory/GPU knobs exposed in Settings → Performance.

use crate::config::{Config, GpuPolicy};
use tauri::Webview;

#[cfg(target_os = "linux")]
pub fn tune(webview: &Webview, config: &Config) {
    use webkit2gtk::{
        glib::ObjectExt, CacheModel, HardwareAccelerationPolicy, NotificationPermissionRequest, PermissionRequestExt,
        SettingsExt, UserMediaPermissionRequest, WebContextExt, WebViewExt,
    };

    let low_memory = config.low_memory;
    let gpu = config.gpu;
    let result = webview.with_webview(move |platform| {
        let view = platform.inner();

        if let Some(settings) = WebViewExt::settings(&view) {
            settings.set_enable_media_stream(true);
            settings.set_enable_webrtc(true);
            settings.set_enable_mediasource(true);
            settings.set_enable_encrypted_media(true);
            settings.set_media_playback_requires_user_gesture(false);
            settings.set_javascript_can_access_clipboard(true);
            // Back/forward page cache keeps whole Teams documents alive; useless in a single-page app.
            settings.set_enable_page_cache(!low_memory);
            settings.set_hardware_acceleration_policy(match gpu {
                GpuPolicy::Auto => HardwareAccelerationPolicy::OnDemand,
                GpuPolicy::Always => HardwareAccelerationPolicy::Always,
                GpuPolicy::Never => HardwareAccelerationPolicy::Never,
            });
        }

        if low_memory {
            if let Some(context) = view.context() {
                // Smallest in-memory resource cache; Teams' own service worker cache still applies.
                context.set_cache_model(CacheModel::DocumentViewer);
            }
        }

        // Camera, microphone and notification prompts are granted: the user
        // already chose to run Teams, and WebKitGTK has no prompt UI of its own.
        view.connect_permission_request(|_, request| {
            if request.is::<UserMediaPermissionRequest>() || request.is::<NotificationPermissionRequest>() {
                request.allow();
                true
            } else {
                false
            }
        });
    });
    if let Err(err) = result {
        eprintln!("[webkit] could not tune webview: {err}");
    }
}

#[cfg(not(target_os = "linux"))]
pub fn tune(_webview: &Webview, _config: &Config) {}
