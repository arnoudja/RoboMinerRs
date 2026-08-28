'use strict';

const fs = require('fs');
const path = require('path');
const { describe, it } = require('node:test');
const assert = require('node:assert/strict');
const vm = require('vm');

const JS_ROOT = path.join(__dirname, '..', '..');
const URL_QUERY_JS = fs.readFileSync(path.join(JS_ROOT, 'common', 'url_query.js'), 'utf8');
const SESSION_STORE_JS = fs.readFileSync(path.join(JS_ROOT, 'common', 'session_store.js'), 'utf8');
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

function matchesSelector(element, selector) {
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
    if (matchesSelector(root, selector)) {
        matches.unshift(root);
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
    vm.runInContext(PAGE_JS, context, { filename: 'page.js' });
    return { context, doc, timers, fetches };
}

describe('mining queue page partial updates', () => {
    it('buildFragmentUrl includes the queue fragment marker and area params', () => {
        const { context, doc } = loadMiningQueuePage();
        const select = createElement('select', { name: 'miningArea1' });
        select.value = '20';
        select.options = [{ value: '20' }];
        doc.page.appendChild(select);
        context.document.querySelectorAll = function(selector) {
            if (selector.indexOf('miningArea') >= 0) {
                return [select];
            }
            return [];
        };

        const url = context.RoboMinerMiningQueuePage.buildFragmentUrl();
        assert.match(url, /fragment=queue/);
        assert.match(url, /miningArea1=20/);
    });

    it('applyFragment swaps hud, wallet, robots, and clear config without touching inspector', () => {
        const { context, doc } = loadMiningQueuePage();
        const fragmentDoc = {
            getElementById(id) {
                if (id === 'mining-queue-fragment') {
                    return { id: id };
                }
                if (id === 'mining-queue-hud-fragment') {
                    const node = createElement('div', { id: id });
                    node.innerHTML = '<span id="new-hud">4/8</span>';
                    return node;
                }
                if (id === 'mining-queue-dynamic-fragment') {
                    const dynamic = createElement('div', { id: id });
                    const wallet = createElement('section', { class: 'page-wallet mining-queue-wallet' });
                    wallet.appendChild(createElement('span', { id: 'new-wallet', textContent: 'wallet' }));
                    dynamic.appendChild(wallet);
                    const error = createElement('p', { class: 'error mining-queue-error' });
                    error.textContent = 'new error';
                    dynamic.appendChild(error);
                    return dynamic;
                }
                if (id === 'mining-queue-robots-fragment') {
                    const robots = createElement('div', { id: id, class: 'mining-queue-robots' });
                    robots.innerHTML = '<form class="mining-queue-card"><span id="new-robot">new</span></form>';
                    return robots;
                }
                if (id === 'mining-queue-clear-config') {
                    const config = createElement('script', { id: id });
                    config.textContent = '{"ores":{"1":{"amount":3}}}';
                    return config;
                }
                return null;
            },
        };
        context.DOMParser = class {
            parseFromString() {
                return fragmentDoc;
            }
        };

        context.RoboMinerMiningQueuePage.applyFragment('<fragment>', doc.page);

        assert.match(doc.hud.innerHTML, /new-hud/);
        assert.match(doc.page.querySelector('.mining-queue-wallet').outerHTML, /new-wallet/);
        assert.match(doc.robots.innerHTML, /new-robot/);
        assert.equal(doc.config.textContent, '{"ores":{"1":{"amount":3}}}');
        assert.match(doc.page.querySelector('.mining-queue-error').textContent, /new error/);
        assert.equal(doc.inspector.textContent, 'inspector stays');
    });

    it('applyFragment replaces nested app-shell-hud without double wrapping', () => {
        const { context, doc } = loadMiningQueuePage();
        const fragmentDoc = {
            getElementById(id) {
                if (id === 'mining-queue-fragment') {
                    return { id: id };
                }
                if (id === 'mining-queue-hud-fragment') {
                    const node = createElement('div', { id: id });
                    const nestedHud = createElement('div', { class: 'app-shell-hud' });
                    nestedHud.appendChild(createElement('span', {
                        id: 'new-hud',
                        textContent: '4/8',
                    }));
                    node.appendChild(nestedHud);
                    return node;
                }
                if (id === 'mining-queue-dynamic-fragment') {
                    const dynamic = createElement('div', { id: id });
                    const wallet = createElement('section', { class: 'page-wallet mining-queue-wallet' });
                    wallet.appendChild(createElement('span', { id: 'new-wallet', textContent: 'wallet' }));
                    dynamic.appendChild(wallet);
                    return dynamic;
                }
                if (id === 'mining-queue-robots-fragment') {
                    return createElement('div', { id: id, class: 'mining-queue-robots' });
                }
                if (id === 'mining-queue-clear-config') {
                    const config = createElement('script', { id: id });
                    config.textContent = '{}';
                    return config;
                }
                return null;
            },
        };
        context.DOMParser = class {
            parseFromString() {
                return fragmentDoc;
            }
        };

        context.RoboMinerMiningQueuePage.applyFragment('<fragment>', doc.page);

        const hud = doc.body.querySelector('.app-shell-hud');
        assert.ok(hud);
        assert.match(hud.outerHTML, /new-hud/);
        assert.equal(
            Array.from(doc.body.querySelectorAll('.app-shell-hud')).length,
            1,
            'live header must contain exactly one .app-shell-hud'
        );
    });

    it('applyFragment skips empty HUD markup so the live header is not wiped', () => {
        const { context, doc } = loadMiningQueuePage();
        const original = doc.hud.innerHTML;
        const fragmentDoc = {
            getElementById(id) {
                if (id === 'mining-queue-fragment') {
                    return { id: id };
                }
                if (id === 'mining-queue-hud-fragment') {
                    return createElement('div', { id: id, innerHTML: '   ' });
                }
                if (id === 'mining-queue-dynamic-fragment') {
                    const dynamic = createElement('div', { id: id });
                    const wallet = createElement('section', { class: 'page-wallet mining-queue-wallet' });
                    wallet.appendChild(createElement('span', { id: 'new-wallet', textContent: 'wallet' }));
                    dynamic.appendChild(wallet);
                    return dynamic;
                }
                if (id === 'mining-queue-robots-fragment') {
                    return createElement('div', { id: id, class: 'mining-queue-robots' });
                }
                if (id === 'mining-queue-clear-config') {
                    const config = createElement('script', { id: id });
                    config.textContent = '{}';
                    return config;
                }
                return null;
            },
        };
        context.DOMParser = class {
            parseFromString() {
                return fragmentDoc;
            }
        };

        context.RoboMinerMiningQueuePage.applyFragment('<fragment>', doc.page);
        assert.equal(doc.hud.innerHTML, original);
    });

    it('claim refresh schedules backoff retries while finishing runs remain', async () => {
        const { context, timers, fetches, doc } = loadMiningQueuePage();
        // Match fragment wallet/HUD so a finishing-run poll does not look "already claimed".
        doc.hud.innerHTML = '';
        doc.hud.appendChild(createElement('span', { id: 'hud-amt', textContent: '1' }));
        const seededWallet = createElement('section', { class: 'page-wallet mining-queue-wallet' });
        seededWallet.appendChild(createElement('span', { id: 'wallet-amt', textContent: '1' }));
        doc.page.querySelector('.mining-queue-wallet').outerHTML = seededWallet.outerHTML;

        const fragmentDoc = {
            getElementById(id) {
                if (id === 'mining-queue-fragment') {
                    return { id: id };
                }
                if (id === 'mining-queue-hud-fragment') {
                    const node = createElement('div', { id: id });
                    const nestedHud = createElement('div', { class: 'app-shell-hud' });
                    nestedHud.appendChild(createElement('span', {
                        id: 'hud-amt',
                        textContent: '1',
                    }));
                    node.appendChild(nestedHud);
                    return node;
                }
                if (id === 'mining-queue-dynamic-fragment') {
                    const dynamic = createElement('div', { id: id });
                    const wallet = createElement('section', { class: 'page-wallet mining-queue-wallet' });
                    wallet.appendChild(createElement('span', { id: 'wallet-amt', textContent: '1' }));
                    dynamic.appendChild(wallet);
                    return dynamic;
                }
                if (id === 'mining-queue-robots-fragment') {
                    const robots = createElement('div', { id: id, class: 'mining-queue-robots' });
                    robots.appendChild(createElement('span', {
                        class: 'mining-queue-status mining-queue-status-updating',
                        textContent: 'Finishing rally',
                    }));
                    robots.appendChild(createElement('span', {
                        class: 'miningqueuetime',
                        'data-seconds-left': '0',
                        'data-refresh-on-complete': 'true',
                    }));
                    return robots;
                }
                if (id === 'mining-queue-clear-config') {
                    const config = createElement('script', { id: id });
                    config.textContent = '{}';
                    return config;
                }
                return null;
            },
        };
        context.DOMParser = class {
            parseFromString() {
                return fragmentDoc;
            }
        };
        context.fetch = function(url) {
            fetches.push({ url: url });
            return Promise.resolve({
                ok: true,
                text() {
                    return Promise.resolve('<fragment>');
                },
            });
        };

        // Ensure fragment application itself installs finishing-run markers.
        context.RoboMinerMiningQueuePage.applyFragment('<fragment>', doc.page);
        assert.ok(
            context.RoboMinerMiningQueuePage.hasFinishingRuns(doc.page),
            'applyFragment should install finishing-run markers'
        );

        await context.RoboMinerMiningQueuePage.performRefresh({ forClaim: true });
        assert.equal(fetches.length, 1);
        assert.ok(context.RoboMinerMiningQueuePage.hasFinishingRuns(doc.page));

        const retry = timers.find((timer) => timer.delay === context.RoboMinerMiningQueuePage.CLAIM_REFRESH_BACKOFF_MS[0]);
        assert.ok(retry, 'expected claim-refresh backoff timer');
    });

    it('formDataToUrlEncoded serializes fields for urlencoded POST bodies', () => {
        const { context } = loadMiningQueuePage();
        const formData = new context.FormData();
        formData.append('submitType', 'add');
        formData.append('robotId', '1');
        formData.append('miningArea1', '20');
        const encoded = context.RoboMinerMiningQueuePage.formDataToUrlEncoded(formData);
        assert.match(encoded, /(^|&)submitType=add(&|$)/);
        assert.match(encoded, /robotId=1/);
        assert.match(encoded, /miningArea1=20/);
    });

    it('refreshQueue debounces multiple timer completions into one fetch', () => {
        const { context, timers, fetches } = loadMiningQueuePage({
            fetch(url) {
                fetches.push({ url: url });
                return Promise.resolve({
                    ok: true,
                    text() {
                        return Promise.resolve([
                            '<div id="mining-queue-fragment">',
                            '<div id="mining-queue-hud-fragment"></div>',
                            '<div id="mining-queue-dynamic-fragment">',
                            '<section class="mining-queue-wallet"></section>',
                            '<div class="mining-queue-robots" id="mining-queue-robots-fragment"></div>',
                            '<script id="mining-queue-clear-config" type="application/json">{}</script>',
                            '</div>',
                            '</div>',
                        ].join(''));
                    },
                });
            },
        });

        context.RoboMinerMiningQueuePage.refreshQueue();
        context.RoboMinerMiningQueuePage.refreshQueue();
        assert.equal(fetches.length, 0);

        const debounced = timers.find((timer) => timer.delay === context.RoboMinerMiningQueuePage.REFRESH_DEBOUNCE_MS);
        assert.ok(debounced);
        debounced.fn();
        assert.equal(fetches.length, 1);
        assert.match(fetches[0].url, /fragment=queue/);
    });
});
