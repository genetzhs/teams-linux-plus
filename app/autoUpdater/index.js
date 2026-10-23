'use strict';

const { dialog, app } = require('electron');

// electron-updater takes ~40ms to load and only does anything inside an
// AppImage, so it is loaded on first use instead of at startup.
let autoUpdater = null;
function updater() {
	autoUpdater ??= require('electron-updater').autoUpdater;
	return autoUpdater;
}

let mainWindow = null;
let isManualCheck = false;
let isChecking = false;

// What the menu shows next to the version: 'unsupported' (not an AppImage),
// 'checking', 'latest', 'downloading', 'ready' (installs on quit or restart)
// or 'error'. `version` is the newer version while downloading or ready.
let status = process.env.APPIMAGE ? 'checking' : 'unsupported';
let newVersion = null;
const listeners = new Set();

function setStatus(next, version = newVersion) {
	status = next;
	newVersion = version;
	for (const listener of listeners) {
		try {
			listener(getStatus());
		} catch (err) {
			console.error('[AutoUpdater] Status listener failed:', err.message);
		}
	}
}

function getStatus() {
	return { status, currentVersion: app.getVersion(), newVersion };
}

/** Calls `listener(getStatus())` whenever the update status changes. */
function onStatusChange(listener) {
	listeners.add(listener);
	return () => listeners.delete(listener);
}

function initialize(window) {
	if (!process.env.APPIMAGE) {
		console.info('[AutoUpdater] Not running as AppImage, auto-updater disabled');
		return;
	}

	mainWindow = window;

	// Updates download in the background and install when the app quits, so
	// the next launch runs the new version; the user can also restart at once.
	updater().autoDownload = true;
	updater().autoInstallOnAppQuit = true;

	updater().on('update-available', onUpdateAvailable);
	updater().on('update-not-available', onUpdateNotAvailable);
	updater().on('update-downloaded', onUpdateDownloaded);
	updater().on('error', onError);

	console.info('[AutoUpdater] Checking for updates...');
	isChecking = true;
	setStatus('checking');
	updater().checkForUpdates().catch(err => {
		console.error('[AutoUpdater] Startup check failed:', err.message);
	});
}

function checkForUpdates() {
	if (!process.env.APPIMAGE) return;
	if (status === 'ready') {
		promptRestart();
		return;
	}
	if (isChecking || status === 'downloading') return;

	isChecking = true;
	isManualCheck = true;
	setStatus('checking');
	updater().checkForUpdates().catch(err => {
		console.error('[AutoUpdater] Manual check failed:', err.message);
	});
}

/** Quit and start the downloaded version. */
function restartAndInstall() {
	if (status !== 'ready') return;
	updater().quitAndInstall();
}

function onUpdateAvailable(info) {
	isChecking = false;
	console.info(`[AutoUpdater] Downloading version ${info.version} in the background`);
	setStatus('downloading', info.version);
	if (isManualCheck) {
		dialog.showMessageBox(mainWindow, {
			type: 'info',
			title: 'Update Available',
			message: `Version ${info.version} is downloading.\nYou will be asked to restart when it is ready.`,
		});
	}
	isManualCheck = false;
}

function onUpdateDownloaded(info) {
	console.info(`[AutoUpdater] Version ${info.version} downloaded; it installs on restart or quit`);
	setStatus('ready', info.version);
	promptRestart();
}

async function promptRestart() {
	if (!mainWindow || mainWindow.isDestroyed()) return;
	const { response } = await dialog.showMessageBox(mainWindow, {
		type: 'info',
		title: 'Update Ready',
		message: `Version ${newVersion} is ready to install.\nYou are running version ${app.getVersion()}.`,
		detail: 'Choose Later and it installs the next time you quit Teams for Linux.',
		buttons: ['Restart & Update', 'Later'],
		defaultId: 0,
		cancelId: 1,
	});
	if (response === 0) restartAndInstall();
}

function onUpdateNotAvailable() {
	isChecking = false;
	setStatus('latest', null);
	if (isManualCheck) {
		dialog.showMessageBox(mainWindow, {
			type: 'info',
			title: 'No Updates Available',
			message: `You're up to date! Current version: ${app.getVersion()}`,
		});
	}
	isManualCheck = false;
}

function onError(err) {
	isChecking = false;
	console.error('[AutoUpdater] Error:', err.message);
	// A failed download leaves the new version unknown; a failed check
	// while a download is ready keeps the ready state.
	if (status !== 'ready') setStatus('error', null);
	if (isManualCheck) {
		showErrorDialog();
	}
	isManualCheck = false;
}

function showErrorDialog() {
	if (mainWindow && !mainWindow.isDestroyed()) {
		dialog.showMessageBox(mainWindow, {
			type: 'error',
			title: 'Update Error',
			message: 'Failed to check for updates. Please try again later.',
		});
	}
}

module.exports = { initialize, checkForUpdates, restartAndInstall, getStatus, onStatusChange };
