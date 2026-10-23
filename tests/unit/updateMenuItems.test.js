'use strict';

const { describe, it, before, after } = require('node:test');
const assert = require('node:assert');

// The app menu shows the running version, whether it is the latest, and a
// restart action once a downloaded update is ready.
const electronPath = require.resolve('electron');
const appMenuPath = require.resolve('../../app/menus/appMenu');

let appMenu;

before(() => {
	require.cache[electronPath] = {
		id: electronPath,
		filename: electronPath,
		loaded: true,
		exports: { shell: {} },
	};
	delete require.cache[appMenuPath];
	appMenu = require('../../app/menus/appMenu');
});

after(() => {
	delete require.cache[electronPath];
	delete require.cache[appMenuPath];
});

function updateItems(status, newVersion = null) {
	const calls = [];
	const menus = new Proxy(
		{
			configGroup: { startupConfig: {} },
			getUpdateStatus: () => ({ status, currentVersion: '2.28.0', newVersion }),
		},
		{
			get(target, prop) {
				if (prop in target) return target[prop];
				return () => calls.push(prop);
			},
		},
	);
	const items = appMenu(menus).submenu;
	const start = items.findIndex((item) => item.label?.startsWith('Version '));
	const end = items.findIndex((item, i) => i > start && item.label === 'Hide');
	return { items: items.slice(start, end), calls };
}

describe('App menu update items', () => {
	it('shows only the version outside an AppImage', () => {
		const { items } = updateItems('unsupported');
		assert.deepStrictEqual(items.map((i) => i.label), ['Version 2.28.0']);
		assert.strictEqual(items[0].enabled, false);
	});

	it('marks the latest version and offers a manual check', () => {
		const { items } = updateItems('latest');
		assert.deepStrictEqual(items.map((i) => i.label), ['Version 2.28.0 (latest)', 'Check for Updates']);
		assert.notStrictEqual(items[1].enabled, false);
	});

	it('disables the check while a download is running', () => {
		const { items } = updateItems('downloading', '2.29.0');
		assert.strictEqual(items[0].label, 'Version 2.28.0 (downloading 2.29.0…)');
		assert.strictEqual(items[1].enabled, false);
	});

	it('offers restart once the update is ready', () => {
		const { items, calls } = updateItems('ready', '2.29.0');
		assert.deepStrictEqual(items.map((i) => i.label), ['Version 2.28.0 (2.29.0 ready)', 'Restart & Update to 2.29.0']);
		items[1].click();
		assert.deepStrictEqual(calls, ['restartToUpdate']);
	});
});
