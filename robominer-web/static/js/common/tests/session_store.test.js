'use strict';

const fs = require('fs');
const path = require('path');
const { describe, it } = require('node:test');
const assert = require('node:assert/strict');
const vm = require('vm');

const SESSION_STORE_JS = fs.readFileSync(
    path.join(__dirname, '..', 'session_store.js'),
    'utf8'
);

describe('session store helper', () => {
    it('round-trips JSON through sessionStorage', () => {
        const storage = {};
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
        };
        sandbox.window = sandbox;
        vm.createContext(sandbox);
        vm.runInContext(SESSION_STORE_JS, sandbox);

        sandbox.RoboMinerSessionStore.writeJson('robominer.test', { areaId: '9' });
        // Parsed JSON from the vm realm is cross-realm; compare via JSON.
        assert.equal(
            JSON.stringify(sandbox.RoboMinerSessionStore.readJson('robominer.test')),
            JSON.stringify({ areaId: '9' })
        );
        assert.equal(sandbox.RoboMinerSessionStore.readJson('missing', { ok: true }).ok, true);
    });
});
