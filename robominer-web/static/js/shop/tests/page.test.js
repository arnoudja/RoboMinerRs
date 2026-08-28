'use strict';

const fs = require('fs');
const path = require('path');
const { describe, it } = require('node:test');
const assert = require('node:assert/strict');
const vm = require('vm');

const JS_ROOT = path.join(__dirname, '..', '..');
const SESSION_STORE_JS = fs.readFileSync(path.join(JS_ROOT, 'common', 'session_store.js'), 'utf8');
const URL_QUERY_JS = fs.readFileSync(path.join(JS_ROOT, 'common', 'url_query.js'), 'utf8');

describe('shop page filters', () => {
    it('persists selected filters to session storage', () => {
        const storageKey = 'robominer.shop.test';
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
        sandbox.RoboMinerSessionStore.writeJson(storageKey, {
            selectedRobotPartTypeId: '2',
            selectedTierId: '3',
            selectedRobotPartId: '901',
        });
        const stored = sandbox.RoboMinerSessionStore.readJson(storageKey);
        assert.equal(
            JSON.stringify(stored),
            JSON.stringify({
                selectedRobotPartTypeId: '2',
                selectedTierId: '3',
                selectedRobotPartId: '901',
            })
        );
    });

    it('detects when URL already carries shop filter params', () => {
        const sandbox = {
            window: null,
            console,
            location: { search: '?selectedRobotPartTypeId=2' },
        };
        sandbox.window = sandbox;
        vm.createContext(sandbox);
        vm.runInContext(URL_QUERY_JS, sandbox);
        assert.equal(
            sandbox.RoboMinerUrlQuery.hasAnyParam([
                'selectedRobotPartTypeId',
                'selectedTierId',
                'selectedRobotPartId',
            ]),
            true
        );
    });
});
