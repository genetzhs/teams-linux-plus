# mailView

Outlook on the web inside the main window, switchable with Teams.

## Behaviour

- **Lazy.** Nothing is created at startup. The Outlook `WebContentsView` is built the first time Mail is opened, so users who never open it pay no memory for it.
- **Overlay, not a column.** The view covers the whole content area on top of Teams (the same model as the multi-account profile views), and is shown or hidden. It stays attached once created: on Linux a view that loads while detached and is attached later paints but gets no input (#3057).
- **Same sign-in.** It uses the root Teams partition (`config.partition`), so the Microsoft sign-in carries over.
- **Hardened.** Outlook needs none of our code, so the view has no preload and runs with `contextIsolation` and `sandbox` on. Its notifications use Electron's native HTML5 notification path.
- **Links.** `windowOpenPolicy.js` keeps Outlook pop-outs and Microsoft sign-in popups in the app and sends every other link to the system browser.
- **Back to Teams automatically** when a call connects, when Join Meeting or Return to Teams is used, when another instance hands over a deep link, and when a profile is picked.

## Switching

| Where | How |
| --- | --- |
| Switcher button | Bottom-left, in Teams' left-rail column; stacked above the profile pill when multi-account is on. Shows the Teams unread count while Mail is on screen |
| Keyboard | `Ctrl+Alt+M` (Teams' own `Ctrl+Shift+M` is mute) |
| Menu and tray | **Mail** in the Teams for Linux menu |

Menu accelerators do not reach a focused `WebContentsView`, so the chord is also caught with `before-input-event` on the Mail view and on the switcher view.

## Configuration

```json
{
  "mail": {
    "enabled": true,
    "url": "https://outlook.office.com/mail/",
    "showSwitcher": true
  }
}
```

Use `https://outlook.live.com/mail/` for a personal Outlook.com account.

## IPC

| Channel | Direction | Purpose |
| --- | --- | --- |
| `mail-switcher-toggle` | renderer → main | Switcher click; only accepted from the switcher's own webContents |
| `mail-switcher-state` | main → renderer | `{ mailVisible, teamsUnread }` for the switcher button |
