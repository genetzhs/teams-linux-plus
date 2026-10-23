// Shared helpers for the bundled pages. Exposed as `globalThis.tfl` so each page
// script stays a plain classic script with no bundler.
(() => {
  const tauri = globalThis.__TAURI__;

  const invoke = (cmd, args) => tauri.core.invoke(cmd, args);
  const listen = (event, handler) => tauri.event.listen(event, (e) => handler(e.payload));

  const applyTheme = (theme) => {
    if (theme === "light" || theme === "dark") {
      document.documentElement.dataset.theme = theme;
    } else {
      delete document.documentElement.dataset.theme;
    }
  };

  // Loads config, applies the theme and keeps it in sync.
  const bootstrap = async () => {
    const config = await invoke("get_config");
    applyTheme(config.theme);
    listen("config-changed", (next) => applyTheme(next.theme));
    return config;
  };

  let toastTimer;
  const toast = (message, { error = false } = {}) => {
    let el = document.querySelector(".toast");
    if (!el) {
      el = document.createElement("div");
      el.className = "toast";
      el.setAttribute("role", "status");
      document.body.append(el);
    }
    el.textContent = message;
    el.classList.toggle("error", error);
    el.classList.add("show");
    clearTimeout(toastTimer);
    toastTimer = setTimeout(() => el.classList.remove("show"), error ? 4000 : 1600);
  };

  const closeWindow = () => tauri.window.getCurrentWindow().close();

  globalThis.tfl = { invoke, listen, applyTheme, bootstrap, toast, closeWindow };
})();
