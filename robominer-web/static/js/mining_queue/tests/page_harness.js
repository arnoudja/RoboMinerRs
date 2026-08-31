'use strict';

const fs = require('fs');
const path = require('path');
const vm = require('vm');

const JS_ROOT = path.join(__dirname, '..', '..');
const URL_QUERY_JS = fs.readFileSync(path.join(JS_ROOT, 'common', 'url_query.js'), 'utf8');
const SESSION_STORE_JS = fs.readFileSync(path.join(JS_ROOT, 'common', 'session_store.js'), 'utf8');
const VIEW_JS = fs.readFileSync(path.join(__dirname, '..', 'view.js'), 'utf8');
const CLAIM_POLL_JS = fs.readFileSync(path.join(__dirname, '..', 'claim_poll.js'), 'utf8');
const ACTIONS_JS = fs.readFileSync(path.join(__dirname, '..', 'actions.js'), 'utf8');
const PAGE_JS = fs.readFileSync(path.join(__dirname, '..', 'page.js'), 'utf8');

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
        textContent: attrs && attrs.textContent !== undefined ? String(attrs.textContent) : '',
        _innerHTML: attrs && attrs.innerHTML !== undefined ? String(attrs.innerHTML) : '',
        options: [],
        disabled: false,
    };
    element.classList = new FakeClassList(element, element.attrs.class || '');
    element.id = element.attrs.id || '';
    element.name = element.attrs.name || '';
    Object.defineProperty(element, 'innerHTML', {
        get() {
            if (element.children.length > 0) {
                return element.children.map((child) => serializeElement(child)).join('');
            }
            return element._innerHTML;
        },
        set(value) {
            element._innerHTML = String(value);
            element.children = [];
            const parsed = parseHtmlTree(element._innerHTML);
            for (const child of parsed) {
                element.appendChild(child);
            }
        },
        configurable: true,
    });
    if (element._innerHTML) {
        const parsed = parseHtmlTree(element._innerHTML);
        for (const child of parsed) {
            element.appendChild(child);
        }
    }
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
        if (name === 'name') {
            element.name = String(value);
        }
        if (name === 'class') {
            element.classList = new FakeClassList(element, String(value));
        }
    };
    element.removeAttribute = function(name) {
        delete element.attrs[name];
        if (name === 'id') {
            element.id = '';
        }
        if (name === 'name') {
            element.name = '';
        }
        if (name === 'class') {
            element.classList = new FakeClassList(element, '');
        }
    };
    element.closest = function(selector) {
        let node = element;
        while (node) {
            if (matchesSelector(node, selector)) {
                return node;
            }
            node = node.parent;
        }
        return null;
    };
    element.appendChild = function(child) {
        if (child.parent) {
            child.parent.children = child.parent.children.filter((node) => node !== child);
        }
        child.parent = element;
        element.children.push(child);
        return child;
    };
    element.insertBefore = function(newNode, referenceNode) {
        if (newNode.parent) {
            newNode.parent.children = newNode.parent.children.filter((node) => node !== newNode);
        }
        newNode.parent = element;
        const index = referenceNode ? element.children.indexOf(referenceNode) : element.children.length;
        element.children.splice(index < 0 ? element.children.length : index, 0, newNode);
        return newNode;
    };
    element.remove = function() {
        if (!element.parent) {
            return;
        }
        element.parent.children = element.parent.children.filter((node) => node !== element);
        element.parent = null;
    };
    element.cloneNode = function() {
        const clone = createElement(element.tagName.toLowerCase(), {
            class: element.getAttribute('class') || '',
            id: element.id,
            textContent: element.textContent,
        });
        clone.innerHTML = element.innerHTML;
        return clone;
    };
    element.addEventListener = function(type, handler) {
        if (!element.listeners[type]) {
            element.listeners[type] = [];
        }
        element.listeners[type].push(handler);
    };
    element.querySelector = function(selector) {
        return queryAll(element, selector)[0] || null;
    };
    element.querySelectorAll = function(selector) {
        return queryAll(element, selector);
    };
    Object.defineProperty(element, 'outerHTML', {
        get() {
            return serializeElement(element);
        },
        set(value) {
            const parsed = parseHtmlTree(value)[0];
            if (!parsed || !element.parent) {
                return;
            }
            const index = element.parent.children.indexOf(element);
            element.parent.children[index] = parsed;
            parsed.parent = element.parent;
            element.parent = null;
        },
    });
    return element;
}

