// Injected into the Teams and Outlook pages before their own scripts run. Keeps
// the surface small: forward web notifications, route keyboard shortcuts, apply
// custom CSS. The unread count is read natively from the document title.
(() => {
  if (globalThis.top !== globalThis.self || globalThis.__TFL_BRIDGE__) return;
  globalThis.__TFL_BRIDGE__ = true;

  const config = globalThis.__TFL__ || {};

  const invoke = (cmd, args) => {
    const ipc = globalThis.__TAURI_INTERNALS__;
    if (!ipc) return Promise.resolve();
    return ipc.invoke(cmd, args).catch(() => {});
  };

  // Teams calls `new Notification(...)`; WebKitGTK would otherwise drop these.
  class BridgedNotification extends EventTarget {
    static get permission() {
      return "granted";
    }

    static requestPermission(callback) {
      if (typeof callback === "function") callback("granted");
      return Promise.resolve("granted");
    }

    constructor(title, options = {}) {
      super();
      this.title = String(title ?? "");
      this.body = String(options.body ?? "");
      this.tag = options.tag ?? "";
      this.data = options.data;
      this.onclick = null;
      this.onclose = null;
      this.onerror = null;
      this.onshow = null;
      invoke("notify", { title: this.title, body: this.body });
      queueMicrotask(() => {
        const event = new Event("show");
        this.dispatchEvent(event);
        if (typeof this.onshow === "function") this.onshow(event);
      });
    }

    close() {}
  }
  Object.defineProperty(globalThis, "Notification", {
    value: BridgedNotification,
    configurable: true,
    writable: true,
  });

  const zoomKeys = { "=": 1, "+": 1, "-": -1, _: -1, 0: 0 };
  globalThis.addEventListener(
    "keydown",
    (event) => {
      if (!event.ctrlKey || event.metaKey) return;
      let handled = true;
      if (event.altKey) {
        // Ctrl+1..6 belong to Teams' own navigation, so views use Ctrl+Alt.
        const view = { Digit1: "teams", Digit2: "mail" }[event.code];
        if (!view || event.shiftKey) return;
        invoke("switch_view", { name: view });
      } else if (!event.shiftKey && event.key in zoomKeys) {
        invoke("set_zoom", { step: zoomKeys[event.key] });
      } else if (!event.shiftKey && event.key === ",") {
        invoke("open_window", { name: "settings" });
      } else if (event.shiftKey && event.key.toLowerCase() === "j") {
        invoke("open_window", { name: "join" });
      } else {
        handled = false;
      }
      if (handled) {
        event.preventDefault();
        event.stopPropagation();
      }
    },
    true,
  );

  if (config.customCss) {
    const apply = () => {
      const style = document.createElement("style");
      style.id = "tfl-custom-css";
      style.textContent = config.customCss;
      document.head.append(style);
    };
    if (document.head) apply();
    else document.addEventListener("DOMContentLoaded", apply, { once: true });
  }
})();
