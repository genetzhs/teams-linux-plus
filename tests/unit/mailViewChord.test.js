'use strict';

const { describe, it } = require('node:test');
const assert = require('node:assert');
const Module = require('node:module');

// app/mailView/index.js requires electron at load time; the chord helper is pure.
const originalLoad = Module._load;
Module._load = function (request, ...rest) {
  if (request === 'electron') return { WebContentsView: class {}, ipcMain: {}, shell: {} };
  return originalLoad.call(this, request, ...rest);
};
const { isMailToggleChord } = require('../../app/mailView');
Module._load = originalLoad;

const key = (overrides) => ({ type: 'keyDown', control: true, alt: true, shift: false, meta: false, key: 'm', ...overrides });

describe('Mail toggle chord', () => {
  it('matches Ctrl+Alt+M on key down, either case', () => {
    assert.strictEqual(isMailToggleChord(key()), true);
    assert.strictEqual(isMailToggleChord(key({ key: 'M' })), true);
  });

  it('ignores key up, other modifiers and other keys', () => {
    assert.strictEqual(isMailToggleChord(key({ type: 'keyUp' })), false);
    assert.strictEqual(isMailToggleChord(key({ shift: true })), false); // Teams uses Ctrl+Shift+M for mute
    assert.strictEqual(isMailToggleChord(key({ alt: false })), false);
    assert.strictEqual(isMailToggleChord(key({ meta: true })), false);
    assert.strictEqual(isMailToggleChord(key({ key: 'n' })), false);
  });
});