function serializeElement(element) {
    const attrs = Object.keys(element.attrs)
        .map((name) => `${name}="${element.attrs[name]}"`)
        .join(' ');
    const open = attrs.length > 0
        ? `<${element.tagName.toLowerCase()} ${attrs}>`
        : `<${element.tagName.toLowerCase()}>`;
    const childMarkup = element.children.map((child) => serializeElement(child)).join('');
    const text = element.textContent || '';
    return `${open}${childMarkup}${text}</${element.tagName.toLowerCase()}>`;
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

function matchesAttributeSelector(element, attrExpr) {
    const match = /^([A-Za-z0-9_-]+)(?:(\^=|=)"([^"]*)")?$/.exec(attrExpr);
    if (!match) {
        return false;
    }
    const name = match[1];
    const op = match[2] || null;
    const expected = match[3];
    const actual = element.getAttribute(name);
    if (op === null) {
        return actual !== null;
    }
    if (actual === null) {
        return false;
    }
    if (op === '=') {
        return actual === expected;
    }
    if (op === '^=') {
        return actual.indexOf(expected) === 0;
    }
    return false;
}

function matchesSimpleSelector(element, selector) {
    let remaining = selector;
    const classes = [];
    remaining = remaining.replace(/\.([A-Za-z0-9_-]+)/g, function(_, className) {
        classes.push(className);
        return '';
    });
    let id = null;
    remaining = remaining.replace(/#([A-Za-z0-9_-]+)/g, function(_, value) {
        id = value;
        return '';
    });
    const attrs = [];
    remaining = remaining.replace(/\[([^\]]+)\]/g, function(_, attrExpr) {
        attrs.push(attrExpr);
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
        if (!matchesAttributeSelector(element, attrs[attrIndex])) {
            return false;
        }
    }
    return true;
}

function matchesSelector(element, selector) {
    const parts = selector.trim().split(/\s+/).filter(Boolean);
    if (parts.length <= 1) {
        return matchesSimpleSelector(element, selector.trim());
    }
    // Descendant: last part must match element; ancestors must match earlier parts in order.
    if (!matchesSimpleSelector(element, parts[parts.length - 1])) {
        return false;
    }
    let ancestor = element.parent;
    for (let partIndex = parts.length - 2; partIndex >= 0; partIndex -= 1) {
        let found = false;
        while (ancestor) {
            if (matchesSimpleSelector(ancestor, parts[partIndex])) {
                found = true;
                ancestor = ancestor.parent;
                break;
            }
            ancestor = ancestor.parent;
        }
        if (!found) {
            return false;
        }
    }
    return true;
}

function queryAll(root, selector) {
    const selectors = selector.split(',').map((part) => part.trim());
    const matches = [];
    for (const candidate of descendants(root)) {
        for (const single of selectors) {
            if (matchesSelector(candidate, single)) {
                matches.push(candidate);
                break;
            }
        }
    }
    for (const single of selectors) {
        if (matchesSelector(root, single)) {
            matches.unshift(root);
            break;
        }
    }
    return matches;
}

function parseHtmlTree(html) {
    const root = createElement('div');
    const tagPattern = /<([a-zA-Z0-9-]+)([^>]*)>([\s\S]*?)<\/\1>/g;
    let match = tagPattern.exec(html);
    while (match) {
        const attrs = {};
        const attrPattern = /([a-zA-Z0-9-]+)="([^"]*)"/g;
        let attrMatch = attrPattern.exec(match[2]);
        while (attrMatch) {
            attrs[attrMatch[1]] = attrMatch[2];
            if (attrMatch[1] === 'class') {
                attrs.class = attrMatch[2];
            }
            attrMatch = attrPattern.exec(match[2]);
        }
        const element = createElement(match[1], attrs);
        element.textContent = match[3];
        root.appendChild(element);
        match = tagPattern.exec(html);
    }
    return root.children;
}

