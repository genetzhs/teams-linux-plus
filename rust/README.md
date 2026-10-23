# Teams for Linux — Rust edition (preview)

A rewrite of the client as a small native shell in Rust ([Tauri 2](https://tauri.app)) on the system **WebKitGTK**, instead of a bundled Chromium. Teams and Outlook Mail live side by side in one window, and the app's own screens (navigation rail, settings, join meeting) were redesigned from scratch.

This is a preview that lives next to the Electron app; it does not replace it yet. See [ADR 033](../docs-site/docs/development/adr/033-rust-tauri-client.md) for the reasoning and the known gaps.

## What you get

- **One window, two apps.** A slim navigation rail switches between Teams and Outlook Mail (`Ctrl+Alt+1` / `Ctrl+Alt+2`). Mail only loads the first time you open it, so it costs no memory until you use it, and turning it off in Settings frees its web process.
- **Native unread badge.** The Teams unread count is read from the page title in Rust and shown on the rail and on the tray icon. Mail shows a dot when a mail notification arrives while you are in Teams.
- **Desktop notifications** for Teams and Outlook, with an option to hide message previews.
- **Join a meeting** from a link (`Ctrl+Shift+J`, tray menu, rail button, or `teams-for-linux-rs <link>` / `msteams:` links from the desktop).
- **Redesigned settings** with auto-save, a light and a dark theme, a restart banner for options that need it, and a live **memory meter** (app shell and web content, read from `/proc`).
- **Low memory mode** (on by default): smallest WebKit cache model and no back/forward page cache. GPU acceleration can be set to Auto, On or Off.
- Single instance, close to tray, start minimized, zoom (`Ctrl +` / `Ctrl −` / `Ctrl 0`), custom CSS and user agent.

## Security model

The remote Teams and Outlook pages get only a minimal IPC surface (`capabilities/teams.json`, `capabilities/mail.json`): show a notification, open a link in the browser (http/https/mailto/tel only), change zoom, switch view and open the app's own dialogs. Reading or writing configuration is only possible from the bundled pages (`capabilities/local.json`). Links that are not part of Microsoft sign-in, Teams or Outlook open in the default browser.

## Build and run

System packages (Debian/Ubuntu names):

```bash
sudo apt install libwebkit2gtk-4.1-dev libgtk-3-dev libayatana-appindicator3-dev librsvg2-dev libsoup-3.0-dev
```

Then, from this directory:

```bash
cargo run                   # debug build
cargo test                  # unit tests
cargo build --release       # optimised binary in target/release/teams-for-linux-rs
cargo install tauri-cli --version "^2" && cargo tauri build   # .deb, .rpm and AppImage
```

Settings are stored in `~/.config/teams-for-linux-rs/config.json`.

## Layout

| Path | Purpose |
| --- | --- |
| `src/main.rs` | App setup, shared state, dialog windows, single instance |
| `src/views.rs` | Main window: rail plus lazily created Teams/Mail webviews |
| `src/commands.rs` | IPC commands |
| `src/config.rs` | Config file with defaults and validation |
| `src/links.rs` | Which URLs stay in the app, meeting-link parsing |
| `src/tray.rs` | Tray menu and the unread badge drawn into the icon |
| `src/webkit.rs` | WebKitGTK media permissions, cache and GPU settings |
| `src/usage.rs` | Memory sampling from `/proc` |
| `inject/bridge.js` | Script injected into Teams/Outlook (notifications, shortcuts, custom CSS) |
| `ui/` | Rail, settings and join pages: plain HTML/CSS/JS, no build step |

## Not there yet

Compared to the Electron app, these are not implemented: screen-sharing picker, incoming-call toast, MQTT, Graph API features, multiple accounts/profiles, custom backgrounds, global shortcuts, spell-check language selection, client certificates and Intune SSO. Clicking a notification does not focus the conversation yet. Call quality and screen sharing depend on the distribution's WebKitGTK build (WebRTC needs WebKitGTK 2.40 or later with GStreamer WebRTC plugins).
