# Auto Updater Module

In-app auto-update for AppImage distributions using electron-updater.

## Overview

Only activates when the app is running as an AppImage (detected via `process.env.APPIMAGE`). On other packaging formats (snap, deb, rpm) or in dev mode, the module does nothing.

On startup, checks for updates silently and downloads a newer version in the background. When the download finishes, a dialog offers "Restart & Update" or "Later"; with "Later" the update installs the next time the app quits.

The app menu and tray menu show the running version with its update status: `(latest)`, `(checking for updates…)`, `(downloading X…)`, `(X ready)` or `(update check failed)`. Below it is "Check for Updates", or "Restart & Update to X" once a download is ready. Outside an AppImage only the version is shown.

## API

**`initialize(mainWindow)`** — Call once after the main window is ready. Configures electron-updater, registers event handlers, and triggers a silent startup check.

**`checkForUpdates()`** — Triggers a manual update check. Shows a dialog whether an update is found or not; with an update already downloaded it offers the restart instead.

**`restartAndInstall()`** — Quits and installs a downloaded update. Does nothing until one is ready.

**`getStatus()`** — Returns `{ status, currentVersion, newVersion }`, where `status` is `unsupported`, `checking`, `latest`, `downloading`, `ready` or `error`.

**`onStatusChange(listener)`** — Calls `listener(getStatus())` on every status change; returns an unsubscribe function. The menus use it to rebuild the version line.

## Dependencies

- `electron-updater` — Runtime dependency for update detection, download, and installation
- `electron` — `dialog` for user prompts, `app` for version info

## Usage

```javascript
const AutoUpdater = require('./autoUpdater');
AutoUpdater.initialize(mainWindow);
```

Menu integration (in `app/menus/index.js`):

```javascript
autoUpdaterModule.onStatusChange(() => this.#refreshUpdateItems());
```
