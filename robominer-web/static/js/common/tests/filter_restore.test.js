'use strict';

const fs = require('fs');
const path = require('path');
const { describe, it } = require('node:test');
const assert = require('node:assert/strict');
const vm = require('vm');

const FILTER_RESTORE_JS = fs.readFileSync(
    path.join(__dirname, '..', 'filter_restore.js'),
    'utf8'
);
const SESSION_STORE_JS = fs.readFileSync(
    path.join(__dirname, '..', 'session_store.js'),
    'utf8'
);

describe('filter restore helper', () => {
    it('restores select values from session storage when URL lacks params', () => {
        const storage = {};
        const select = {
            name: 'areaId',
            value: '',
        };
        const sandbox = {
            window: null,
            console,
            sessionStorage: {
                getItem(key) {
                    return Object.prototype.hasOwnProperty.call(storage, key) ? storage[key] : null;
                },
                setItem(key, value) {
                    storage[key] = String(value);
                },
            },
            document: {
                getElementsByName(name) {
                    return name === 'areaId' ? [select] : [];
                },
            },
            RoboMinerUrlQuery: {
                get() {
                    return null;
                },
            },
        };
        sandbox.window = sandbox;
        vm.createContext(sandbox);
        vm.runInContext(SESSION_STORE_JS, sandbox);
        vm.runInContext(FILTER_RESTORE_JS, sandbox);
        sandbox.RoboMinerSessionStore.writeJson('robominer.test.filters', { areaId: '1001' });
        sandbox.RoboMinerFilterRestore.restoreSelectFilters({
            storageKey: 'robominer.test.filters',
            selectNames: ['areaId'],
        });
        assert.equal(select.value, '1001');
    });
});
