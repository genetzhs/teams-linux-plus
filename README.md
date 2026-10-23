# Teams Linux Plus

<img src="build/icons/256x256.png" alt="" width="96" align="right">

**Unofficial Microsoft Teams client for Linux, with Outlook Mail in the same window.**

Teams Linux Plus is a fork of [Teams for Linux](https://github.com/IsmaelMartinez/teams-for-linux) by Ismael Martinez and its contributors. It keeps everything that project does (notifications, tray, screen sharing, profiles, calls) and adds:

- **Mail in the same window.** Outlook on the web next to Teams: switch with the button in the bottom-left corner, `Ctrl+Alt+M`, or **Mail** in the menu and tray. It only loads the first time you open it and uses the same sign-in as Teams. Links in emails open in your browser.
- **Faster on Linux.** The GPU stays on under XWayland (the default on Wayland sessions), GPU rasterization and VA-API hardware video are on by default, and UI animations are skipped for snappier navigation.
- **Redesigned app windows.** One design system with light and dark themes for the join-meeting dialog, call and notification toasts, profile dialogs, PIN and sign-in prompts, Quick Chat and the screen-share picker.
- **A new icon** and monochrome tray glyphs that stay readable at 16px.

> [!NOTE]
> This is an independent project, not affiliated with Microsoft or with the upstream Teams for Linux project. Some features are limited by the Teams web app.

## Run from source

You need Node.js 24 (see `.nvmrc`) and git.

```bash
git clone https://github.com/genetzhs/teams-linux-plus.git
cd teams-linux-plus
npm ci
npm start
```

If the optional native module `cbor-extract` fails to compile, `npm ci` warns and carries on; the app does not need it.

## Build packages

```bash
npm run dist:linux      # AppImage, deb, rpm, snap in dist/
sudo apt install ./dist/teams-for-linux_*_amd64.deb
```

The packaged app keeps the `teams-for-linux` name and settings folder (`~/.config/teams-for-linux`), so it replaces an existing Teams for Linux install and keeps your sign-in and settings.

## Configuration

Settings live in `~/.config/teams-for-linux/config.json`. The options added here:

```json
{
  "mail": {
    "enabled": true,
    "url": "https://outlook.office.com/mail/",
    "showSwitcher": true
  },
  "media": {
    "linuxPerformanceMode": true,
    "reduceMotion": true
  }
}
```

- `mail.url`: use `https://outlook.live.com/mail/` for a personal Outlook.com account.
- `media.linuxPerformanceMode`: set to `false` if the window flickers or shows black areas.
- `media.reduceMotion`: set to `false` to get Teams' animations back.
- If the window stays blank, set `"performance": { "disableGpu": true }`.

Every other option is documented in [docs-site/docs/configuration.md](docs-site/docs/configuration.md).

## Rust preview

`rust/` holds an experimental Rust (Tauri) client on the system WebKitGTK. It is much lighter, but calls do not work on distributions whose WebKitGTK is built without WebRTC (Ubuntu 24.04 included). See [rust/README.md](rust/README.md).

## License and credits

GPL-3.0, like the project it is based on: see [LICENSE.md](LICENSE.md). Teams for Linux is © Ismael Martinez and contributors; its original README is kept in [docs/UPSTREAM-README.md](docs/UPSTREAM-README.md).
