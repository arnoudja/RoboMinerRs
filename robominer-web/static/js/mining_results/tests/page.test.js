'use strict';

const fs = require('fs');
const path = require('path');
const { describe, it } = require('node:test');
const assert = require('node:assert/strict');
const vm = require('vm');

const JS_ROOT = path.join(__dirname, '..', '..');
const URL_QUERY_JS = fs.readFileSync(path.join(JS_ROOT, 'common', 'url_query.js'), 'utf8');
const PAGE_JS = fs.readFileSync(path.join(JS_ROOT, 'mining_results', 'page.js'), 'utf8');

class FakeClassList {
    constructor(element, className) {
        this.element = element;
        this.tokens = new Set();
        if (className) {
            className.split(/\s+/).filter(Boolean).forEach((token) => this.tokens.add(token));
        }
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
            this.add(name);
            return true;
        }
        if (force === false) {
            this.remove(name);
            return false;
        }
        if (this.contains(name)) {
            this.remove(name);
            return false;
        }
        this.add(name);
        return true;
    }
}

function createElement(tagName, attrs) {
    const element = {
        tagName: String(tagName).toUpperCase(),
        children: [],
        parent: null,
        attrs: Object.assign({}, attrs || {}),
        listeners: {},
        hidden: false,
        value: attrs && attrs.value !== undefined ? String(attrs.value) : '',
        options: [],
    };
    element.classList = new FakeClassList(element, element.attrs.class || '');
    element.id = element.attrs.id || '';
    element.getAttribute = function(name) {
        if (name === 'class') {
            return Array.from(element.classList.tokens).join(' ');
        }
        return Object.prototype.hasOwnProperty.call(element.attrs, name)
            ? String(element.attrs[name])
            : null;
    };
    element.setAttribute = function(name, value) {
        element.attrs[name] = String(value);
        if (name === 'id') {
            element.id = String(value);
        }
        if (name === 'class') {
            element.classList = new FakeClassList(element, String(value));
        }
    };
    element.appendChild = function(child) {
        if (child.parent) {
            child.parent.children = child.parent.children.filter((node) => node !== child);
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
    element.click = function() {
        const handlers = element.listeners.click || [];
        for (let index = 0; index < handlers.length; index += 1) {
            handlers[index]({ currentTarget: element });
        }
    };
    element.dispatchEvent = function(type) {
        const handlers = element.listeners[type] || [];
        for (let index = 0; index < handlers.length; index += 1) {
            handlers[index]({ currentTarget: element });
        }
    };
    element.querySelector = function(selector) {
        return queryAll(element, selector)[0] || null;
    };
    element.querySelectorAll = function(selector) {
        return queryAll(element, selector);
    };
    return element;
}

function descendants(root) {
    const nodes = [];
    const visit = function(node) {
        for (let index = 0; index < node.children.length; index += 1) {
            const child = node.children[index];
            nodes.push(child);
            visit(child);
        }
    };
    visit(root);
    return nodes;
}

function matchesSelector(element, selector) {
    let remaining = selector;
    const notClasses = [];
    remaining = remaining.replace(/:not\(\.([^)]+)\)/g, function(_, className) {
        notClasses.push(className);
        return '';
    });
    const classes = [];
    remaining = remaining.replace(/\.([A-Za-z0-9_-]+)/g, function(_, className) {
        classes.push(className);
        return '';
    });
    const attrs = [];
    remaining = remaining.replace(/\[([A-Za-z0-9_-]+)(?:=(?:"([^"]*)"|'([^']*)'|([^\]]+)))?\]/g, function(_, name, doubleQuoted, singleQuoted, bare) {
        const value = doubleQuoted !== undefined
            ? doubleQuoted
            : singleQuoted !== undefined
                ? singleQuoted
                : bare;
        attrs.push({ name: name, value: value });
        return '';
    });
    let id = null;
    remaining = remaining.replace(/#([A-Za-z0-9_-]+)/g, function(_, value) {
        id = value;
        return '';
    });
    remaining = remaining.trim();
    if (remaining && element.tagName !== remaining.toUpperCase()) {
        return false;
    }
    if (id && element.id !== id) {
        return false;
    }
    for (let classIndex = 0; classIndex < classes.length; classIndex += 1) {
        if (!element.classList.contains(classes[classIndex])) {
            return false;
        }
    }
    for (let attrIndex = 0; attrIndex < attrs.length; attrIndex += 1) {
        const actual = element.getAttribute(attrs[attrIndex].name);
        if (attrs[attrIndex].value === undefined) {
            if (actual === null) {
                return false;
            }
        } else if (actual !== attrs[attrIndex].value) {
            return false;
        }
    }
    for (let notIndex = 0; notIndex < notClasses.length; notIndex += 1) {
        if (element.classList.contains(notClasses[notIndex])) {
            return false;
        }
    }
    return true;
}

function queryAll(root, selector) {
    return descendants(root).filter((element) => matchesSelector(element, selector));
}

