'use strict';

const fs = require('fs');
const path = require('path');
const { describe, it } = require('node:test');
const assert = require('node:assert/strict');
const vm = require('vm');

const VIEW_JS = fs.readFileSync(path.join(__dirname, '..', 'view.js'), 'utf8');

function loadView() {
    const sandbox = { window: null, console, document: {}, FormData: class {} };
    sandbox.window = sandbox;
    vm.createContext(sandbox);
    vm.runInContext(VIEW_JS, sandbox);
    return sandbox;
}

describe('mining queue view module', () => {
    it('registers RoboMinerMiningQueueInstall.view installer', () => {
        const sandbox = loadView();
        assert.equal(typeof sandbox.RoboMinerMiningQueueInstall.view, 'function');
        const ctx = {
            pageRoot: null,
            timerIntervals: [],
            init() {},
            refreshQueue() {},
            collectQueueQueryParams() {
                return {};
            },
            writeStoredAreaSelections() {},
            restoreAreaSelectionsFromStorage() {},
            updateClearButtonLabel() {},
            buildFragmentUrl() {
                return 'miningQueue?fragment=queue';
            },
        };
        const view = sandbox.RoboMinerMiningQueueInstall.view(ctx);
        assert.equal(typeof view.applyFragment, 'function');
        assert.equal(typeof view.applyHudFragment, 'function');
        assert.equal(typeof view.formDataToUrlEncoded, 'function');
        assert.equal(typeof view.initView, 'function');
    });

    it('formDataToUrlEncoded encodes named fields', () => {
        const sandbox = loadView();
        const ctx = {
            pageRoot: null,
            timerIntervals: [],
            init() {},
            refreshQueue() {},
            collectQueueQueryParams() {
                return {};
            },
            writeStoredAreaSelections() {},
            restoreAreaSelectionsFromStorage() {},
            updateClearButtonLabel() {},
            buildFragmentUrl() {
                return 'miningQueue?fragment=queue';
            },
        };
        const view = sandbox.RoboMinerMiningQueueInstall.view(ctx);
        class FakeFormData {
            constructor() {
                this.entries = [['submitType', 'add'], ['miningArea1', '42']];
            }
            forEach(callback) {
                this.entries.forEach(([key, value]) => callback(value, key));
            }
        }
        sandbox.FormData = FakeFormData;
        sandbox.URLSearchParams = class {
            constructor() {
                this.parts = [];
            }
            append(key, value) {
                this.parts.push(encodeURIComponent(key) + '=' + encodeURIComponent(value));
            }
            toString() {
                return this.parts.join('&');
            }
        };
        const encoded = view.formDataToUrlEncoded(new FakeFormData());
        assert.match(encoded, /submitType=add/);
        assert.match(encoded, /miningArea1=42/);
    });
});
