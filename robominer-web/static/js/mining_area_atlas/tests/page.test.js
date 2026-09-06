'use strict';

const fs = require('fs');
const path = require('path');
const { describe, it } = require('node:test');
const assert = require('node:assert/strict');
const vm = require('vm');

const JS_ROOT = path.join(__dirname, '..', '..');
const URL_QUERY_JS = fs.readFileSync(path.join(JS_ROOT, 'common', 'url_query.js'), 'utf8');
const ATLAS_JS = fs.readFileSync(path.join(__dirname, '..', 'page.js'), 'utf8');

class FakeClassList {
    constructor(initial) {
        this.tokens = new Set(String(initial || '').split(/\s+/).filter(Boolean));
    }

    add(name) {
        this.tokens.add(name);
    }

    remove(name) {
        this.tokens.delete(name);
    }

    contains(name) {
        return this.tokens.has(name);
    }

    toggle(name, force) {
        if (force === true) {
            this.tokens.add(name);
            return true;
        }
        if (force === false) {
            this.tokens.delete(name);
            return false;
        }
        if (this.tokens.has(name)) {
            this.tokens.delete(name);
            return false;
        }
        this.tokens.add(name);
        return true;
    }
}

function createElement(attrs) {
    const element = {
        attrs: Object.assign({}, attrs || {}),
        children: [],
        parent: null,
        listeners: {},
        hidden: false,
        checked: false,
        value: attrs && attrs.value !== undefined ? String(attrs.value) : '',
        options: attrs && attrs.options ? attrs.options.slice() : [],
        classList: new FakeClassList(attrs && attrs.class),
    };
    element.id = element.attrs.id || '';
    element.getAttribute = function(name) {
        return Object.prototype.hasOwnProperty.call(element.attrs, name)
            ? String(element.attrs[name])
            : null;
    };
    element.setAttribute = function(name, value) {
        element.attrs[name] = String(value);
    };
    element.querySelectorAll = function(selector) {
        if (selector !== '.mining-area-atlas-row') {
            return [];
        }
        return element.children.filter((child) =>
            child.classList.contains('mining-area-atlas-row')
        );
    };
    element.appendChild = function(child) {
        if (child.parent) {
            const index = child.parent.children.indexOf(child);
            if (index >= 0) {
                child.parent.children.splice(index, 1);
            }
        }
        child.parent = element;
        element.children.push(child);
        return child;
    };
    element.addEventListener = function(type, handler) {
        if (!element.listeners[type]) {
            element.listeners[type] = [];
        }
        element.listeners[type].push(handler);
    };
    element.dispatchEvent = function(type) {
        for (const handler of element.listeners[type] || []) {
            handler();
        }
    };
    return element;
}

function createAtlasSandbox(urlSearch) {
    const historyCalls = [];
    const sortSelect = createElement({
        id: 'miningAreaAtlasSort',
        value: 'level',
        options: [
            { value: 'level' },
            { value: 'name' },
            { value: 'total' },
            { value: 'ore' },
        ],
    });
    const oreSelect = createElement({
        id: 'miningAreaAtlasOreSort',
        value: '1',
        options: [{ value: '1' }, { value: '2' }],
    });
    const oreField = createElement({ id: 'miningAreaAtlasOreField' });
    const affordableOnly = createElement({ id: 'miningAreaAtlasAffordableOnly' });
    const empty = createElement({ id: 'miningAreaAtlasFilterEmpty' });
    empty.hidden = true;
    const tbody = createElement({ id: 'miningAreaAtlasRows' });
    const rows = [
        createElement({
            class: 'mining-area-atlas-row',
            'data-area-name': 'Zeta',
            'data-area-id': '1',
            'data-total-yield': '10',
            'data-ore-yield-1': '5',
            'data-ore-yield-2': '1',
            'data-affordable': '0',
        }),
        createElement({
            class: 'mining-area-atlas-row',
            'data-area-name': 'Alpha',
            'data-area-id': '3',
            'data-total-yield': '40',
            'data-ore-yield-1': '2',
            'data-ore-yield-2': '9',
            'data-affordable': '1',
        }),
        createElement({
            class: 'mining-area-atlas-row',
            'data-area-name': 'Beta',
            'data-area-id': '2',
            'data-total-yield': '20',
            'data-ore-yield-1': '8',
            'data-ore-yield-2': '3',
            'data-affordable': '1',
        }),
    ];
    for (const row of rows) {
        tbody.appendChild(row);
    }

    const byId = {
        miningAreaAtlasSort: sortSelect,
        miningAreaAtlasOreSort: oreSelect,
        miningAreaAtlasOreField: oreField,
        miningAreaAtlasAffordableOnly: affordableOnly,
        miningAreaAtlasRows: tbody,
        miningAreaAtlasFilterEmpty: empty,
    };

    const sandbox = {
        window: null,
        console,
        location: { search: urlSearch || '' },
        history: {
            replaceState(_state, _title, url) {
                historyCalls.push(url);
            },
        },
        document: {
            getElementById(id) {
                return byId[id] || null;
            },
        },
        historyCalls,
        sortSelect,
        oreSelect,
        oreField,
        affordableOnly,
        empty,
        tbody,
    };
    sandbox.window = sandbox;
    vm.createContext(sandbox);
    vm.runInContext(URL_QUERY_JS, sandbox);
    vm.runInContext(ATLAS_JS, sandbox);
    return sandbox;
}