function createPageDocument() {
    const root = createElement('div');
    const robotFilter = createElement('select', { id: 'miningResultsRobotFilter' });
    robotFilter.options = [
        { value: '' },
        { value: '1' },
        { value: '2' },
    ];
    robotFilter.value = '';
    const areaFilter = createElement('select', { id: 'miningResultsAreaFilter' });
    areaFilter.options = [
        { value: '' },
        { value: 'Alpha' },
        { value: 'Beta' },
    ];
    areaFilter.value = '';
    const sortFilter = createElement('select', { id: 'miningResultsSortFilter' });
    sortFilter.options = [
        { value: 'newest' },
        { value: 'reward' },
        { value: 'score' },
    ];
    sortFilter.value = 'newest';
    const cards = createElement('div', {
        class: 'mining-results-run-cards',
        'data-initial-visible': '5',
        'data-load-more-step': '5',
    });
    const panels = createElement('div', { class: 'mining-results-detail-panels' });
    const loadMoreWrap = createElement('p', { id: 'miningResultsLoadMoreWrap', class: 'mining-results-load-more-wrap' });
    loadMoreWrap.hidden = true;
    const loadMore = createElement('button', { id: 'miningResultsLoadMore', class: 'mining-results-load-more' });
    const empty = createElement('p', { id: 'miningResultsFilterEmpty', class: 'mining-results-filter-empty' });
    empty.hidden = true;

    const runs = [
        { id: 10, robotId: 1, area: 'Alpha', end: 8000, reward: 8 },
        { id: 11, robotId: 2, area: 'Beta', end: 7000, reward: 7 },
        { id: 12, robotId: 1, area: 'Alpha', end: 6000, reward: 6 },
        { id: 13, robotId: 2, area: 'Beta', end: 5000, reward: 5 },
        { id: 14, robotId: 1, area: 'Alpha', end: 4000, reward: 4 },
        { id: 15, robotId: 2, area: 'Beta', end: 3000, reward: 3 },
        { id: 16, robotId: 1, area: 'Alpha', end: 2000, reward: 2 },
        { id: 17, robotId: 2, area: 'Beta', end: 1000, reward: 1 },
    ];
    for (let index = 0; index < runs.length; index += 1) {
        const run = runs[index];
        const card = createElement('button', {
            class: 'mining-results-run-card',
            'data-run-id': String(run.id),
            'data-robot-id': String(run.robotId),
            'data-area-name': run.area,
            'data-sort-end': String(run.end),
            'data-sort-reward': String(run.reward),
            'data-sort-score': String(run.reward),
        });
        const panel = createElement('div', {
            class: 'mining-results-detail-panel',
            id: 'miningResultDetails' + run.id,
            'data-run-id': String(run.id),
            'data-robot-id': String(run.robotId),
            'data-area-name': run.area,
            'data-sort-end': String(run.end),
            'data-sort-reward': String(run.reward),
            'data-sort-score': String(run.reward),
        });
        panel.hidden = true;
        cards.appendChild(card);
        panels.appendChild(panel);
    }

    root.appendChild(robotFilter);
    root.appendChild(areaFilter);
    root.appendChild(sortFilter);
    root.appendChild(cards);
    root.appendChild(panels);
    loadMoreWrap.appendChild(loadMore);
    root.appendChild(loadMoreWrap);
    root.appendChild(empty);

    const document = {
        documentElement: root,
        body: root,
        getElementById: function(id) {
            if (root.id === id) {
                return root;
            }
            return descendants(root).find((element) => element.id === id) || null;
        },
        querySelector: function(selector) {
            return queryAll(root, selector)[0] || null;
        },
        querySelectorAll: function(selector) {
            return queryAll(root, selector);
        },
    };
    return document;
}

function loadPage(search) {
    const document = createPageDocument();
    const context = {
        console,
        document,
        Array,
        Number,
        Object,
        String,
        Math,
        encodeURIComponent,
        decodeURIComponent,
        location: { search: search || '' },
        history: {
            replaceState: function() {},
        },
    };
    context.window = context;
    context.globalThis = context;
    vm.createContext(context);
    vm.runInContext(URL_QUERY_JS, context, { filename: 'url_query.js' });
    vm.runInContext(PAGE_JS, context, { filename: 'page.js' });
    return document;
}

function visibleRunIds(document) {
    return Array.from(document.querySelectorAll('.mining-results-run-card'))
        .filter((card) => !card.classList.contains('mining-results-filter-hidden')
            && !card.classList.contains('mining-results-run-card-collapsed'))
        .map((card) => card.getAttribute('data-run-id'));
}

describe('mining results recent runs list', () => {
    it('shows five runs by default and reveals more from the same list', () => {
        const document = loadPage('');
        assert.deepEqual(visibleRunIds(document), ['10', '11', '12', '13', '14']);
        assert.equal(document.getElementById('miningResultsLoadMoreWrap').hidden, false);

        document.getElementById('miningResultsLoadMore').click();
        assert.deepEqual(visibleRunIds(document), ['10', '11', '12', '13', '14', '15', '16', '17']);
        assert.equal(document.getElementById('miningResultsLoadMoreWrap').hidden, true);
    });

    it('expands the list far enough to show a linked run', () => {
        const document = loadPage('?runId=16');
        assert.deepEqual(visibleRunIds(document), ['10', '11', '12', '13', '14', '15', '16']);
        assert.equal(
            document.querySelector('.mining-results-run-card-active').getAttribute('data-run-id'),
            '16'
        );
        assert.equal(document.getElementById('miningResultsLoadMoreWrap').hidden, false);
    });

    it('filters the single list and resets the visible window', () => {
        const document = loadPage('');
        document.getElementById('miningResultsLoadMore').click();
        const robotFilter = document.getElementById('miningResultsRobotFilter');
        robotFilter.value = '1';
        robotFilter.dispatchEvent('change');
        assert.deepEqual(visibleRunIds(document), ['10', '12', '14', '16']);
        assert.equal(document.getElementById('miningResultsLoadMoreWrap').hidden, true);
    });
});
