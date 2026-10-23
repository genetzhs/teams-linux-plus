(() => {
  const toggle = document.getElementById("toggle");
  const badge = document.getElementById("badge");

  toggle.addEventListener("click", () => globalThis.mailSwitcherApi.toggle());

  globalThis.mailSwitcherApi.onState(({ mailVisible, teamsUnread }) => {
    document.body.classList.toggle("mail", mailVisible);
    const label = mailVisible ? "Back to Teams (Ctrl+Alt+M)" : "Open Mail (Ctrl+Alt+M)";
    toggle.title = label;
    toggle.setAttribute("aria-label", label);
    // The Teams count only matters while Teams is out of sight.
    const count = mailVisible ? teamsUnread : 0;
    badge.hidden = count === 0;
    badge.textContent = count > 99 ? "99+" : String(count);
  });
})();
