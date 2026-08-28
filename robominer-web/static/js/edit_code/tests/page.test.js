'use strict';

const fs = require('fs');
const path = require('path');
const { describe, it } = require('node:test');
const assert = require('node:assert/strict');
const vm = require('vm');

const URL_QUERY_JS = fs.readFileSync(
    path.join(__dirname, '..', '..', 'common', 'url_query.js'),
    'utf8'
);
const URL_SYNC_JS = fs.readFileSync(path.join(__dirname, '..', 'url_sync.js'), 'utf8');

describe('edit code url sync', () => {
    it('syncs program selection into the editCode query string', () => {
        const sandbox = {
            window: null,
            console,
            location: { pathname: '/editCode', search: '' },
            history: {
                replaceState(_state, _title, url) {
                    sandbox.location.search = url.includes('?') ? '?' + url.split('?')[1] : '';
                },
            },
        };
        sandbox.window = sandbox;
        vm.createContext(sandbox);
        vm.runInContext(URL_QUERY_JS, sandbox);
        vm.runInContext(URL_SYNC_JS, sandbox);
        sandbox.syncEditCodeUrl('42');
        assert.match(sandbox.location.search, /nextProgramSourceId=42/);
    });
});
