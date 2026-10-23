(() => {
  const { invoke, bootstrap, closeWindow } = globalThis.tfl;
  const form = document.querySelector("form");
  const input = document.querySelector("#link");
  const hint = document.querySelector("#hint");
  const submit = form.querySelector('[type="submit"]');
  const defaultHint = hint.textContent;

  // Quick client-side check to enable the button; the backend validates for real.
  const looksLikeLink = (value) => /(https:\/\/teams\.(microsoft\.com|live\.com|cloud\.microsoft)\/|msteams:)/i.test(value);

  const setError = (message) => {
    hint.textContent = message || defaultHint;
    hint.classList.toggle("error", Boolean(message));
    input.setAttribute("aria-invalid", String(Boolean(message)));
  };

  input.addEventListener("input", () => {
    submit.disabled = !looksLikeLink(input.value);
    setError("");
  });

  form.addEventListener("submit", async (event) => {
    event.preventDefault();
    submit.disabled = true;
    submit.textContent = "Joining…";
    try {
      await invoke("join_meeting", { link: input.value });
    } catch (err) {
      setError(String(err));
      submit.textContent = "Join";
      submit.disabled = false;
      input.focus();
    }
  });

  form.querySelector('[data-action="cancel"]').addEventListener("click", closeWindow);
  document.addEventListener("keydown", (event) => {
    if (event.key === "Escape") closeWindow();
  });

  bootstrap().then(() => input.focus());
})();