function rowNames(tbody) {
    return tbody.children.map((row) => row.getAttribute('data-area-name'));
}

describe('mining area atlas page', () => {
    it('sorts by area level descending by default', () => {
        const sandbox = createAtlasSandbox('');
        assert.equal(sandbox.sortSelect.value, 'level');
        assert.deepEqual(rowNames(sandbox.tbody), ['Alpha', 'Beta', 'Zeta']);
        assert.deepEqual(
            sandbox.tbody.children.map((row) => row.getAttribute('data-area-id')),
            ['3', '2', '1']
        );
        assert.equal(sandbox.oreField.hidden, true);
    });

    it('sorts by name and total yield', () => {
        const sandbox = createAtlasSandbox('');
        sandbox.sortSelect.value = 'name';
        sandbox.sortSelect.dispatchEvent('change');
        assert.deepEqual(rowNames(sandbox.tbody), ['Alpha', 'Beta', 'Zeta']);

        sandbox.sortSelect.value = 'total';
        sandbox.sortSelect.dispatchEvent('change');
        assert.deepEqual(rowNames(sandbox.tbody), ['Alpha', 'Beta', 'Zeta']);
        assert.deepEqual(
            sandbox.tbody.children.map((row) => row.getAttribute('data-total-yield')),
            ['40', '20', '10']
        );
    });

    it('sorts by selected ore yield and shows ore field', () => {
        const sandbox = createAtlasSandbox('');
        sandbox.sortSelect.value = 'ore';
        sandbox.oreSelect.value = '1';
        sandbox.sortSelect.dispatchEvent('change');
        assert.equal(sandbox.oreField.hidden, false);
        assert.deepEqual(rowNames(sandbox.tbody), ['Beta', 'Zeta', 'Alpha']);
    });

    it('filters unaffordable rows and toggles empty state', () => {
        const sandbox = createAtlasSandbox('');
        sandbox.affordableOnly.checked = true;
        sandbox.affordableOnly.dispatchEvent('change');
        const visible = sandbox.tbody.children.filter(
            (row) => !row.classList.contains('mining-area-atlas-filter-hidden')
        );
        assert.deepEqual(
            visible.map((row) => row.getAttribute('data-area-name')),
            ['Alpha', 'Beta']
        );
        assert.equal(sandbox.empty.hidden, true);

        for (const row of sandbox.tbody.children) {
            row.setAttribute('data-affordable', '0');
        }
        sandbox.affordableOnly.dispatchEvent('change');
        assert.equal(
            sandbox.tbody.children.every((row) =>
                row.classList.contains('mining-area-atlas-filter-hidden')
            ),
            true
        );
        assert.equal(sandbox.empty.hidden, false);
    });

    it('restores controls from URL and syncs query params', () => {
        const sandbox = createAtlasSandbox('?sort=ore&oreId=2&affordable=1');
        assert.equal(sandbox.sortSelect.value, 'ore');
        assert.equal(sandbox.oreSelect.value, '2');
        assert.equal(sandbox.affordableOnly.checked, true);
        assert.equal(sandbox.oreField.hidden, false);
        assert.ok(
            sandbox.historyCalls.some((url) =>
                url.includes('miningAreaOverview?') &&
                url.includes('sort=ore') &&
                url.includes('oreId=2') &&
                url.includes('affordable=1')
            )
        );
    });
});
