---
id: 033-rust-tauri-client
---

# ADR 033: Rust/Tauri Client on the System WebView (Preview)

## Status

🚧 Proposed (2026-10-23).

## Context

The Electron app bundles its own Chromium. That gives Teams the browser it was built for, but it also means a large download, a high baseline memory use before Teams itself loads, and a renderer stack that the project updates on Electron's schedule rather than the distribution's. Users regularly ask for something lighter, and some also want Outlook Mail available next to Teams without a second app.

Three options were weighed:

- **Optimise the Electron app.** Keeps every feature, but the Chromium baseline stays.
- **A native toolkit such as egui.** Gives full control over the app's own screens, but cannot render Teams or Outlook: both are web apps and need a browser engine. It would only draw the settings and dialogs, and would add its own GPU context on top of the web engine, which works against the memory goal.
- **A thin Rust shell on the system WebView ([Tauri 2](https://tauri.app)).** On Linux this is WebKitGTK, shared with the rest of the desktop and updated by the distribution. The app's own screens can be ordinary HTML/CSS rendered by the same engine at no extra cost.

## Decision

Build a preview client in `rust/` with Tauri 2 on WebKitGTK, alongside the Electron app rather than replacing it.

- **One window, multiple webviews.** A fixed-width navigation rail (local page) next to one content webview per service. Teams loads at startup; Outlook is created on first use and closed again when Mail is turned off, so it costs nothing until used. On Linux Tauri packs child webviews into the window's `GtkBox`, so the box is switched to horizontal and the rail's width is pinned; GTK then sizes the visible content view.
- **Native where it is cheap.** The Teams unread count comes from `on_document_title_changed` in Rust rather than an injected observer. The tray badge is drawn into the icon's RGBA buffer without an image library.
- **Minimal remote IPC.** Remote pages are granted only `notify`, `open_external`, `set_zoom`, `open_window` and `switch_view` through capabilities scoped to the Teams and Outlook origins. Configuration commands are only granted to the bundled pages.
- **Memory knobs.** A default-on low memory mode uses WebKit's `DocumentViewer` cache model and disables the page cache; GPU acceleration is configurable. The settings window shows live memory use from `/proc`.
- **Redesigned app screens.** Rail, settings and join-meeting pages share one token-based stylesheet with light and dark themes and no web fonts or build step.

## Consequences

- WebKitGTK is not a browser Teams officially supports, so the client presents an Edge user agent. Calls and screen sharing depend on the distribution's WebKitGTK and GStreamer WebRTC support and need validation on real accounts before the preview can be recommended.
- Most of the Electron app's features are not ported yet (screen-share picker, incoming-call toast, MQTT, Graph API, profiles, custom backgrounds, global shortcuts, certificates, Intune). They are listed in `rust/README.md` and should be ported only once the base is validated with users.
- Child webviews need Tauri's `unstable` feature. The API is used narrowly (`Window::add_child`, show/hide) so an upgrade should stay contained to `src/views.rs`.
- The preview has its own CI job (`.github/workflows/rust.yml`) for tests and clippy; packaging is not wired into releases.