class FakeDOMParser {
    parseFromString(html) {
        const nodes = parseHtmlTree(html);
        const byId = {};
        for (const node of nodes) {
            if (node.id) {
                byId[node.id] = node;
            }
            for (const descendant of descendants(node)) {
                if (descendant.id) {
                    byId[descendant.id] = descendant;
                }
            }
        }
        return {
            getElementById(id) {
                return byId[id] || null;
            },
        };
    }
}

function createMiningQueueDocument() {
    const body = createElement('body');
    const hud = createElement('div', { class: 'app-shell-hud' });
    hud.innerHTML = '<span id="old-hud">1/3</span>';
    body.appendChild(hud);

    const page = createElement('div', {
        class: 'mining-queue-page',
        'data-area-storage-key': 'robominer.miningQueue.areaSelections.test',
    });
    const wallet = createElement('section', { class: 'page-wallet mining-queue-wallet' });
    wallet.innerHTML = '<span id="old-wallet">wallet</span>';
    page.appendChild(wallet);

    const error = createElement('p', { class: 'error mining-queue-error' });
    error.textContent = 'old error';
    page.appendChild(error);

    const deck = createElement('div', { class: 'mining-queue-deck' });
    const robots = createElement('div', { class: 'mining-queue-robots' });
    robots.innerHTML = '<form class="mining-queue-card"><span id="old-robot">old</span></form>';
    deck.appendChild(robots);
    const inspector = createElement('div', { class: 'mining-queue-inspector' });
    inspector.textContent = 'inspector stays';
    deck.appendChild(inspector);
    page.appendChild(deck);

    const config = createElement('script', {
        id: 'mining-queue-clear-config',
        type: 'application/json',
        textContent: '{"ores":{}}',
    });
    page.appendChild(config);
    body.appendChild(page);

    const main = createElement('div', { id: 'main-content' });
    main.appendChild(page);
    body.appendChild(main);

    return { body, page, hud, wallet, robots, inspector, config, main };
}

function loadMiningQueuePage(contextOverrides) {
    const doc = createMiningQueueDocument();
    const timers = [];
    const fetches = [];
    const context = Object.assign({
        console,
        Math,
        Number,
        Object,
        Array,
        String,
        isFinite,
        Date,
        FormData: class {
            constructor(form) {
                this.entries = form ? [['csrfToken', 'token-1'], ['robotId', '7']] : [];
            }
            append(key, value) {
                this.entries.push([key, String(value)]);
            }
            set(key, value) {
                for (var index = 0; index < this.entries.length; index += 1) {
                    if (this.entries[index][0] === key) {
                        this.entries[index][1] = String(value);
                        return;
                    }
                }
                this.entries.push([key, String(value)]);
            }
            forEach(callback) {
                for (var index = 0; index < this.entries.length; index += 1) {
                    callback(this.entries[index][1], this.entries[index][0]);
                }
            }
        },
        URLSearchParams: URLSearchParams,
        DOMParser: FakeDOMParser,
        document: {
            querySelector(selector) {
                if (selector === '.app-shell-hud') {
                    return doc.body.querySelector('.app-shell-hud') || doc.hud;
                }
                if (selector === '.mining-queue-page') {
                    return doc.page;
                }
                return doc.body.querySelector(selector);
            },
            querySelectorAll(selector) {
                return doc.body.querySelectorAll(selector);
            },
            getElementById(id) {
                if (id === 'main-content') {
                    return doc.main;
                }
                if (id === 'mining-queue-clear-config') {
                    return doc.config;
                }
                if (id === 'infoMiningAreaId') {
                    return null;
                }
                return doc.body.querySelector('#' + id);
            },
            createElement(tagName) {
                return createElement(tagName);
            },
            addEventListener() {},
        },
        window: null,
        globalThis: null,
        setTimeout(fn, delay) {
            timers.push({ fn: fn, delay: delay });
            return timers.length;
        },
        clearTimeout() {},
        clearInterval() {},
        setInterval() {
            return 1;
        },
        requestAnimationFrame(fn) {
            fn();
        },
        fetch(url, options) {
            fetches.push({ url: url, options: options });
            return Promise.resolve({
                ok: true,
                text() {
                    return Promise.resolve('');
                },
            });
        },
        location: { replace() {} },
        history: { replaceState() {} },
        RoboMinerSessionStore: {
            readJson() {
                return null;
            },
            writeJson() {},
        },
    }, contextOverrides || {});
    context.window = context;
    context.globalThis = context;
    vm.createContext(context);
    vm.runInContext(URL_QUERY_JS.replace('window', 'globalThis'), context, { filename: 'url_query.js' });
    vm.runInContext(SESSION_STORE_JS.replace('window', 'globalThis'), context, { filename: 'session_store.js' });
    vm.runInContext(VIEW_JS, context, { filename: 'view.js' });
    vm.runInContext(CLAIM_POLL_JS, context, { filename: 'claim_poll.js' });
    vm.runInContext(ACTIONS_JS, context, { filename: 'actions.js' });
    vm.runInContext(PAGE_JS, context, { filename: 'page.js' });
    return { context, doc, timers, fetches };
}

