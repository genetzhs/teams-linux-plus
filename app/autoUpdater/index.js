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

function initialize(window) {
	if (!process.env.APPIMAGE) {
		console.info('[AutoUpdater] Not running as AppImage, auto-updater disabled');
		return;
	}

	mainWindow = window;

	// Updates download in the background and install when the app quits, so
	// the next launch runs the new version without asking.
	updater().autoDownload = true;
	updater().autoInstallOnAppQuit = true;

	updater().on('update-available', onUpdateAvailable);
	updater().on('update-not-available', onUpdateNotAvailable);
	updater().on('update-downloaded', onUpdateDownloaded);
	updater().on('error', onError);

	console.info('[AutoUpdater] Checking for updates...');
	isChecking = true;
	updater().checkForUpdates().catch(err => {
		console.error('[AutoUpdater] Startup check failed:', err.message);
	});
}

function checkForUpdates() {
	if (!process.env.APPIMAGE) return;
	if (isChecking) return;

	isChecking = true;
	isManualCheck = true;
	updater().checkForUpdates().catch(err => {
		console.error('[AutoUpdater] Manual check failed:', err.message);
	});
}

function onUpdateAvailable(info) {
	isChecking = false;
	console.info(`[AutoUpdater] Downloading version ${info.version} in the background`);
	if (isManualCheck) {
		dialog.showMessageBox(mainWindow, {
			type: 'info',
			title: 'Update Available',
			message: `Version ${info.version} is downloading.\nIt will be installed when you quit Teams for Linux.`,
		});
	}
	isManualCheck = false;
}

function onUpdateDownloaded(info) {
	console.info(`[AutoUpdater] Version ${info.version} downloaded; it installs when the app quits`);
}

function onUpdateNotAvailable() {
	isChecking = false;
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

module.exports = { initialize, checkForUpdates };
