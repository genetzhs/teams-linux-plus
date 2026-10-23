(() => {
  const { invoke, listen, bootstrap } = globalThis.tfl;
  const tabs = [...document.querySelectorAll(".app")];

  const render = (state) => {
    for (const tab of tabs) {
      const view = tab.dataset.view;
      tab.setAttribute("aria-selected", String(view === state.active));
      tab.tabIndex = view === state.active ? 0 : -1;
    }
    document.querySelector('[data-view="mail"]').hidden = !state.mailEnabled;

    const teams = document.querySelector('[data-badge="teams"]');
    teams.hidden = state.teamsUnread === 0;
    teams.textContent = state.teamsUnread > 99 ? "99+" : String(state.teamsUnread);

    document.querySelector('[data-badge="mail"]').hidden = !state.mailAttention;
  };

  const select = (view) => invoke("switch_view", { name: view }).catch(() => {});

  for (const tab of tabs) {
    tab.addEventListener("click", () => select(tab.dataset.view));
    tab.addEventListener("keydown", (event) => {
      if (event.key !== "ArrowDown" && event.key !== "ArrowUp") return;
      const visible = tabs.filter((t) => !t.hidden);
      const next = visible[(visible.indexOf(tab) + (event.key === "ArrowDown" ? 1 : -1) + visible.length) % visible.length];
      next.focus();
      select(next.dataset.view);
      event.preventDefault();
    });
  }

  for (const button of document.querySelectorAll("[data-open]")) {
    button.addEventListener("click", () => invoke("open_window", { name: button.dataset.open }));
  }

  bootstrap();
  invoke("rail_state").then(render);
  listen("rail-state", render);
})();
