(() => {
  const { invoke, listen, bootstrap, toast, applyTheme } = globalThis.tfl;
  const fields = [...document.querySelectorAll("[data-key]")];
  const tabs = [...document.querySelectorAll(".nav [role=tab]")];
  const banner = document.querySelector(".banner");
  let config = {};

  // ---------- Navigation ----------

  const showPanel = (id) => {
    for (const tab of tabs) {
      const active = tab.dataset.panel === id;
      tab.setAttribute("aria-selected", String(active));
      tab.tabIndex = active ? 0 : -1;
    }
    for (const panel of document.querySelectorAll(".panel")) {
      panel.hidden = panel.id !== id;
    }
    document.querySelector(".content").scrollTop = 0;
    usagePolling(id === "performance");
  };

  for (const tab of tabs) {
    tab.addEventListener("click", () => showPanel(tab.dataset.panel));
    tab.addEventListener("keydown", (event) => {
      const step = { ArrowDown: 1, ArrowUp: -1 }[event.key];
      if (!step) return;
      const next = tabs[(tabs.indexOf(tab) + step + tabs.length) % tabs.length];
      next.focus();
      showPanel(next.dataset.panel);
      event.preventDefault();
    });
  }

  // ---------- Form binding ----------

  const zoomOutput = document.querySelector('[data-output="zoom"]');
  const showZoom = (value) => {
    zoomOutput.textContent = `${Math.round(value * 100)}%`;
  };

  const fill = (cfg) => {
    for (const el of fields) {
      const value = cfg[el.dataset.key];
      if (el.type === "checkbox") el.checked = Boolean(value);
      else if (el.type === "radio") el.checked = el.value === value;
      else if (document.activeElement !== el) el.value = value ?? "";
    }
    showZoom(cfg.zoom);
  };

  const read = (el) => {
    if (el.type === "checkbox") return el.checked;
    if (el.type === "range") return Number(el.value);
    return el.value;
  };

  let saveTimer;
  const save = () =>
    invoke("save_config", { config })
      .then(({ restartRequired }) => {
        banner.hidden = !restartRequired;
        toast("Saved");
      })
      .catch((err) => toast(String(err), { error: true }));

  const onChange = (el, { immediate }) => {
    if (el.type === "radio" && !el.checked) return;
    config = { ...config, [el.dataset.key]: read(el) };
    if (el.dataset.key === "theme") applyTheme(config.theme);
    if (el.dataset.key === "zoom") showZoom(config.zoom);
    clearTimeout(saveTimer);
    saveTimer = setTimeout(save, immediate ? 0 : 450);
  };

  for (const el of fields) {
    const isText = el.tagName === "TEXTAREA" || ["text", "url"].includes(el.type);
    el.addEventListener(isText ? "input" : "change", () => onChange(el, { immediate: !isText }));
    if (el.type === "range") el.addEventListener("input", () => showZoom(Number(el.value)));
  }

  document.querySelector('[data-action="restart"]').addEventListener("click", () => invoke("restart"));
  for (const button of document.querySelectorAll("[data-link]")) {
    button.addEventListener("click", () => invoke("open_external", { url: button.dataset.link }));
  }
  document.addEventListener("keydown", (event) => {
    if (event.key === "Escape") globalThis.tfl.closeWindow();
  });

  // ---------- Memory meter ----------

  const formatKib = (kib) => (kib >= 1024 * 1024 ? `${(kib / 1048576).toFixed(2)} GB` : `${Math.round(kib / 1024)} MB`);
  const setText = (selector, text) => {
    for (const el of document.querySelectorAll(selector)) el.textContent = text;
  };

  const refreshUsage = async () => {
    const usage = await invoke("resource_usage");
    const total = usage.appKib + usage.webKib || 1;
    setText('[data-usage="total"]', formatKib(total));
    setText('[data-usage="app"]', formatKib(usage.appKib));
    setText('[data-usage="web"]', formatKib(usage.webKib));
    setText('[data-usage="processes"]', `${usage.processes + 1} processes`);
    document.querySelector(".bar-app").style.width = `${(usage.appKib / total) * 100}%`;
    document.querySelector(".bar-web").style.width = `${(usage.webKib / total) * 100}%`;
  };

  let usageTimer;
  function usagePolling(on) {
    clearInterval(usageTimer);
    if (!on) return;
    refreshUsage();
    usageTimer = setInterval(refreshUsage, 2000);
  }

  // ---------- Start ----------

  bootstrap().then((cfg) => {
    config = cfg;
    fill(cfg);
  });
  // Zoom changed from inside Teams, or another settings change.
  listen("config-changed", (next) => {
    config = { ...config, ...next };
    fill(config);
  });
  invoke("app_info").then((info) => {
    setText('[data-info="version"]', `Version ${info.version}`);
    document.querySelector('.value[data-info="version"]').textContent = info.version;
    setText('.value[data-info="tauri"]', info.tauri);
    setText('.value[data-info="webkit"]', info.webkit);
  });
})();
