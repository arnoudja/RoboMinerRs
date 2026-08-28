'use strict';

const fs = require('fs');
const path = require('path');
const { describe, it } = require('node:test');
const assert = require('node:assert/strict');
const vm = require('vm');

const PANEL_STATE_JS = fs.readFileSync(
    path.join(__dirname, '..', 'panel_state.js'),
    'utf8'
);

describe('panel state helper', () => {
    it('tracks dirty state from captured baseline', () => {
        const panel = {
            attrs: {},
            fields: [
                { name: 'title', value: 'Alpha', disabled: false },
            ],
            querySelectorAll(selector) {
                if (selector === 'input, select, textarea, button') {
                    return this.fields;
                }
                if (selector === 'input[name], select[name], textarea[name]') {
                    return this.fields.filter((field) => field.name);
                }
                return [];
            },
            getAttribute(name) {
                return this.attrs[name] || null;
            },
            setAttribute(name, value) {
                this.attrs[name] = value;
            },
        };

        const sandbox = { window: null, console };
        sandbox.window = sandbox;
        vm.createContext(sandbox);
        vm.runInContext(PANEL_STATE_JS, sandbox);

        assert.equal(sandbox.RoboMinerPanelState.isPanelDirty(panel), false);
        sandbox.RoboMinerPanelState.capturePanelBaseline(panel);
        assert.equal(sandbox.RoboMinerPanelState.isPanelDirty(panel), false);

        panel.fields[0].value = 'Beta';
        assert.equal(sandbox.RoboMinerPanelState.isPanelDirty(panel), true);

        sandbox.RoboMinerPanelState.restorePanelBaseline(panel);
        assert.equal(panel.fields[0].value, 'Alpha');
        assert.equal(sandbox.RoboMinerPanelState.isPanelDirty(panel), false);
    });

    it('disables and enables panel fields', () => {
        const panel = {
            fields: [
                { name: 'title', disabled: false },
                { name: 'save', disabled: false },
            ],
            querySelectorAll(selector) {
                if (selector === 'input, select, textarea, button') {
                    return this.fields;
                }
                return [];
            },
        };

        const sandbox = { window: null, console };
        sandbox.window = sandbox;
        vm.createContext(sandbox);
        vm.runInContext(PANEL_STATE_JS, sandbox);

        sandbox.RoboMinerPanelState.setPanelEnabled(panel, false);
        assert.equal(panel.fields.every((field) => field.disabled), true);
        sandbox.RoboMinerPanelState.setPanelEnabled(panel, true);
        assert.equal(panel.fields.every((field) => field.disabled === false), true);
    });
});
