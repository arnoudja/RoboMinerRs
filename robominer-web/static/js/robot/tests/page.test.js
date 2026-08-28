'use strict';

const fs = require('fs');
const path = require('path');
const { describe, it } = require('node:test');
const assert = require('node:assert/strict');
const vm = require('vm');

const PANEL_STATE_JS = fs.readFileSync(
    path.join(__dirname, '..', '..', 'common', 'panel_state.js'),
    'utf8'
);
const URL_QUERY_JS = fs.readFileSync(
    path.join(__dirname, '..', '..', 'common', 'url_query.js'),
    'utf8'
);

function createPanel(initialValue) {
    const field = { name: 'programSourceId', value: initialValue, disabled: false };
    let baseline = null;
    return {
        getAttribute(name) {
            return name === 'data-form-baseline' ? baseline : null;
        },
        setAttribute(name, value) {
            if (name === 'data-form-baseline') {
                baseline = value;
            }
        },
        querySelectorAll(selector) {
            if (selector === 'input[name], select[name], textarea[name]') {
                return [field];
            }
            if (selector === 'input, select, textarea, button') {
                return [field];
            }
            return [];
        },
        querySelector(selector) {
            if (selector.startsWith('select[name^="programSourceId"]')) {
                return field;
            }
            return null;
        },
        changeValue(next) {
            field.value = next;
        },
    };
}

describe('robot page panel state', () => {
    it('marks panel dirty when a tracked field changes', () => {
        const sandbox = { window: null, console };
        sandbox.window = sandbox;
        vm.createContext(sandbox);
        vm.runInContext(PANEL_STATE_JS, sandbox);
        const panel = createPanel('1');
        sandbox.RoboMinerPanelState.capturePanelBaseline(panel, ['robotId']);
        panel.changeValue('2');
        assert.equal(
            sandbox.RoboMinerPanelState.isPanelDirty(panel, ['robotId']),
            true
        );
    });

    it('syncs robot id into the URL query string', () => {
        const sandbox = {
            window: null,
            console,
            location: { pathname: '/robot', search: '' },
            history: {
                replaceState(_state, _title, url) {
                    sandbox.location.pathname = url.split('?')[0];
                    sandbox.location.search = url.includes('?') ? '?' + url.split('?')[1] : '';
                },
            },
        };
        sandbox.window = sandbox;
        vm.createContext(sandbox);
        vm.runInContext(URL_QUERY_JS, sandbox);
        sandbox.RoboMinerUrlQuery.sync('robot', { robotId: '7' });
        assert.match(sandbox.location.search, /robotId=7/);
    });
});