function buildRobotCard(options) {
    const form = createElement('form', {
        class: 'mining-queue-card',
        'data-robot-id': String(options.robotId),
        action: 'miningQueue',
        method: 'post',
    });
    if (options.csrfToken) {
        form.appendChild(createElement('input', {
            type: 'hidden',
            name: 'csrfToken',
            value: String(options.csrfToken),
        }));
    }
    form.appendChild(createElement('input', {
        type: 'hidden',
        name: 'robotId',
        value: String(options.robotId),
    }));

    const status = createElement('div', { class: 'mining-queue-card-status' });
    status.appendChild(createElement('span', {
        id: 'status-' + options.robotId,
        textContent: options.statusText,
    }));
    form.appendChild(status);

    const actions = createElement('div', { class: 'mining-queue-actions' });
    const select = createElement('select', {
        id: 'miningArea' + options.robotId,
        name: 'miningArea' + options.robotId,
        class: 'tableitem mining-queue-area-select',
    });
    const optionAttrs = { value: String(options.areaValue) };
    if (options.blockReason) {
        optionAttrs['data-block-reason'] = options.blockReason;
    }
    const option = createElement('option', optionAttrs);
    option.textContent = 'Area';
    option.value = String(options.areaValue);
    select.appendChild(option);
    select.options = [option];
    select.selectedIndex = 0;
    select.value = String(options.areaValue);
    actions.appendChild(select);

    const buttons = createElement('div', { class: 'mining-queue-action-buttons' });
    const addBtn = createElement('button', {
        type: 'submit',
        class: 'mining-queue-btn mining-queue-btn-primary',
        name: 'submitType',
        value: 'add',
    });
    addBtn.textContent = 'Add to queue';
    if (options.blockReason) {
        addBtn.disabled = true;
        addBtn.setAttribute('title', options.blockReason);
    }
    buttons.appendChild(addBtn);

    const fillBtn = createElement('button', {
        type: 'submit',
        class: 'mining-queue-btn',
        name: 'submitType',
        value: 'fill',
    });
    fillBtn.textContent = 'Fill queue';
    if (options.blockReason) {
        fillBtn.disabled = true;
        fillBtn.setAttribute('title', options.blockReason);
    }
    buttons.appendChild(fillBtn);

    const clearAttrs = {
        type: 'button',
        class: 'mining-queue-btn mining-queue-clear-btn',
        'data-clearable-count': String(options.clearableCount),
    };
    const clearBtn = createElement('button', clearAttrs);
    clearBtn.textContent = 'Clear queue';
    clearBtn.disabled = Number(options.clearableCount) === 0;
    if (clearBtn.disabled) {
        clearBtn.setAttribute('title', 'No queued runs to clear');
    }
    buttons.appendChild(clearBtn);

    const hint = createElement('p', { class: 'mining-queue-action-hint' });
    hint.textContent = options.blockReason || '';
    hint.hidden = !options.blockReason;
    buttons.appendChild(hint);
    actions.appendChild(buttons);
    form.appendChild(actions);
    return form;
}

module.exports = {
    createElement,
    loadMiningQueuePage,
    buildRobotCard,
    createMiningQueueDocument,
};
