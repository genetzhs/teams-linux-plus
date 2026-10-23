const { contextBridge, ipcRenderer } = require("electron");

// Bridge for the Teams/Mail switcher button. Our own HTML only, so it runs
// with contextIsolation + sandbox and reaches main through two channels.
contextBridge.exposeInMainWorld("mailSwitcherApi", {
  toggle: () => ipcRenderer.send("mail-switcher-toggle"),
  // Main pushes { mailVisible, teamsUnread }.
  onState: (callback) =>
    ipcRenderer.on("mail-switcher-state", (_event, state) => callback(state)),
});
