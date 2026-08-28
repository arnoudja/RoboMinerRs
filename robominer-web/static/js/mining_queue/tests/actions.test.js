'use strict';

const fs = require('fs');
const path = require('path');
const { describe, it } = require('node:test');
const assert = require('node:assert/strict');
const vm = require('vm');

const ACTIONS_JS = fs.readFileSync(path.join(__dirname, '..', 'actions.js'), 'utf8');

function loadActions() {
    const sandbox = {
        window: null,
        console,
        document: {
            getElementById() {
                return null;
            },
        },
        FormData: class {},
    };
    sandbox.window = sandbox;
    vm.createContext(sandbox);
    vm.runInContext(ACTIONS_JS, sandbox);
    return sandbox;
}

describe('mining queue actions module', () => {
    it('registers clear/remove helpers and wires updateClearButtonLabel on ctx', () => {
        const sandbox = loadActions();
        const ctx = {
            buildFragmentUrl() {
                return 'miningQueue?fragment=queue';
            },
            updateClearButtonLabel: null,
        };
        const view = {
            fetchFragment() {
                return Promise.resolve();
            },
        };
        const actions = sandbox.RoboMinerMiningQueueInstall.actions(ctx, view);
        assert.equal(typeof actions.clearQueuedRuns, 'function');
        assert.equal(typeof actions.removeQueuedRun, 'function');
        assert.equal(typeof actions.submitFormPartial, 'function');
        assert.equal(typeof ctx.updateClearButtonLabel, 'function');
    });
});
