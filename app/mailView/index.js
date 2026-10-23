const { WebContentsView, ipcMain, shell } = require("electron");
const path = require("node:path");
const { createMailWindowOpenHandler } = require("./windowOpenPolicy");

// Same footprint as the profile switcher pill (profileViewManager.js): a small
// square in Teams' left-rail column, so nothing in Teams is covered.
const SWITCHER_SIZE = 56;

function isMailToggleChord(input) {
  return (
    input.type === "keyDown" &&
    input.control &&
    input.alt &&
    !input.shift &&
    !input.meta &&
    input.key.toLowerCase() === "m"
  );
}

/**
 * MailViewManager — Outlook on the web inside the main window.
 *
 * The Mail view is a `WebContentsView` overlay on the root window, on the same
 * session partition as Teams so the Microsoft sign-in carries over. It is only
 * created the first time Mail is opened, so users who never open it pay no
 * memory for it. Once created it stays attached and is shown or hidden: a view
 * attached after loading paints but gets no input on Linux (#3057).
 *
 * Outlook needs no access to our code, so the view has no preload and runs
 * with contextIsolation and sandbox on, unlike the Teams views.
 */
class MailViewManager {
  #window;
  #config;
  #focusTeams;
  #switcherOffset;
  #view = null;
  #switcher = null;
  #visible = false;
  #teamsUnread = 0;
  #resizeHandler = null;
  #toggleHandler = null;

  /**
   * @param {Electron.BrowserWindow} window  Main app window
   * @param {object} config  Loaded app config (`mail`, `partition`, `chromeUserAgent`)
   * @param {object} [options]
   * @param {() => void} [options.focusTeams]  Focus the visible Teams surface
   *   (the active profile view with multi-account on) when Mail is hidden.
   * @param {number} [options.switcherOffset]  Extra distance from the bottom
   *   edge for the switcher, to stack it above the profile switcher pill.
   */
  constructor(window, config, { focusTeams, switcherOffset = 0 } = {}) {
    this.#window = window;
    this.#config = config;
    this.#focusTeams = focusTeams || (() => window.webContents.focus());
    this.#switcherOffset = switcherOffset;
  }

  get enabled() {
    return this.#config.mail?.enabled === true;
  }

  get isVisible() {
    return this.#visible;
  }

  initialize() {
    if (!this.enabled) return;

    this.#resizeHandler = () => this.#applyBounds();
    this.#window.on("resize", this.#resizeHandler);

    // Only our own switcher may toggle; Teams and Outlook renderers cannot.
    this.#toggleHandler = (event) => {
      if (this.#switcher && event.sender === this.#switcher.webContents) {
        this.toggle();
      }
    };
    // Switcher button click: toggle between Teams and Mail.
    ipcMain.on("mail-switcher-toggle", this.#toggleHandler);

    if (this.#config.mail?.showSwitcher !== false) {
      this.#createSwitcher();
    }
    this.#window.once("closed", () => this.dispose());
  }

