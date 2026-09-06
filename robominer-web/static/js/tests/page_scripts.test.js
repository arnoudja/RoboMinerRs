'use strict';

const fs = require('fs');
const path = require('path');
const test = require('node:test');
const assert = require('node:assert/strict');
const vm = require('vm');

const JS_ROOT = path.join(__dirname, '..');
const PAGE_SCRIPTS = [
    'common/panel_state.js',
    'common/url_query.js',
    'common/session_store.js',
    'common/app_dialog.js',
    'common/local_time.js',
    'common/area_filter.js',
    'common/password_toggle.js',
    'common/signup_pow.js',
    'common/shell_nav.js',
    'edit_code/editor.js',
    'edit_code/url_sync.js',
    'edit_code/save.js',
    'edit_code/page.js',
    'robot/page.js',
    'shop/page.js',
    'mining_queue/clear_wallet.js',
    'mining_queue/view.js',
    'mining_queue/claim_poll.js',
    'mining_queue/actions.js',
    'mining_queue/page.js',
    'mining_results/page.js',
    'mining_area_atlas/page.js',
];

for (const relative of PAGE_SCRIPTS) {
    test(`page script parses: ${relative}`, () => {
        const source = fs.readFileSync(path.join(JS_ROOT, relative), 'utf8');
        assert.ok(source.trim().length > 0, `${relative} should not be empty`);
        assert.doesNotThrow(() => new vm.Script(source, { filename: relative }));
    });
}
