'use strict';

const { describe, it } = require('node:test');
const assert = require('node:assert');
const { createMailWindowOpenHandler } = require('../../app/mailView/windowOpenPolicy');

function handlerWithLog() {
  const opened = [];
  const handler = createMailWindowOpenHandler({ openExternal: (url) => opened.push(url) });
  return { handler, opened };
}

describe('Mail window-open policy', () => {
  it('keeps Outlook pop-outs and sign-in popups in the app', () => {
    const { handler, opened } = handlerWithLog();
    for (const url of [
      'https://outlook.office.com/mail/deeplink/compose',
      'https://outlook.live.com/mail/0/',
      'https://login.microsoftonline.com/common/oauth2/authorize',
      'about:blank',
    ]) {
      assert.deepStrictEqual(handler({ url }), { action: 'allow' }, url);
    }
    assert.deepStrictEqual(opened, []);
  });

  it('sends other links in mails to the browser', () => {
    const { handler, opened } = handlerWithLog();
    assert.deepStrictEqual(handler({ url: 'https://example.com/invoice' }), { action: 'deny' });
    assert.deepStrictEqual(handler({ url: 'mailto:someone@example.com' }), { action: 'deny' });
    assert.deepStrictEqual(opened, ['https://example.com/invoice', 'mailto:someone@example.com']);
  });

  it('does not trust look-alike hosts', () => {
    const { handler, opened } = handlerWithLog();
    assert.deepStrictEqual(handler({ url: 'https://outlook.office.com.evil.example/' }), { action: 'deny' });
    assert.deepStrictEqual(handler({ url: 'http://outlook.office.com/' }), { action: 'deny' });
    assert.strictEqual(opened.length, 2);
  });

  it('never hands dangerous or broken URLs to the OS', () => {
    const { handler, opened } = handlerWithLog();
    assert.deepStrictEqual(handler({ url: 'file:///etc/passwd' }), { action: 'deny' });
    assert.deepStrictEqual(handler({ url: 'javascript:alert(1)' }), { action: 'deny' });
    assert.deepStrictEqual(handler({ url: 'not a url' }), { action: 'deny' });
    assert.deepStrictEqual(handler({}), { action: 'deny' });
    assert.deepStrictEqual(opened, []);
  });
});