  dispose() {
    if (this.#resizeHandler) {
      this.#window.removeListener("resize", this.#resizeHandler);
      this.#resizeHandler = null;
    }
    if (this.#toggleHandler) {
      ipcMain.removeListener("mail-switcher-toggle", this.#toggleHandler);
      this.#toggleHandler = null;
    }
    for (const view of [this.#view, this.#switcher]) {
      if (view && !view.webContents.isDestroyed()) view.webContents.close();
    }
    this.#view = null;
    this.#switcher = null;
  }

  show() {
    if (!this.enabled || this.#window.isDestroyed()) return;
    if (!this.#view) this.#createView();
    this.#applyBounds();
    this.#view.setVisible(true);
    // addChildView on an attached view only moves it to the top.
    this.#window.contentView.addChildView(this.#view);
    this.raiseSwitcher();
    this.#view.webContents.focus();
    this.#setVisible(true);
  }

  hide() {
    if (!this.#visible) return;
    this.#view?.setVisible(false);
    this.#setVisible(false);
    this.#focusTeams();
  }

  toggle() {
    if (this.#visible) this.hide();
    else this.show();
  }

  /** Teams unread count, shown on the switcher while Mail is on screen. */
  setTeamsUnread(count) {
    this.#teamsUnread = Number.isInteger(count) && count > 0 ? count : 0;
    this.#pushState();
  }

  /** Keep the switcher above views added later (profile views). */
  raiseSwitcher() {
    if (!this.#switcher) return;
    try {
      this.#window.contentView.addChildView(this.#switcher);
    } catch {
      // Window mid-teardown.
    }
  }

  #setVisible(visible) {
    this.#visible = visible;
    this.#pushState();
  }

  // Menu accelerators do not reach a focused WebContentsView, so Ctrl+Alt+M
  // is caught on our own views directly. preventDefault also stops the menu,
  // so the chord can never toggle twice.
  #handleToggleChord(webContents) {
    webContents.on("before-input-event", (event, input) => {
      if (isMailToggleChord(input)) {
        event.preventDefault();
        this.toggle();
      }
    });
  }

  #createView() {
    const view = new WebContentsView({
      webPreferences: {
        partition: this.#config.partition,
        contextIsolation: true,
        sandbox: true,
        nodeIntegration: false,
        spellcheck: true,
        focusOnNavigation: false,
      },
    });
    view.webContents.setWindowOpenHandler(
      createMailWindowOpenHandler({
        openExternal: (url) =>
          shell.openExternal(url).catch(() => {
            console.error("[MAIL] Could not open link externally");
          }),
      })
    );
    this.#handleToggleChord(view.webContents);
    // focusOnNavigation is off so a background Outlook navigation never
    // steals the keyboard from Teams; the focus() in show() runs before the
    // first page exists, so give focus once a load finishes on screen.
    view.webContents.on("did-finish-load", () => {
      if (this.#visible) view.webContents.focus();
    });
    view.webContents.once("destroyed", () => {
      if (this.#view !== view) return;
      try {
        this.#window.contentView.removeChildView(view);
      } catch {
        // Window already gone.
      }
      this.#view = null;
      if (this.#visible) this.#setVisible(false);
    });
    view.setBackgroundColor("#ffffff");
    this.#view = view;
    this.#window.contentView.addChildView(view);
    console.info("[MAIL] Loading Outlook");
    view.webContents.loadURL(this.#config.mail.url, {
      userAgent: this.#config.chromeUserAgent,
    });
  }

  #createSwitcher() {
    const view = new WebContentsView({
      webPreferences: {
        preload: path.join(__dirname, "preload.js"),
        contextIsolation: true,
        sandbox: true,
        nodeIntegration: false,
      },
    });
    view.setBackgroundColor("#00000000");
    this.#switcher = view;
    this.#handleToggleChord(view.webContents);
    this.#applySwitcherBounds();
    this.#window.contentView.addChildView(view);
    view.webContents.loadFile(path.join(__dirname, "switcher.html"));
    view.webContents.once("did-finish-load", () => this.#pushState());
  }

  #applyBounds() {
    const [width, height] = this.#window.getContentSize();
    this.#view?.setBounds({ x: 0, y: 0, width, height });
    this.#applySwitcherBounds();
  }

  #applySwitcherBounds() {
    if (!this.#switcher) return;
    const [width, height] = this.#window.getContentSize();
    const size = Math.min(SWITCHER_SIZE, width, height);
    this.#switcher.setBounds({
      x: 0,
      y: Math.max(0, height - size - this.#switcherOffset),
      width: size,
      height: size,
    });
  }

  #pushState() {
    if (!this.#switcher || this.#switcher.webContents.isDestroyed()) return;
    this.#switcher.webContents.send("mail-switcher-state", {
      mailVisible: this.#visible,
      teamsUnread: this.#teamsUnread,
    });
  }
}

module.exports = MailViewManager;
module.exports.SWITCHER_SIZE = SWITCHER_SIZE;
module.exports.isMailToggleChord = isMailToggleChord;
