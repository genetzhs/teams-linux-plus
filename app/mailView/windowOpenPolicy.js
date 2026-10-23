/**
 * Window-open policy for the Outlook Mail view.
 *
 * Outlook opens its own pop-outs (compose, reading pane, attachment previews)
 * and sign-in popups with window.open on Microsoft hosts; those stay in the
 * app. Every other link a mail contains goes to the system browser, so a
 * phishing link in an email never runs inside the signed-in session.
 *
 * Pure (no Electron imports): `openExternal` is injected so the branches are
 * unit-testable.
 */

const IN_APP_HOST_SUFFIXES = [
  "outlook.office.com",
  "outlook.office365.com",
  "outlook.cloud.microsoft",
  "outlook.live.com",
  "login.microsoftonline.com",
  "login.microsoft.com",
  "login.live.com",
];

const EXTERNAL_PROTOCOLS = new Set(["https:", "http:", "mailto:"]);

function hostMatches(hostname, suffix) {
  return hostname === suffix || hostname.endsWith(`.${suffix}`);
}

function opensInApp(url) {
  return (
    url.protocol === "about:" ||
    (url.protocol === "https:" &&
      IN_APP_HOST_SUFFIXES.some((suffix) => hostMatches(url.hostname, suffix)))
  );
}

function createMailWindowOpenHandler({ openExternal }) {
  return (details) => {
    let url;
    try {
      url = new URL(details?.url ?? "");
    } catch {
      return { action: "deny" };
    }
    if (opensInApp(url)) {
      return { action: "allow" };
    }
    if (EXTERNAL_PROTOCOLS.has(url.protocol)) {
      openExternal(url.href);
    }
    return { action: "deny" };
  };
}

module.exports = { createMailWindowOpenHandler, opensInApp };
